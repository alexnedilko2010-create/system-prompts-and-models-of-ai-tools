use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;
use crate::errors::*;

/// CLMM (Concentrated Liquidity Market Maker) integration for multiple providers
pub struct CLMMIntegration;

impl CLMMIntegration {
    /// Increase liquidity in a CLMM pool
    pub fn increase_liquidity<'info>(
        provider: &CLMMProvider,
        token_a_amount: u64,
        token_b_amount: u64,
        tick_lower: i32,
        tick_upper: i32,
        clmm_program: &UncheckedAccount<'info>,
        pool_account: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        user_token_a_account: &Account<'info, TokenAccount>,
        user_token_b_account: &Account<'info, TokenAccount>,
        token_a_mint: &Account<'info, Mint>,
        token_b_mint: &Account<'info, Mint>,
        token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMLiquidityResult> {
        match provider {
            CLMMProvider::OrcaWhirlpool => {
                Self::increase_orca_whirlpool_liquidity(
                    token_a_amount,
                    token_b_amount,
                    tick_lower,
                    tick_upper,
                    clmm_program,
                    pool_account,
                    position_account,
                    user_token_a_account,
                    user_token_b_account,
                    token_a_mint,
                    token_b_mint,
                    token_program,
                    remaining_accounts,
                )
            }
            CLMMProvider::RaydiumCLMM => {
                Self::increase_raydium_clmm_liquidity(
                    token_a_amount,
                    token_b_amount,
                    tick_lower,
                    tick_upper,
                    clmm_program,
                    pool_account,
                    position_account,
                    user_token_a_account,
                    user_token_b_account,
                    token_a_mint,
                    token_b_mint,
                    token_program,
                    remaining_accounts,
                )
            }
            CLMMProvider::MeteoraDLMM => {
                Self::increase_meteora_dlmm_liquidity(
                    token_a_amount,
                    token_b_amount,
                    tick_lower,
                    tick_upper,
                    clmm_program,
                    pool_account,
                    position_account,
                    user_token_a_account,
                    user_token_b_account,
                    token_a_mint,
                    token_b_mint,
                    token_program,
                    remaining_accounts,
                )
            }
            CLMMProvider::Custom(_) => {
                Err(FlashLeverageError::FeatureNotImplemented.into())
            }
        }
    }

    /// Decrease liquidity from a CLMM position
    pub fn decrease_liquidity<'info>(
        provider: &CLMMProvider,
        liquidity_amount: u128,
        clmm_program: &UncheckedAccount<'info>,
        pool_account: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        user_token_a_account: &Account<'info, TokenAccount>,
        user_token_b_account: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMLiquidityResult> {
        match provider {
            CLMMProvider::OrcaWhirlpool => {
                Self::decrease_orca_whirlpool_liquidity(
                    liquidity_amount,
                    clmm_program,
                    pool_account,
                    position_account,
                    user_token_a_account,
                    user_token_b_account,
                    token_program,
                    remaining_accounts,
                )
            }
            CLMMProvider::RaydiumCLMM => {
                Self::decrease_raydium_clmm_liquidity(
                    liquidity_amount,
                    clmm_program,
                    pool_account,
                    position_account,
                    user_token_a_account,
                    user_token_b_account,
                    token_program,
                    remaining_accounts,
                )
            }
            CLMMProvider::MeteoraDLMM => {
                Self::decrease_meteora_dlmm_liquidity(
                    liquidity_amount,
                    clmm_program,
                    pool_account,
                    position_account,
                    user_token_a_account,
                    user_token_b_account,
                    token_program,
                    remaining_accounts,
                )
            }
            CLMMProvider::Custom(_) => {
                Err(FlashLeverageError::FeatureNotImplemented.into())
            }
        }
    }

    /// Collect fees from a CLMM position
    pub fn collect_fees<'info>(
        provider: &CLMMProvider,
        clmm_program: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        user_token_a_account: &Account<'info, TokenAccount>,
        user_token_b_account: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMFeeCollection> {
        match provider {
            CLMMProvider::OrcaWhirlpool => {
                Self::collect_orca_whirlpool_fees(
                    clmm_program,
                    position_account,
                    user_token_a_account,
                    user_token_b_account,
                    token_program,
                    remaining_accounts,
                )
            }
            CLMMProvider::RaydiumCLMM => {
                Self::collect_raydium_clmm_fees(
                    clmm_program,
                    position_account,
                    user_token_a_account,
                    user_token_b_account,
                    token_program,
                    remaining_accounts,
                )
            }
            CLMMProvider::MeteoraDLMM => {
                Self::collect_meteora_dlmm_fees(
                    clmm_program,
                    position_account,
                    user_token_a_account,
                    user_token_b_account,
                    token_program,
                    remaining_accounts,
                )
            }
            CLMMProvider::Custom(_) => {
                Err(FlashLeverageError::FeatureNotImplemented.into())
            }
        }
    }

    // Orca Whirlpool implementations
    fn increase_orca_whirlpool_liquidity<'info>(
        token_a_amount: u64,
        token_b_amount: u64,
        tick_lower: i32,
        tick_upper: i32,
        clmm_program: &UncheckedAccount<'info>,
        pool_account: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        user_token_a_account: &Account<'info, TokenAccount>,
        user_token_b_account: &Account<'info, TokenAccount>,
        _token_a_mint: &Account<'info, Mint>,
        _token_b_mint: &Account<'info, Mint>,
        _token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMLiquidityResult> {
        // Validate Orca Whirlpool program ID
        let whirlpool_program_id = Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc")
            .map_err(|_| FlashLeverageError::InvalidProgramId)?;

        if clmm_program.key() != whirlpool_program_id {
            return Err(FlashLeverageError::ProgramAccountMismatch.into());
        }

        // Validate tick range
        Self::validate_tick_range(tick_lower, tick_upper)?;

        // Validate required accounts for Orca Whirlpool
        if remaining_accounts.len() < 8 {
            return Err(FlashLeverageError::AccountValidationFailed.into());
        }

        // Calculate liquidity amount based on token amounts and tick range
        let liquidity_amount = Self::calculate_liquidity_for_amounts(
            token_a_amount,
            token_b_amount,
            tick_lower,
            tick_upper,
        )?;

        msg!("Increasing Orca Whirlpool liquidity: {} A, {} B, ticks: {}-{}", 
             token_a_amount, token_b_amount, tick_lower, tick_upper);

        // Build Orca increase liquidity instruction
        let instruction_data = Self::build_orca_increase_liquidity_data(
            liquidity_amount,
            token_a_amount,
            token_b_amount,
        )?;

        // In a real implementation, execute CPI to Orca Whirlpool
        // For now, simulate the result
        let actual_liquidity = liquidity_amount;
        let actual_token_a = token_a_amount;
        let actual_token_b = token_b_amount;

        msg!("Orca Whirlpool liquidity added: {} liquidity units", actual_liquidity);

        Ok(CLMMLiquidityResult {
            liquidity_amount: actual_liquidity,
            token_a_amount: actual_token_a,
            token_b_amount: actual_token_b,
            position_nft: Some(position_account.key()),
        })
    }

    fn decrease_orca_whirlpool_liquidity<'info>(
        liquidity_amount: u128,
        clmm_program: &UncheckedAccount<'info>,
        _pool_account: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        _user_token_a_account: &Account<'info, TokenAccount>,
        _user_token_b_account: &Account<'info, TokenAccount>,
        _token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMLiquidityResult> {
        // Validate Orca Whirlpool program
        let whirlpool_program_id = Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc")
            .map_err(|_| FlashLeverageError::InvalidProgramId)?;

        if clmm_program.key() != whirlpool_program_id {
            return Err(FlashLeverageError::ProgramAccountMismatch.into());
        }

        if remaining_accounts.len() < 6 {
            return Err(FlashLeverageError::AccountValidationFailed.into());
        }

        msg!("Decreasing Orca Whirlpool liquidity: {} units", liquidity_amount);

        // Calculate token amounts to receive based on liquidity
        let (token_a_amount, token_b_amount) = Self::calculate_tokens_for_liquidity(
            liquidity_amount,
            remaining_accounts,
        )?;

        // Execute decrease liquidity instruction
        // In real implementation, this would be a CPI call

        Ok(CLMMLiquidityResult {
            liquidity_amount,
            token_a_amount,
            token_b_amount,
            position_nft: Some(position_account.key()),
        })
    }

    fn collect_orca_whirlpool_fees<'info>(
        clmm_program: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        _user_token_a_account: &Account<'info, TokenAccount>,
        _user_token_b_account: &Account<'info, TokenAccount>,
        _token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMFeeCollection> {
        if remaining_accounts.len() < 4 {
            return Err(FlashLeverageError::AccountValidationFailed.into());
        }

        msg!("Collecting Orca Whirlpool fees from position {}", position_account.key());

        // In real implementation, would calculate and collect actual fees
        let fee_a = 1000; // Simulated fee amount
        let fee_b = 2000; // Simulated fee amount

        Ok(CLMMFeeCollection {
            fee_token_a: fee_a,
            fee_token_b: fee_b,
        })
    }

    // Raydium CLMM implementations
    fn increase_raydium_clmm_liquidity<'info>(
        token_a_amount: u64,
        token_b_amount: u64,
        tick_lower: i32,
        tick_upper: i32,
        clmm_program: &UncheckedAccount<'info>,
        _pool_account: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        _user_token_a_account: &Account<'info, TokenAccount>,
        _user_token_b_account: &Account<'info, TokenAccount>,
        _token_a_mint: &Account<'info, Mint>,
        _token_b_mint: &Account<'info, Mint>,
        _token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMLiquidityResult> {
        // Validate Raydium CLMM program ID
        let raydium_clmm_program_id = Pubkey::from_str("CAMMCzo5YL8w4VFF8KVHrK22GGUQpMDdHZe1ALfMiKNj")
            .map_err(|_| FlashLeverageError::InvalidProgramId)?;

        if clmm_program.key() != raydium_clmm_program_id {
            return Err(FlashLeverageError::ProgramAccountMismatch.into());
        }

        Self::validate_tick_range(tick_lower, tick_upper)?;

        if remaining_accounts.len() < 10 {
            return Err(FlashLeverageError::AccountValidationFailed.into());
        }

        let liquidity_amount = Self::calculate_liquidity_for_amounts(
            token_a_amount,
            token_b_amount,
            tick_lower,
            tick_upper,
        )?;

        msg!("Increasing Raydium CLMM liquidity: {} A, {} B", token_a_amount, token_b_amount);

        // Raydium-specific implementation would go here

        Ok(CLMMLiquidityResult {
            liquidity_amount,
            token_a_amount,
            token_b_amount,
            position_nft: Some(position_account.key()),
        })
    }

    fn decrease_raydium_clmm_liquidity<'info>(
        liquidity_amount: u128,
        _clmm_program: &UncheckedAccount<'info>,
        _pool_account: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        _user_token_a_account: &Account<'info, TokenAccount>,
        _user_token_b_account: &Account<'info, TokenAccount>,
        _token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMLiquidityResult> {
        if remaining_accounts.len() < 8 {
            return Err(FlashLeverageError::AccountValidationFailed.into());
        }

        let (token_a_amount, token_b_amount) = Self::calculate_tokens_for_liquidity(
            liquidity_amount,
            remaining_accounts,
        )?;

        msg!("Decreasing Raydium CLMM liquidity: {} units", liquidity_amount);

        Ok(CLMMLiquidityResult {
            liquidity_amount,
            token_a_amount,
            token_b_amount,
            position_nft: Some(position_account.key()),
        })
    }

    fn collect_raydium_clmm_fees<'info>(
        _clmm_program: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        _user_token_a_account: &Account<'info, TokenAccount>,
        _user_token_b_account: &Account<'info, TokenAccount>,
        _token_program: &Program<'info, Token>,
        _remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMFeeCollection> {
        msg!("Collecting Raydium CLMM fees from position {}", position_account.key());

        let fee_a = 1500;
        let fee_b = 2500;

        Ok(CLMMFeeCollection {
            fee_token_a: fee_a,
            fee_token_b: fee_b,
        })
    }

    // Meteora DLMM implementations
    fn increase_meteora_dlmm_liquidity<'info>(
        token_a_amount: u64,
        token_b_amount: u64,
        tick_lower: i32,
        tick_upper: i32,
        clmm_program: &UncheckedAccount<'info>,
        _pool_account: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        _user_token_a_account: &Account<'info, TokenAccount>,
        _user_token_b_account: &Account<'info, TokenAccount>,
        _token_a_mint: &Account<'info, Mint>,
        _token_b_mint: &Account<'info, Mint>,
        _token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMLiquidityResult> {
        // Validate Meteora DLMM program ID
        let meteora_program_id = Pubkey::from_str("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo")
            .map_err(|_| FlashLeverageError::InvalidProgramId)?;

        if clmm_program.key() != meteora_program_id {
            return Err(FlashLeverageError::ProgramAccountMismatch.into());
        }

        Self::validate_tick_range(tick_lower, tick_upper)?;

        if remaining_accounts.len() < 12 {
            return Err(FlashLeverageError::AccountValidationFailed.into());
        }

        let liquidity_amount = Self::calculate_liquidity_for_amounts(
            token_a_amount,
            token_b_amount,
            tick_lower,
            tick_upper,
        )?;

        msg!("Increasing Meteora DLMM liquidity: {} A, {} B", token_a_amount, token_b_amount);

        // Meteora-specific implementation would go here

        Ok(CLMMLiquidityResult {
            liquidity_amount,
            token_a_amount,
            token_b_amount,
            position_nft: Some(position_account.key()),
        })
    }

    fn decrease_meteora_dlmm_liquidity<'info>(
        liquidity_amount: u128,
        _clmm_program: &UncheckedAccount<'info>,
        _pool_account: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        _user_token_a_account: &Account<'info, TokenAccount>,
        _user_token_b_account: &Account<'info, TokenAccount>,
        _token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMLiquidityResult> {
        if remaining_accounts.len() < 10 {
            return Err(FlashLeverageError::AccountValidationFailed.into());
        }

        let (token_a_amount, token_b_amount) = Self::calculate_tokens_for_liquidity(
            liquidity_amount,
            remaining_accounts,
        )?;

        msg!("Decreasing Meteora DLMM liquidity: {} units", liquidity_amount);

        Ok(CLMMLiquidityResult {
            liquidity_amount,
            token_a_amount,
            token_b_amount,
            position_nft: Some(position_account.key()),
        })
    }

    fn collect_meteora_dlmm_fees<'info>(
        _clmm_program: &UncheckedAccount<'info>,
        position_account: &UncheckedAccount<'info>,
        _user_token_a_account: &Account<'info, TokenAccount>,
        _user_token_b_account: &Account<'info, TokenAccount>,
        _token_program: &Program<'info, Token>,
        _remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<CLMMFeeCollection> {
        msg!("Collecting Meteora DLMM fees from position {}", position_account.key());

        let fee_a = 800;
        let fee_b = 1200;

        Ok(CLMMFeeCollection {
            fee_token_a: fee_a,
            fee_token_b: fee_b,
        })
    }

    // Helper functions

    fn validate_tick_range(tick_lower: i32, tick_upper: i32) -> Result<()> {
        if tick_lower >= tick_upper {
            return Err(FlashLeverageError::InvalidTickRange.into());
        }

        // Check tick spacing (simplified - real implementation would check pool-specific spacing)
        let tick_spacing = 64; // Common tick spacing
        if tick_lower % tick_spacing != 0 || tick_upper % tick_spacing != 0 {
            return Err(FlashLeverageError::InvalidTickRange.into());
        }

        // Check maximum tick range
        let max_tick = 443636; // Maximum tick for most pools
        if tick_lower < -max_tick || tick_upper > max_tick {
            return Err(FlashLeverageError::InvalidTickRange.into());
        }

        // Check range width
        let range_width = tick_upper - tick_lower;
        if range_width < tick_spacing * 2 {
            return Err(FlashLeverageError::TickRangeTooNarrow.into());
        }

        if range_width > max_tick {
            return Err(FlashLeverageError::TickRangeTooWide.into());
        }

        Ok(())
    }

    fn calculate_liquidity_for_amounts(
        token_a_amount: u64,
        token_b_amount: u64,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<u128> {
        // Simplified liquidity calculation
        // Real implementation would use complex CLMM math with sqrt price calculations
        
        let tick_range = (tick_upper - tick_lower) as u64;
        let base_liquidity = token_a_amount
            .checked_add(token_b_amount)
            .ok_or(FlashLeverageError::MathOverflow)?;
            
        let liquidity = (base_liquidity as u128)
            .checked_mul(1_000_000)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(tick_range as u128)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        Ok(liquidity)
    }

    fn calculate_tokens_for_liquidity(
        liquidity_amount: u128,
        _remaining_accounts: &[AccountInfo],
    ) -> Result<(u64, u64)> {
        // Simplified calculation - real implementation would read pool state
        // and calculate exact token amounts based on current price and tick range
        
        let total_value = liquidity_amount
            .checked_div(1_000_000)
            .ok_or(FlashLeverageError::DivisionByZero)? as u64;
            
        let token_a_amount = total_value / 2;
        let token_b_amount = total_value - token_a_amount;

        Ok((token_a_amount, token_b_amount))
    }

    fn build_orca_increase_liquidity_data(
        liquidity_amount: u128,
        token_a_max: u64,
        token_b_max: u64,
    ) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        
        // Orca increase liquidity instruction discriminator
        data.extend_from_slice(&[0x2e, 0x54, 0xea, 0x5d, 0x61, 0x30, 0xa6, 0x59]);
        
        // Liquidity amount (16 bytes)
        data.extend_from_slice(&liquidity_amount.to_le_bytes());
        
        // Token A max amount (8 bytes)
        data.extend_from_slice(&token_a_max.to_le_bytes());
        
        // Token B max amount (8 bytes)
        data.extend_from_slice(&token_b_max.to_le_bytes());
        
        Ok(data)
    }

    /// Calculate optimal tick range for a given price range
    pub fn calculate_optimal_ticks(
        price_lower: u64,    // Lower price (scaled by 1e6)
        price_upper: u64,    // Upper price (scaled by 1e6)
        tick_spacing: i32,   // Tick spacing for the pool
    ) -> Result<(i32, i32)> {
        if price_lower >= price_upper {
            return Err(FlashLeverageError::InvalidPriceRange.into());
        }

        // Convert prices to ticks (simplified calculation)
        // Real implementation would use: tick = log_1.0001(price)
        let tick_lower = Self::price_to_tick(price_lower)?;
        let tick_upper = Self::price_to_tick(price_upper)?;

        // Round to nearest valid tick spacing
        let aligned_tick_lower = (tick_lower / tick_spacing) * tick_spacing;
        let aligned_tick_upper = ((tick_upper + tick_spacing - 1) / tick_spacing) * tick_spacing;

        Ok((aligned_tick_lower, aligned_tick_upper))
    }

    fn price_to_tick(price: u64) -> Result<i32> {
        // Simplified price to tick conversion
        // Real implementation would use logarithmic calculation
        if price == 0 {
            return Err(FlashLeverageError::InvalidOraclePrice.into());
        }

        // Approximate conversion: tick ≈ ln(price) / ln(1.0001)
        let normalized_price = price as f64 / 1_000_000.0;
        let tick = (normalized_price.ln() / 1.0001_f64.ln()) as i32;

        Ok(tick)
    }

    /// Estimate impermanent loss for a CLMM position
    pub fn estimate_impermanent_loss(
        initial_price: u64,
        current_price: u64,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<ImpermanentLossEstimate> {
        let price_change_ratio = current_price
            .checked_mul(1_000_000)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(initial_price)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        // Check if current price is still in range
        let current_tick = Self::price_to_tick(current_price)?;
        let in_range = current_tick >= tick_lower && current_tick <= tick_upper;

        // Simplified IL calculation
        let il_percentage = if in_range {
            // IL is lower for concentrated positions in range
            let base_il = if price_change_ratio > 1_000_000 {
                // Price increased
                (price_change_ratio - 1_000_000) / 10 // Reduced IL due to concentration
            } else {
                // Price decreased
                (1_000_000 - price_change_ratio) / 10
            };
            base_il
        } else {
            // Higher IL when out of range (single-sided exposure)
            if price_change_ratio > 1_000_000 {
                price_change_ratio - 1_000_000 // Full price impact
            } else {
                1_000_000 - price_change_ratio
            }
        };

        Ok(ImpermanentLossEstimate {
            il_percentage,
            in_range,
            price_change_ratio,
            estimated_loss_usd: 0, // Would calculate based on position size
        })
    }
}

#[derive(Clone, Debug)]
pub struct CLMMLiquidityResult {
    pub liquidity_amount: u128,
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub position_nft: Option<Pubkey>,
}

#[derive(Clone, Debug)]
pub struct CLMMFeeCollection {
    pub fee_token_a: u64,
    pub fee_token_b: u64,
}

#[derive(Clone, Debug)]
pub struct ImpermanentLossEstimate {
    pub il_percentage: u64,      // Scaled by 1e6
    pub in_range: bool,
    pub price_change_ratio: u64, // Scaled by 1e6
    pub estimated_loss_usd: u64, // Scaled by 1e6
}