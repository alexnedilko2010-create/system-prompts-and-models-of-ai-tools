use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;
use crate::errors::*;

/// Traditional lending integration for various protocols
pub struct LendingIntegration;

impl LendingIntegration {
    /// Take a traditional loan from a lending protocol
    pub fn borrow<'info>(
        provider: &LendingProvider,
        borrow_amount: u64,
        collateral_amount: u64,
        lending_program: &UncheckedAccount<'info>,
        lending_market: &UncheckedAccount<'info>,
        reserve_account: &UncheckedAccount<'info>,
        obligation_account: &UncheckedAccount<'info>,
        user_collateral_account: &Account<'info, TokenAccount>,
        user_borrow_account: &Account<'info, TokenAccount>,
        collateral_mint: &Account<'info, Mint>,
        borrow_mint: &Account<'info, Mint>,
        token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<LendingBorrowResult> {
        match provider {
            LendingProvider::MarginFi => {
                Self::borrow_marginfi(
                    borrow_amount,
                    collateral_amount,
                    lending_program,
                    lending_market,
                    reserve_account,
                    obligation_account,
                    user_collateral_account,
                    user_borrow_account,
                    collateral_mint,
                    borrow_mint,
                    token_program,
                    remaining_accounts,
                )
            }
            LendingProvider::Solend => {
                Self::borrow_solend(
                    borrow_amount,
                    collateral_amount,
                    lending_program,
                    lending_market,
                    reserve_account,
                    obligation_account,
                    user_collateral_account,
                    user_borrow_account,
                    collateral_mint,
                    borrow_mint,
                    token_program,
                    remaining_accounts,
                )
            }
            LendingProvider::Kamino => {
                Self::borrow_kamino(
                    borrow_amount,
                    collateral_amount,
                    lending_program,
                    lending_market,
                    reserve_account,
                    obligation_account,
                    user_collateral_account,
                    user_borrow_account,
                    collateral_mint,
                    borrow_mint,
                    token_program,
                    remaining_accounts,
                )
            }
            LendingProvider::Port => {
                Self::borrow_port(
                    borrow_amount,
                    collateral_amount,
                    lending_program,
                    lending_market,
                    reserve_account,
                    obligation_account,
                    user_collateral_account,
                    user_borrow_account,
                    collateral_mint,
                    borrow_mint,
                    token_program,
                    remaining_accounts,
                )
            }
            LendingProvider::Custom(_) => {
                Err(FlashLeverageError::FeatureNotImplemented.into())
            }
        }
    }

    /// Repay a traditional loan
    pub fn repay<'info>(
        provider: &LendingProvider,
        repay_amount: u64,
        lending_program: &UncheckedAccount<'info>,
        obligation_account: &UncheckedAccount<'info>,
        reserve_account: &UncheckedAccount<'info>,
        user_repay_account: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<LendingRepayResult> {
        match provider {
            LendingProvider::MarginFi => {
                Self::repay_marginfi(
                    repay_amount,
                    lending_program,
                    obligation_account,
                    reserve_account,
                    user_repay_account,
                    token_program,
                    remaining_accounts,
                )
            }
            LendingProvider::Solend => {
                Self::repay_solend(
                    repay_amount,
                    lending_program,
                    obligation_account,
                    reserve_account,
                    user_repay_account,
                    token_program,
                    remaining_accounts,
                )
            }
            LendingProvider::Kamino => {
                Self::repay_kamino(
                    repay_amount,
                    lending_program,
                    obligation_account,
                    reserve_account,
                    user_repay_account,
                    token_program,
                    remaining_accounts,
                )
            }
            LendingProvider::Port => {
                Self::repay_port(
                    repay_amount,
                    lending_program,
                    obligation_account,
                    reserve_account,
                    user_repay_account,
                    token_program,
                    remaining_accounts,
                )
            }
            LendingProvider::Custom(_) => {
                Err(FlashLeverageError::FeatureNotImplemented.into())
            }
        }
    }

    /// Calculate health factor for a position
    pub fn calculate_health_factor(
        provider: &LendingProvider,
        collateral_value_usd: u64,
        borrow_value_usd: u64,
        collateral_factor: u16, // In basis points
    ) -> Result<u64> {
        if borrow_value_usd == 0 {
            return Ok(u64::MAX); // Infinite health factor (no debt)
        }

        let adjusted_collateral = collateral_value_usd
            .checked_mul(collateral_factor as u64)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        let health_factor = adjusted_collateral
            .checked_mul(1_000_000) // Scale by 1e6
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(borrow_value_usd)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        // Provider-specific adjustments
        let adjusted_health_factor = match provider {
            LendingProvider::MarginFi => health_factor,
            LendingProvider::Solend => health_factor,
            LendingProvider::Kamino => health_factor,
            LendingProvider::Port => health_factor,
            LendingProvider::Custom(_) => health_factor,
        };

        Ok(adjusted_health_factor)
    }

    /// Get maximum borrowable amount
    pub fn get_max_borrow_amount(
        provider: &LendingProvider,
        collateral_value_usd: u64,
        collateral_factor: u16,
        existing_borrow_usd: u64,
    ) -> Result<u64> {
        let max_borrow_capacity = collateral_value_usd
            .checked_mul(collateral_factor as u64)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        if max_borrow_capacity <= existing_borrow_usd {
            return Ok(0);
        }

        let available_borrow = max_borrow_capacity
            .checked_sub(existing_borrow_usd)
            .ok_or(FlashLeverageError::MathUnderflow)?;

        // Apply provider-specific safety margins
        let safety_margin_bps = match provider {
            LendingProvider::MarginFi => 500,  // 5% safety margin
            LendingProvider::Solend => 300,   // 3% safety margin
            LendingProvider::Kamino => 400,   // 4% safety margin
            LendingProvider::Port => 600,     // 6% safety margin
            LendingProvider::Custom(_) => 500,
        };

        let safe_borrow_amount = available_borrow
            .checked_mul(10000 - safety_margin_bps)
            .ok_or(FlashLeverageError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FlashLeverageError::DivisionByZero)?;

        Ok(safe_borrow_amount)
    }

    // MarginFi implementations
    fn borrow_marginfi<'info>(
        borrow_amount: u64,
        collateral_amount: u64,
        lending_program: &UncheckedAccount<'info>,
        lending_market: &UncheckedAccount<'info>,
        reserve_account: &UncheckedAccount<'info>,
        obligation_account: &UncheckedAccount<'info>,
        user_collateral_account: &Account<'info, TokenAccount>,
        user_borrow_account: &Account<'info, TokenAccount>,
        _collateral_mint: &Account<'info, Mint>,
        _borrow_mint: &Account<'info, Mint>,
        _token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<LendingBorrowResult> {
        // Validate MarginFi program ID
        let marginfi_program_id = Pubkey::from_str("MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA")
            .map_err(|_| FlashLeverageError::InvalidProgramId)?;

        if lending_program.key() != marginfi_program_id {
            return Err(FlashLeverageError::ProgramAccountMismatch.into());
        }

        if remaining_accounts.len() < 8 {
            return Err(FlashLeverageError::AccountValidationFailed.into());
        }

        // Validate collateral ratio
        let health_factor = Self::calculate_health_factor(
            &LendingProvider::MarginFi,
            collateral_amount,
            borrow_amount,
            8000, // 80% collateral factor for MarginFi
        )?;

        if health_factor < 1_200_000 { // 1.2 minimum health factor
            return Err(FlashLeverageError::CollateralRatioInsufficient.into());
        }

        msg!("MarginFi borrow: {} collateral, {} borrow", collateral_amount, borrow_amount);

        // Build MarginFi borrow instruction
        let instruction_data = Self::build_marginfi_borrow_data(
            borrow_amount,
            collateral_amount,
        )?;

        // Execute borrow (would be CPI in real implementation)
        let interest_rate_bps = 500; // 5% APR

        Ok(LendingBorrowResult {
            borrowed_amount: borrow_amount,
            collateral_deposited: collateral_amount,
            interest_rate_bps,
            health_factor,
            obligation_account: obligation_account.key(),
        })
    }

    fn repay_marginfi<'info>(
        repay_amount: u64,
        _lending_program: &UncheckedAccount<'info>,
        obligation_account: &UncheckedAccount<'info>,
        _reserve_account: &UncheckedAccount<'info>,
        _user_repay_account: &Account<'info, TokenAccount>,
        _token_program: &Program<'info, Token>,
        _remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<LendingRepayResult> {
        msg!("MarginFi repay: {} amount", repay_amount);

        // Calculate interest accrued (simplified)
        let interest_accrued = repay_amount / 100; // 1% interest

        Ok(LendingRepayResult {
            repaid_amount: repay_amount,
            interest_paid: interest_accrued,
            remaining_debt: 0, // Assuming full repayment
            collateral_released: 0, // Would calculate based on actual position
        })
    }

    // Solend implementations
    fn borrow_solend<'info>(
        borrow_amount: u64,
        collateral_amount: u64,
        lending_program: &UncheckedAccount<'info>,
        _lending_market: &UncheckedAccount<'info>,
        _reserve_account: &UncheckedAccount<'info>,
        obligation_account: &UncheckedAccount<'info>,
        _user_collateral_account: &Account<'info, TokenAccount>,
        _user_borrow_account: &Account<'info, TokenAccount>,
        _collateral_mint: &Account<'info, Mint>,
        _borrow_mint: &Account<'info, Mint>,
        _token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<LendingBorrowResult> {
        // Validate Solend program ID
        let solend_program_id = Pubkey::from_str("So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo")
            .map_err(|_| FlashLeverageError::InvalidProgramId)?;

        if lending_program.key() != solend_program_id {
            return Err(FlashLeverageError::ProgramAccountMismatch.into());
        }

        if remaining_accounts.len() < 10 {
            return Err(FlashLeverageError::AccountValidationFailed.into());
        }

        let health_factor = Self::calculate_health_factor(
            &LendingProvider::Solend,
            collateral_amount,
            borrow_amount,
            7500, // 75% collateral factor for Solend
        )?;

        if health_factor < 1_250_000 { // 1.25 minimum health factor
            return Err(FlashLeverageError::CollateralRatioInsufficient.into());
        }

        msg!("Solend borrow: {} collateral, {} borrow", collateral_amount, borrow_amount);

        let interest_rate_bps = 600; // 6% APR

        Ok(LendingBorrowResult {
            borrowed_amount: borrow_amount,
            collateral_deposited: collateral_amount,
            interest_rate_bps,
            health_factor,
            obligation_account: obligation_account.key(),
        })
    }

    fn repay_solend<'info>(
        repay_amount: u64,
        _lending_program: &UncheckedAccount<'info>,
        _obligation_account: &UncheckedAccount<'info>,
        _reserve_account: &UncheckedAccount<'info>,
        _user_repay_account: &Account<'info, TokenAccount>,
        _token_program: &Program<'info, Token>,
        _remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<LendingRepayResult> {
        msg!("Solend repay: {} amount", repay_amount);

        let interest_accrued = repay_amount / 80; // 1.25% interest

        Ok(LendingRepayResult {
            repaid_amount: repay_amount,
            interest_paid: interest_accrued,
            remaining_debt: 0,
            collateral_released: 0,
        })
    }

    // Kamino implementations
    fn borrow_kamino<'info>(
        borrow_amount: u64,
        collateral_amount: u64,
        lending_program: &UncheckedAccount<'info>,
        _lending_market: &UncheckedAccount<'info>,
        _reserve_account: &UncheckedAccount<'info>,
        obligation_account: &UncheckedAccount<'info>,
        _user_collateral_account: &Account<'info, TokenAccount>,
        _user_borrow_account: &Account<'info, TokenAccount>,
        _collateral_mint: &Account<'info, Mint>,
        _borrow_mint: &Account<'info, Mint>,
        _token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<LendingBorrowResult> {
        // Validate Kamino program ID
        let kamino_program_id = Pubkey::from_str("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD")
            .map_err(|_| FlashLeverageError::InvalidProgramId)?;

        if lending_program.key() != kamino_program_id {
            return Err(FlashLeverageError::ProgramAccountMismatch.into());
        }

        if remaining_accounts.len() < 12 {
            return Err(FlashLeverageError::AccountValidationFailed.into());
        }

        let health_factor = Self::calculate_health_factor(
            &LendingProvider::Kamino,
            collateral_amount,
            borrow_amount,
            8200, // 82% collateral factor for Kamino
        )?;

        if health_factor < 1_150_000 { // 1.15 minimum health factor
            return Err(FlashLeverageError::CollateralRatioInsufficient.into());
        }

        msg!("Kamino borrow: {} collateral, {} borrow", collateral_amount, borrow_amount);

        let interest_rate_bps = 450; // 4.5% APR

        Ok(LendingBorrowResult {
            borrowed_amount: borrow_amount,
            collateral_deposited: collateral_amount,
            interest_rate_bps,
            health_factor,
            obligation_account: obligation_account.key(),
        })
    }

    fn repay_kamino<'info>(
        repay_amount: u64,
        _lending_program: &UncheckedAccount<'info>,
        _obligation_account: &UncheckedAccount<'info>,
        _reserve_account: &UncheckedAccount<'info>,
        _user_repay_account: &Account<'info, TokenAccount>,
        _token_program: &Program<'info, Token>,
        _remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<LendingRepayResult> {
        msg!("Kamino repay: {} amount", repay_amount);

        let interest_accrued = repay_amount / 120; // 0.83% interest

        Ok(LendingRepayResult {
            repaid_amount: repay_amount,
            interest_paid: interest_accrued,
            remaining_debt: 0,
            collateral_released: 0,
        })
    }

    // Port Finance implementations
    fn borrow_port<'info>(
        borrow_amount: u64,
        collateral_amount: u64,
        lending_program: &UncheckedAccount<'info>,
        _lending_market: &UncheckedAccount<'info>,
        _reserve_account: &UncheckedAccount<'info>,
        obligation_account: &UncheckedAccount<'info>,
        _user_collateral_account: &Account<'info, TokenAccount>,
        _user_borrow_account: &Account<'info, TokenAccount>,
        _collateral_mint: &Account<'info, Mint>,
        _borrow_mint: &Account<'info, Mint>,
        _token_program: &Program<'info, Token>,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<LendingBorrowResult> {
        // Validate Port Finance program ID
        let port_program_id = Pubkey::from_str("Port7uDYB3wk6GJAw4KT1WpTeMtSu9bTcChBHkX2LfR")
            .map_err(|_| FlashLeverageError::InvalidProgramId)?;

        if lending_program.key() != port_program_id {
            return Err(FlashLeverageError::ProgramAccountMismatch.into());
        }

        if remaining_accounts.len() < 9 {
            return Err(FlashLeverageError::AccountValidationFailed.into());
        }

        let health_factor = Self::calculate_health_factor(
            &LendingProvider::Port,
            collateral_amount,
            borrow_amount,
            7000, // 70% collateral factor for Port Finance
        )?;

        if health_factor < 1_300_000 { // 1.3 minimum health factor
            return Err(FlashLeverageError::CollateralRatioInsufficient.into());
        }

        msg!("Port Finance borrow: {} collateral, {} borrow", collateral_amount, borrow_amount);

        let interest_rate_bps = 700; // 7% APR

        Ok(LendingBorrowResult {
            borrowed_amount: borrow_amount,
            collateral_deposited: collateral_amount,
            interest_rate_bps,
            health_factor,
            obligation_account: obligation_account.key(),
        })
    }

    fn repay_port<'info>(
        repay_amount: u64,
        _lending_program: &UncheckedAccount<'info>,
        _obligation_account: &UncheckedAccount<'info>,
        _reserve_account: &UncheckedAccount<'info>,
        _user_repay_account: &Account<'info, TokenAccount>,
        _token_program: &Program<'info, Token>,
        _remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<LendingRepayResult> {
        msg!("Port Finance repay: {} amount", repay_amount);

        let interest_accrued = repay_amount / 70; // 1.43% interest

        Ok(LendingRepayResult {
            repaid_amount: repay_amount,
            interest_paid: interest_accrued,
            remaining_debt: 0,
            collateral_released: 0,
        })
    }

    // Helper functions

    fn build_marginfi_borrow_data(
        borrow_amount: u64,
        collateral_amount: u64,
    ) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        
        // MarginFi borrow instruction discriminator
        data.extend_from_slice(&[0x5d, 0x36, 0xc7, 0x8f, 0x23, 0x1c, 0x4b, 0x92]);
        
        // Collateral amount (8 bytes)
        data.extend_from_slice(&collateral_amount.to_le_bytes());
        
        // Borrow amount (8 bytes)
        data.extend_from_slice(&borrow_amount.to_le_bytes());
        
        Ok(data)
    }

    /// Validate lending parameters
    pub fn validate_lending_params(
        provider: &LendingProvider,
        borrow_amount: u64,
        collateral_amount: u64,
        collateral_mint: &Pubkey,
        borrow_mint: &Pubkey,
    ) -> Result<()> {
        // Basic validations
        if borrow_amount == 0 {
            return Err(FlashLeverageError::InsufficientBalance.into());
        }

        if collateral_amount == 0 {
            return Err(FlashLeverageError::InsufficientBalance.into());
        }

        // Provider-specific minimums
        let min_borrow_amount = match provider {
            LendingProvider::MarginFi => 1_000_000,    // 1 token minimum
            LendingProvider::Solend => 500_000,       // 0.5 token minimum
            LendingProvider::Kamino => 2_000_000,     // 2 token minimum
            LendingProvider::Port => 1_500_000,       // 1.5 token minimum
            LendingProvider::Custom(_) => 1_000_000,
        };

        if borrow_amount < min_borrow_amount {
            return Err(FlashLeverageError::PositionSizeTooSmall.into());
        }

        // Validate mints
        if collateral_mint == &Pubkey::default() || borrow_mint == &Pubkey::default() {
            return Err(FlashLeverageError::InvalidTokenMint.into());
        }

        Ok(())
    }

    /// Get lending protocol parameters
    pub fn get_protocol_params(provider: &LendingProvider) -> ProtocolParams {
        match provider {
            LendingProvider::MarginFi => ProtocolParams {
                max_ltv_bps: 8000,           // 80%
                liquidation_threshold_bps: 8500, // 85%
                liquidation_penalty_bps: 500,    // 5%
                min_health_factor: 1_200_000,    // 1.2
                base_interest_rate_bps: 200,     // 2%
                max_interest_rate_bps: 3000,     // 30%
                utilization_kink_bps: 8000,      // 80%
            },
            LendingProvider::Solend => ProtocolParams {
                max_ltv_bps: 7500,
                liquidation_threshold_bps: 8000,
                liquidation_penalty_bps: 300,
                min_health_factor: 1_250_000,
                base_interest_rate_bps: 150,
                max_interest_rate_bps: 2500,
                utilization_kink_bps: 7500,
            },
            LendingProvider::Kamino => ProtocolParams {
                max_ltv_bps: 8200,
                liquidation_threshold_bps: 8700,
                liquidation_penalty_bps: 400,
                min_health_factor: 1_150_000,
                base_interest_rate_bps: 100,
                max_interest_rate_bps: 2000,
                utilization_kink_bps: 8500,
            },
            LendingProvider::Port => ProtocolParams {
                max_ltv_bps: 7000,
                liquidation_threshold_bps: 7500,
                liquidation_penalty_bps: 600,
                min_health_factor: 1_300_000,
                base_interest_rate_bps: 250,
                max_interest_rate_bps: 3500,
                utilization_kink_bps: 7000,
            },
            LendingProvider::Custom(_) => ProtocolParams {
                max_ltv_bps: 7500,
                liquidation_threshold_bps: 8000,
                liquidation_penalty_bps: 500,
                min_health_factor: 1_200_000,
                base_interest_rate_bps: 200,
                max_interest_rate_bps: 2500,
                utilization_kink_bps: 7500,
            },
        }
    }

    /// Calculate interest rate based on utilization
    pub fn calculate_interest_rate(
        provider: &LendingProvider,
        utilization_bps: u16, // Utilization in basis points
    ) -> Result<u16> {
        let params = Self::get_protocol_params(provider);
        
        let interest_rate = if utilization_bps <= params.utilization_kink_bps {
            // Below kink: linear interpolation
            params.base_interest_rate_bps + 
                ((utilization_bps as u64 * (params.max_interest_rate_bps - params.base_interest_rate_bps) as u64) / 
                 params.utilization_kink_bps as u64) as u16
        } else {
            // Above kink: exponential increase
            let excess_utilization = utilization_bps - params.utilization_kink_bps;
            let excess_rate = (excess_utilization as u64 * params.max_interest_rate_bps as u64) / 
                             (10000 - params.utilization_kink_bps as u64);
            
            (params.max_interest_rate_bps as u64 + excess_rate).min(params.max_interest_rate_bps as u64) as u16
        };

        Ok(interest_rate)
    }
}

#[derive(Clone, Debug)]
pub struct LendingBorrowResult {
    pub borrowed_amount: u64,
    pub collateral_deposited: u64,
    pub interest_rate_bps: u16,
    pub health_factor: u64,
    pub obligation_account: Pubkey,
}

#[derive(Clone, Debug)]
pub struct LendingRepayResult {
    pub repaid_amount: u64,
    pub interest_paid: u64,
    pub remaining_debt: u64,
    pub collateral_released: u64,
}

#[derive(Clone, Debug)]
pub struct ProtocolParams {
    pub max_ltv_bps: u16,                // Maximum loan-to-value ratio
    pub liquidation_threshold_bps: u16,   // Liquidation threshold
    pub liquidation_penalty_bps: u16,     // Liquidation penalty
    pub min_health_factor: u64,          // Minimum health factor (scaled by 1e6)
    pub base_interest_rate_bps: u16,     // Base interest rate
    pub max_interest_rate_bps: u16,      // Maximum interest rate
    pub utilization_kink_bps: u16,       // Utilization kink point
}