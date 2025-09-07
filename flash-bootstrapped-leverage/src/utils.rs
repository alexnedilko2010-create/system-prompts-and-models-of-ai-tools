use anchor_lang::prelude::*;
use crate::errors::*;
use crate::state::*;

/// Utility functions for flash-bootstrapped leverage operations
pub struct Utils;

impl Utils {
    /// Calculate the effective leverage ratio for a position
    pub fn calculate_effective_leverage(
        total_position_value: u64,
        user_capital: u64,
    ) -> Result<u64> {
        if user_capital == 0 {
            return Err(FlashLeverageError::DivisionByZero.into());
        }

        let leverage = total_position_value
            .checked_mul(100) // Scale by 100 for percentage
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(user_capital)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        Ok(leverage)
    }

    /// Calculate position health factor
    pub fn calculate_health_factor(
        collateral_value_usd: u64,
        debt_value_usd: u64,
        liquidation_threshold_bps: u16,
    ) -> Result<u64> {
        if debt_value_usd == 0 {
            return Ok(u64::MAX); // Infinite health factor
        }

        let adjusted_collateral = collateral_value_usd
            .checked_mul(liquidation_threshold_bps as u64)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        let health_factor = adjusted_collateral
            .checked_mul(1_000_000) // Scale by 1e6
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(debt_value_usd)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        Ok(health_factor)
    }

    /// Calculate impermanent loss for a concentrated liquidity position
    pub fn calculate_impermanent_loss(
        initial_price: u64,
        current_price: u64,
        price_lower: u64,
        price_upper: u64,
    ) -> Result<ImpermanentLossInfo> {
        // Check if current price is within range
        let in_range = current_price >= price_lower && current_price <= price_upper;
        
        // Calculate price change ratio
        let price_ratio = current_price
            .checked_mul(1_000_000)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(initial_price)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        // Calculate IL based on concentrated liquidity math
        let il_percentage = if in_range {
            // Reduced IL for concentrated positions within range
            Self::calculate_concentrated_il(price_ratio, price_lower, price_upper, initial_price)?
        } else {
            // Higher IL when out of range (single-sided exposure)
            if current_price < price_lower {
                // All position is in token A
                1_000_000 - price_ratio // 100% - price_ratio
            } else {
                // All position is in token B
                price_ratio - 1_000_000 // price_ratio - 100%
            }
        };

        Ok(ImpermanentLossInfo {
            il_percentage,
            in_range,
            price_ratio,
            severity: Self::classify_il_severity(il_percentage),
        })
    }

    /// Calculate expected APY from fees for a CLMM position
    pub fn calculate_fee_apy(
        liquidity_amount: u128,
        pool_volume_24h: u64,
        fee_tier_bps: u16,
        total_liquidity: u128,
        concentration_factor: u64, // How concentrated the position is
    ) -> Result<u64> {
        if total_liquidity == 0 {
            return Ok(0);
        }

        // Calculate daily fees earned
        let daily_fees = (pool_volume_24h as u128)
            .checked_mul(fee_tier_bps as u128)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        // Calculate position's share of fees
        let position_share = liquidity_amount
            .checked_mul(1_000_000)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(total_liquidity)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        let position_daily_fees = daily_fees
            .checked_mul(position_share)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(1_000_000)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        // Apply concentration factor (concentrated positions can earn more fees per dollar)
        let adjusted_daily_fees = position_daily_fees
            .checked_mul(concentration_factor as u128)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(1_000_000)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        // Annualize (365 days)
        let annual_fees = adjusted_daily_fees
            .checked_mul(365)
            .ok_or(FlashLeverageError::MathOverflow)?;

        // Calculate APY as percentage (scaled by 1e6)
        let position_value = liquidity_amount / 1_000_000; // Simplified position value
        if position_value == 0 {
            return Ok(0);
        }

        let apy = (annual_fees as u64)
            .checked_mul(1_000_000)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(position_value as u64)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        Ok(apy)
    }

    /// Calculate optimal rebalancing threshold
    pub fn calculate_rebalancing_threshold(
        current_price: u64,
        price_lower: u64,
        price_upper: u64,
        volatility_bps: u16, // Annualized volatility in basis points
    ) -> Result<RebalancingThreshold> {
        let price_range = price_upper
            .checked_sub(price_lower)
            .ok_or(FlashLeverageError::MathUnderflow)?;

        // Calculate distance from range boundaries
        let distance_to_lower = current_price
            .checked_sub(price_lower)
            .ok_or(FlashLeverageError::MathUnderflow)?;
        
        let distance_to_upper = price_upper
            .checked_sub(current_price)
            .ok_or(FlashLeverageError::MathUnderflow)?;

        // Calculate volatility-adjusted threshold
        let volatility_factor = volatility_bps as u64;
        let base_threshold = price_range / 10; // 10% of range as base
        
        let volatility_adjustment = base_threshold
            .checked_mul(volatility_factor)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(1000) // Adjust for basis points
            .ok_or(FlashLeverageError::DivisionByZero)?;

        let lower_threshold = price_lower
            .checked_add(base_threshold + volatility_adjustment)
            .ok_or(FlashLeverageError::MathOverflow)?;

        let upper_threshold = price_upper
            .checked_sub(base_threshold + volatility_adjustment)
            .ok_or(FlashLeverageError::MathUnderflow)?;

        let needs_rebalancing = current_price <= lower_threshold || current_price >= upper_threshold;

        Ok(RebalancingThreshold {
            lower_threshold,
            upper_threshold,
            needs_rebalancing,
            urgency: Self::calculate_rebalancing_urgency(
                current_price,
                lower_threshold,
                upper_threshold,
                price_lower,
                price_upper,
            )?,
        })
    }

    /// Validate transaction deadline
    pub fn validate_deadline(deadline: i64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        if current_time > deadline {
            return Err(FlashLeverageError::DeadlineExceeded.into());
        }

        // Check if deadline is too far in the future (max 1 hour)
        if deadline > current_time + 3600 {
            return Err(FlashLeverageError::DeadlineExceeded.into());
        }

        Ok(())
    }

    /// Calculate gas optimization parameters
    pub fn calculate_gas_optimization(
        operation_complexity: u8, // 1-10 scale
        network_congestion: u8,   // 1-10 scale
        priority_level: PriorityLevel,
    ) -> Result<GasOptimization> {
        let base_compute_units = match operation_complexity {
            1..=3 => 200_000,
            4..=6 => 400_000,
            7..=8 => 600_000,
            9..=10 => 800_000,
            _ => return Err(FlashLeverageError::InvalidInstructionData.into()),
        };

        let congestion_multiplier = match network_congestion {
            1..=3 => 100, // 1.0x
            4..=6 => 120, // 1.2x
            7..=8 => 150, // 1.5x
            9..=10 => 200, // 2.0x
            _ => return Err(FlashLeverageError::InvalidInstructionData.into()),
        };

        let priority_multiplier = match priority_level {
            PriorityLevel::Low => 80,    // 0.8x
            PriorityLevel::Normal => 100, // 1.0x
            PriorityLevel::High => 130,  // 1.3x
            PriorityLevel::Critical => 200, // 2.0x
        };

        let compute_units = base_compute_units
            .checked_mul(congestion_multiplier)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(100)
            .ok_or(FlashLeverageError::DivisionByZero)?
            .checked_mul(priority_multiplier)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(100)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        let priority_fee = match priority_level {
            PriorityLevel::Low => 0,
            PriorityLevel::Normal => 1000,      // 0.001 SOL
            PriorityLevel::High => 5000,       // 0.005 SOL
            PriorityLevel::Critical => 10000,  // 0.01 SOL
        };

        Ok(GasOptimization {
            compute_units,
            priority_fee,
            estimated_cost: compute_units / 1000 + priority_fee, // Simplified cost calculation
        })
    }

    // Private helper functions

    fn calculate_concentrated_il(
        price_ratio: u64,
        price_lower: u64,
        price_upper: u64,
        initial_price: u64,
    ) -> Result<u64> {
        // Simplified concentrated IL calculation
        // Real implementation would use complex CLMM mathematics
        
        let range_width = price_upper
            .checked_sub(price_lower)
            .ok_or(FlashLeverageError::MathUnderflow)?;
        
        let price_deviation = if price_ratio > 1_000_000 {
            price_ratio - 1_000_000
        } else {
            1_000_000 - price_ratio
        };

        // IL is reduced by concentration factor
        let concentration_factor = 2_000_000u64 // 2.0x range
            .checked_mul(1_000_000)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(range_width)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        let concentrated_il = price_deviation
            .checked_div(concentration_factor.max(1_000_000))
            .ok_or(FlashLeverageError::DivisionByZero)?;

        Ok(concentrated_il)
    }

    fn classify_il_severity(il_percentage: u64) -> ILSeverity {
        match il_percentage {
            0..=50_000 => ILSeverity::Low,        // 0-5%
            50_001..=150_000 => ILSeverity::Medium, // 5-15%
            150_001..=300_000 => ILSeverity::High,  // 15-30%
            _ => ILSeverity::Critical,             // >30%
        }
    }

    fn calculate_rebalancing_urgency(
        current_price: u64,
        lower_threshold: u64,
        upper_threshold: u64,
        price_lower: u64,
        price_upper: u64,
    ) -> Result<RebalancingUrgency> {
        if current_price >= lower_threshold && current_price <= upper_threshold {
            return Ok(RebalancingUrgency::None);
        }

        // Calculate how close we are to the range boundaries
        let distance_to_boundary = if current_price < lower_threshold {
            lower_threshold - current_price
        } else {
            current_price - upper_threshold
        };

        let total_range = price_upper
            .checked_sub(price_lower)
            .ok_or(FlashLeverageError::MathUnderflow)?;

        let urgency_ratio = distance_to_boundary
            .checked_mul(100)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(total_range)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        match urgency_ratio {
            0..=10 => Ok(RebalancingUrgency::Low),
            11..=25 => Ok(RebalancingUrgency::Medium),
            26..=50 => Ok(RebalancingUrgency::High),
            _ => Ok(RebalancingUrgency::Critical),
        }
    }
}

// Helper structs and enums

#[derive(Clone, Debug)]
pub struct ImpermanentLossInfo {
    pub il_percentage: u64,  // Scaled by 1e6
    pub in_range: bool,
    pub price_ratio: u64,    // Scaled by 1e6
    pub severity: ILSeverity,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ILSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug)]
pub struct RebalancingThreshold {
    pub lower_threshold: u64,
    pub upper_threshold: u64,
    pub needs_rebalancing: bool,
    pub urgency: RebalancingUrgency,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RebalancingUrgency {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PriorityLevel {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Clone, Debug)]
pub struct GasOptimization {
    pub compute_units: u64,
    pub priority_fee: u64,      // In lamports
    pub estimated_cost: u64,    // In lamports
}

/// Risk assessment utilities
impl Utils {
    /// Assess overall position risk
    pub fn assess_position_risk(
        health_factor: u64,
        il_info: &ImpermanentLossInfo,
        market_volatility_bps: u16,
        liquidity_utilization_bps: u16,
    ) -> Result<RiskAssessment> {
        let health_risk = Self::assess_health_risk(health_factor);
        let il_risk = Self::assess_il_risk(il_info);
        let market_risk = Self::assess_market_risk(market_volatility_bps);
        let liquidity_risk = Self::assess_liquidity_risk(liquidity_utilization_bps);

        // Calculate composite risk score (0-100)
        let composite_score = (health_risk as u32 + il_risk as u32 + market_risk as u32 + liquidity_risk as u32) / 4;

        let overall_risk = match composite_score {
            0..=25 => RiskLevel::Low,
            26..=50 => RiskLevel::Medium,
            51..=75 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        Ok(RiskAssessment {
            overall_risk,
            health_risk_score: health_risk,
            il_risk_score: il_risk,
            market_risk_score: market_risk,
            liquidity_risk_score: liquidity_risk,
            composite_score: composite_score as u8,
            recommendations: Self::generate_risk_recommendations(overall_risk),
        })
    }

    fn assess_health_risk(health_factor: u64) -> u8 {
        match health_factor {
            2_000_000.. => 0,           // >2.0: Very safe
            1_500_000..=1_999_999 => 20, // 1.5-2.0: Safe
            1_200_000..=1_499_999 => 40, // 1.2-1.5: Moderate
            1_050_000..=1_199_999 => 70, // 1.05-1.2: Risky
            _ => 100,                    // <1.05: Critical
        }
    }

    fn assess_il_risk(il_info: &ImpermanentLossInfo) -> u8 {
        match il_info.severity {
            ILSeverity::Low => 10,
            ILSeverity::Medium => 30,
            ILSeverity::High => 60,
            ILSeverity::Critical => 90,
        }
    }

    fn assess_market_risk(volatility_bps: u16) -> u8 {
        match volatility_bps {
            0..=1000 => 10,      // <10% volatility: Low risk
            1001..=2500 => 30,   // 10-25%: Medium risk
            2501..=5000 => 60,   // 25-50%: High risk
            _ => 90,             // >50%: Critical risk
        }
    }

    fn assess_liquidity_risk(utilization_bps: u16) -> u8 {
        match utilization_bps {
            0..=5000 => 10,      // <50% utilization: Low risk
            5001..=7500 => 30,   // 50-75%: Medium risk
            7501..=9000 => 60,   // 75-90%: High risk
            _ => 90,             // >90%: Critical risk
        }
    }

    fn generate_risk_recommendations(risk_level: RiskLevel) -> Vec<String> {
        match risk_level {
            RiskLevel::Low => vec![
                "Position is healthy".to_string(),
                "Consider taking profits if targets are met".to_string(),
            ],
            RiskLevel::Medium => vec![
                "Monitor position closely".to_string(),
                "Consider reducing leverage if volatility increases".to_string(),
            ],
            RiskLevel::High => vec![
                "Consider reducing position size".to_string(),
                "Prepare for potential rebalancing".to_string(),
                "Monitor health factor closely".to_string(),
            ],
            RiskLevel::Critical => vec![
                "Immediate action required".to_string(),
                "Consider emergency position closure".to_string(),
                "Risk of liquidation is high".to_string(),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub health_risk_score: u8,      // 0-100
    pub il_risk_score: u8,          // 0-100
    pub market_risk_score: u8,      // 0-100
    pub liquidity_risk_score: u8,   // 0-100
    pub composite_score: u8,        // 0-100
    pub recommendations: Vec<String>,
}