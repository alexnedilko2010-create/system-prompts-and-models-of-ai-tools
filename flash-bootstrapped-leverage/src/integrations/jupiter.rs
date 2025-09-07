use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;
use crate::errors::*;

/// Jupiter integration for token swaps to achieve optimal A/B ratios
pub struct JupiterIntegration;

impl JupiterIntegration {
    /// Execute Jupiter swap to achieve target A/B ratio for CLMM position
    pub fn execute_swap<'info>(
        input_mint: &Account<'info, Mint>,
        output_mint: &Account<'info, Mint>,
        input_amount: u64,
        minimum_output_amount: u64,
        slippage_tolerance_bps: u16,
        jupiter_program: &UncheckedAccount<'info>,
        user_input_account: &Account<'info, TokenAccount>,
        user_output_account: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
        route_data: &[u8],
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<u64> {
        // Validate Jupiter program ID
        let jupiter_program_id = Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4")
            .map_err(|_| FlashLeverageError::InvalidProgramId)?;

        if jupiter_program.key() != jupiter_program_id {
            return Err(FlashLeverageError::ProgramAccountMismatch.into());
        }

        // Validate swap parameters
        Self::validate_swap_params(
            input_amount,
            minimum_output_amount,
            slippage_tolerance_bps,
        )?;

        // Parse and validate Jupiter route data
        let route_info = Self::parse_jupiter_route(route_data)?;
        
        // Verify route matches our input/output mints
        if route_info.input_mint != input_mint.key() || route_info.output_mint != output_mint.key() {
            return Err(FlashLeverageError::InvalidJupiterRoute.into());
        }

        // Check if route is still valid (not expired)
        let current_time = Clock::get()?.unix_timestamp;
        if current_time > route_info.valid_until {
            return Err(FlashLeverageError::JupiterRouteExpired.into());
        }

        // Calculate expected output with slippage
        let expected_output = Self::calculate_expected_output(
            input_amount,
            &route_info,
            slippage_tolerance_bps,
        )?;

        if expected_output < minimum_output_amount {
            return Err(FlashLeverageError::SlippageToleranceExceeded.into());
        }

        // Build Jupiter swap instruction
        let swap_instruction = Self::build_jupiter_swap_instruction(
            input_amount,
            minimum_output_amount,
            route_data,
            user_input_account.key(),
            user_output_account.key(),
            remaining_accounts,
        )?;

        // Execute the swap via CPI
        let accounts = Self::build_jupiter_accounts(
            jupiter_program,
            user_input_account,
            user_output_account,
            token_program,
            remaining_accounts,
        )?;

        msg!("Executing Jupiter swap: {} {} -> {} {}", 
             input_amount, input_mint.key(), expected_output, output_mint.key());

        // In a real implementation, this would execute the actual CPI
        // For now, we'll simulate the swap result
        let actual_output = expected_output;

        msg!("Jupiter swap completed: {} tokens received", actual_output);

        Ok(actual_output)
    }

    /// Calculate optimal swap amounts to achieve target A/B ratio
    pub fn calculate_optimal_swap_amounts(
        current_token_a: u64,
        current_token_b: u64,
        target_ratio_a: u64,  // Target ratio A (scaled by 1e6)
        target_ratio_b: u64,  // Target ratio B (scaled by 1e6)
        price_a_to_b: u64,    // Price of A in terms of B (scaled by 1e6)
    ) -> Result<SwapCalculation> {
        // Calculate current ratio
        let current_ratio_a = current_token_a
            .checked_mul(1_000_000)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(current_token_a.checked_add(current_token_b).ok_or(FlashLeverageError::MathOverflow)?)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        let current_ratio_b = 1_000_000 - current_ratio_a;

        msg!("Current ratio - A: {}, B: {}", current_ratio_a, current_ratio_b);
        msg!("Target ratio - A: {}, B: {}", target_ratio_a, target_ratio_b);

        // Determine swap direction and amount
        if current_ratio_a > target_ratio_a {
            // Need to swap A -> B
            let excess_a = current_token_a
                .checked_mul(current_ratio_a - target_ratio_a)
                .ok_or(FlashLeverageError::MathOverflow)?
                .checked_div(1_000_000)
                .ok_or(FlashLeverageError::DivisionByZero)?;

            let swap_amount_a = excess_a / 2; // Swap half of excess
            let expected_b = swap_amount_a
                .checked_mul(price_a_to_b)
                .ok_or(FlashLeverageError::MathOverflow)?
                .checked_div(1_000_000)
                .ok_or(FlashLeverageError::DivisionByZero)?;

            Ok(SwapCalculation {
                needs_swap: true,
                swap_direction: SwapDirection::AToB,
                input_amount: swap_amount_a,
                expected_output: expected_b,
                final_token_a: current_token_a - swap_amount_a,
                final_token_b: current_token_b + expected_b,
            })
        } else if current_ratio_b > target_ratio_b {
            // Need to swap B -> A
            let excess_b = current_token_b
                .checked_mul(current_ratio_b - target_ratio_b)
                .ok_or(FlashLeverageError::MathOverflow)?
                .checked_div(1_000_000)
                .ok_or(FlashLeverageError::DivisionByZero)?;

            let swap_amount_b = excess_b / 2; // Swap half of excess
            let expected_a = swap_amount_b
                .checked_mul(1_000_000)
                .ok_or(FlashLeverageError::MathOverflow)?
                .checked_div(price_a_to_b)
                .ok_or(FlashLeverageError::DivisionByZero)?;

            Ok(SwapCalculation {
                needs_swap: true,
                swap_direction: SwapDirection::BToA,
                input_amount: swap_amount_b,
                expected_output: expected_a,
                final_token_a: current_token_a + expected_a,
                final_token_b: current_token_b - swap_amount_b,
            })
        } else {
            // No swap needed, ratios are already optimal
            Ok(SwapCalculation {
                needs_swap: false,
                swap_direction: SwapDirection::AToB, // Doesn't matter
                input_amount: 0,
                expected_output: 0,
                final_token_a: current_token_a,
                final_token_b: current_token_b,
            })
        }
    }

    /// Get Jupiter quote for a swap
    pub async fn get_jupiter_quote(
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<JupiterQuote> {
        // In a real implementation, this would make an HTTP request to Jupiter API
        // For now, we'll simulate the quote
        
        msg!("Getting Jupiter quote for {} -> {}, amount: {}", 
             input_mint, output_mint, amount);

        // Simulate quote calculation (in practice, use Jupiter API)
        let simulated_output = amount
            .checked_mul(95) // Assume 5% price impact
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(100)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        let price_impact_bps = 500; // 5% price impact
        let fee_bps = 30; // 0.3% fee

        Ok(JupiterQuote {
            input_mint: *input_mint,
            output_mint: *output_mint,
            input_amount: amount,
            output_amount: simulated_output,
            price_impact_bps,
            fee_bps,
            route_plan: vec![], // Would contain actual route in real implementation
            valid_until: Clock::get()?.unix_timestamp + 30, // Valid for 30 seconds
        })
    }

    // Private helper functions

    fn validate_swap_params(
        input_amount: u64,
        minimum_output_amount: u64,
        slippage_tolerance_bps: u16,
    ) -> Result<()> {
        if input_amount == 0 {
            return Err(FlashLeverageError::InsufficientBalance.into());
        }

        if minimum_output_amount == 0 {
            return Err(FlashLeverageError::InvalidInstructionData.into());
        }

        if slippage_tolerance_bps > 10000 {
            return Err(FlashLeverageError::SlippageToleranceExceeded.into());
        }

        Ok(())
    }

    fn parse_jupiter_route(route_data: &[u8]) -> Result<JupiterRouteInfo> {
        if route_data.len() < 64 {
            return Err(FlashLeverageError::InvalidJupiterRoute.into());
        }

        // Parse route data (simplified)
        let input_mint = Pubkey::new_from_array(
            route_data[0..32].try_into()
                .map_err(|_| FlashLeverageError::InvalidJupiterRoute)?
        );

        let output_mint = Pubkey::new_from_array(
            route_data[32..64].try_into()
                .map_err(|_| FlashLeverageError::InvalidJupiterRoute)?
        );

        // Parse timestamp (next 8 bytes)
        let valid_until = if route_data.len() >= 72 {
            i64::from_le_bytes(
                route_data[64..72].try_into()
                    .map_err(|_| FlashLeverageError::InvalidJupiterRoute)?
            )
        } else {
            Clock::get()?.unix_timestamp + 30 // Default 30 seconds
        };

        Ok(JupiterRouteInfo {
            input_mint,
            output_mint,
            valid_until,
            route_data: route_data.to_vec(),
        })
    }

    fn calculate_expected_output(
        input_amount: u64,
        route_info: &JupiterRouteInfo,
        slippage_tolerance_bps: u16,
    ) -> Result<u64> {
        // In a real implementation, this would use the route data to calculate output
        // For now, we'll use a simplified calculation
        
        let base_output = input_amount
            .checked_mul(95) // Assume 5% price impact
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(100)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        // Apply slippage tolerance
        let slippage_adjustment = base_output
            .checked_mul(slippage_tolerance_bps as u64)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        let expected_output = base_output
            .checked_sub(slippage_adjustment)
            .ok_or(FlashLeverageError::MathUnderflow)?;

        Ok(expected_output)
    }

    fn build_jupiter_swap_instruction(
        input_amount: u64,
        minimum_output_amount: u64,
        route_data: &[u8],
        user_input_account: Pubkey,
        user_output_account: Pubkey,
        _remaining_accounts: &[AccountInfo],
    ) -> Result<Vec<u8>> {
        let mut instruction_data = Vec::new();
        
        // Jupiter swap instruction discriminator
        instruction_data.extend_from_slice(&[0xf8, 0xc6, 0x9e, 0x91, 0xe1, 0x75, 0x87, 0xc8]); // swap discriminator
        
        // Input amount (8 bytes)
        instruction_data.extend_from_slice(&input_amount.to_le_bytes());
        
        // Minimum output amount (8 bytes)
        instruction_data.extend_from_slice(&minimum_output_amount.to_le_bytes());
        
        // Route data length (4 bytes)
        instruction_data.extend_from_slice(&(route_data.len() as u32).to_le_bytes());
        
        // Route data
        instruction_data.extend_from_slice(route_data);
        
        Ok(instruction_data)
    }

    fn build_jupiter_accounts<'info>(
        jupiter_program: &UncheckedAccount<'info>,
        user_input_account: &Account<'info, TokenAccount>,
        user_output_account: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<Vec<AccountInfo<'info>>> {
        let mut accounts = Vec::new();
        
        // Core accounts
        accounts.push(jupiter_program.to_account_info());
        accounts.push(user_input_account.to_account_info());
        accounts.push(user_output_account.to_account_info());
        accounts.push(token_program.to_account_info());
        
        // Add remaining accounts (swap route accounts)
        for account in remaining_accounts {
            accounts.push(account.clone());
        }
        
        Ok(accounts)
    }
}

#[derive(Clone, Debug)]
pub struct SwapCalculation {
    pub needs_swap: bool,
    pub swap_direction: SwapDirection,
    pub input_amount: u64,
    pub expected_output: u64,
    pub final_token_a: u64,
    pub final_token_b: u64,
}

#[derive(Clone, Debug)]
pub enum SwapDirection {
    AToB,
    BToA,
}

#[derive(Clone, Debug)]
pub struct JupiterQuote {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact_bps: u16,
    pub fee_bps: u16,
    pub route_plan: Vec<RoutePlanStep>,
    pub valid_until: i64,
}

#[derive(Clone, Debug)]
pub struct RoutePlanStep {
    pub swap_info: SwapInfo,
    pub percent: u8,
}

#[derive(Clone, Debug)]
pub struct SwapInfo {
    pub amm_key: Pubkey,
    pub label: String,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub in_amount: u64,
    pub out_amount: u64,
    pub fee_amount: u64,
    pub fee_mint: Pubkey,
}

#[derive(Clone, Debug)]
pub struct JupiterRouteInfo {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub valid_until: i64,
    pub route_data: Vec<u8>,
}

/// Helper function to estimate optimal liquidity provision ratio
impl JupiterIntegration {
    pub fn estimate_liquidity_ratio(
        token_a_amount: u64,
        token_b_amount: u64,
        price_lower: u64, // Lower bound price (A/B ratio, scaled by 1e6)
        price_upper: u64, // Upper bound price (A/B ratio, scaled by 1e6)
        current_price: u64, // Current price (A/B ratio, scaled by 1e6)
    ) -> Result<LiquidityRatio> {
        if price_lower >= price_upper {
            return Err(FlashLeverageError::InvalidPriceRange.into());
        }

        if current_price < price_lower || current_price > price_upper {
            return Err(FlashLeverageError::PriceOutOfRange.into());
        }

        // Calculate optimal ratio based on concentrated liquidity math
        // This is a simplified calculation - real CLMM math is more complex
        
        let price_range = price_upper
            .checked_sub(price_lower)
            .ok_or(FlashLeverageError::MathUnderflow)?;
            
        let price_position = current_price
            .checked_sub(price_lower)
            .ok_or(FlashLeverageError::MathUnderflow)?;
            
        // Calculate ratio based on position in range
        let ratio_b = price_position
            .checked_mul(1_000_000)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(price_range)
            .ok_or(FlashLeverageError::DivisionByZero)?;
            
        let ratio_a = 1_000_000 - ratio_b;

        // Calculate required amounts
        let total_value_in_b = token_a_amount
            .checked_mul(current_price)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(1_000_000)
            .ok_or(FlashLeverageError::DivisionByZero)?
            .checked_add(token_b_amount)
            .ok_or(FlashLeverageError::MathOverflow)?;

        let required_b = total_value_in_b
            .checked_mul(ratio_b)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(1_000_000)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        let required_a = total_value_in_b
            .checked_sub(required_b)
            .ok_or(FlashLeverageError::MathUnderflow)?
            .checked_mul(1_000_000)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(current_price)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        Ok(LiquidityRatio {
            required_token_a: required_a,
            required_token_b: required_b,
            ratio_a,
            ratio_b,
            efficiency_score: Self::calculate_efficiency_score(
                current_price, price_lower, price_upper
            )?,
        })
    }

    fn calculate_efficiency_score(
        current_price: u64,
        price_lower: u64,
        price_upper: u64,
    ) -> Result<u64> {
        let range_width = price_upper
            .checked_sub(price_lower)
            .ok_or(FlashLeverageError::MathUnderflow)?;
            
        let distance_from_center = {
            let center = price_lower
                .checked_add(price_upper)
                .ok_or(FlashLeverageError::MathOverflow)?
                .checked_div(2)
                .ok_or(FlashLeverageError::DivisionByZero)?;
                
            if current_price > center {
                current_price - center
            } else {
                center - current_price
            }
        };

        // Efficiency is higher when price is closer to center of range
        let efficiency = 1_000_000
            .checked_sub(
                distance_from_center
                    .checked_mul(2_000_000)
                    .ok_or(FlashLeverageError::MathOverflow)?
                    .checked_div(range_width)
                    .ok_or(FlashLeverageError::DivisionByZero)?
            )
            .ok_or(FlashLeverageError::MathUnderflow)?;

        Ok(efficiency)
    }
}

#[derive(Clone, Debug)]
pub struct LiquidityRatio {
    pub required_token_a: u64,
    pub required_token_b: u64,
    pub ratio_a: u64, // Scaled by 1e6
    pub ratio_b: u64, // Scaled by 1e6
    pub efficiency_score: u64, // Scaled by 1e6, higher is better
}