use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use solana_program::program_pack::Pack;

declare_id!("FLashBoostrap11111111111111111111111111111");

pub mod instructions;
pub mod state;
pub mod errors;
pub mod utils;
pub mod integrations;
pub mod strategies;

use instructions::*;
use state::*;
use errors::*;

#[program]
pub mod flash_bootstrapped_leverage {
    use super::*;

    /// Initialize the flash bootstrapped leverage program
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    /// Execute the complete flash-bootstrapped leverage strategy
    /// 1. Flash borrow token B
    /// 2. Swap via Jupiter to achieve A/B ratio
    /// 3. Increase liquidity in CLMM
    /// 4. Take traditional loan in lending protocol
    /// 5. Repay flash loan
    pub fn execute_flash_leverage(
        ctx: Context<ExecuteFlashLeverage>,
        params: FlashLeverageParams,
    ) -> Result<()> {
        instructions::execute_flash_leverage::handler(ctx, params)
    }

    /// Close position and repay traditional loan
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        instructions::close_position::handler(ctx)
    }

    /// Emergency function to liquidate position if needed
    pub fn emergency_liquidate(ctx: Context<EmergencyLiquidate>) -> Result<()> {
        instructions::emergency_liquidate::handler(ctx)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + LeverageState::INIT_SPACE,
        seeds = [b"leverage_state"],
        bump
    )]
    pub leverage_state: Account<'info, LeverageState>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteFlashLeverage<'info> {
    #[account(
        mut,
        seeds = [b"leverage_state"],
        bump = leverage_state.bump
    )]
    pub leverage_state: Account<'info, LeverageState>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    // Token accounts
    #[account(mut)]
    pub user_token_a_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_token_b_account: Account<'info, TokenAccount>,
    
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,
    
    // Flash loan provider accounts (will be dynamically determined)
    /// CHECK: Flash loan provider program
    pub flash_loan_program: UncheckedAccount<'info>,
    
    // Jupiter swap accounts
    /// CHECK: Jupiter program
    pub jupiter_program: UncheckedAccount<'info>,
    
    // CLMM accounts (Orca/Raydium/Meteora)
    /// CHECK: CLMM program
    pub clmm_program: UncheckedAccount<'info>,
    
    /// CHECK: Pool account
    pub pool_account: UncheckedAccount<'info>,
    
    /// CHECK: Position account
    pub position_account: UncheckedAccount<'info>,
    
    // Lending protocol accounts
    /// CHECK: Lending program
    pub lending_program: UncheckedAccount<'info>,
    
    /// CHECK: Lending market
    pub lending_market: UncheckedAccount<'info>,
    
    /// CHECK: Reserve account
    pub reserve_account: UncheckedAccount<'info>,
    
    // Standard programs
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(
        mut,
        seeds = [b"leverage_state"],
        bump = leverage_state.bump
    )]
    pub leverage_state: Account<'info, LeverageState>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    // Similar accounts as ExecuteFlashLeverage but for closing
    #[account(mut)]
    pub user_token_a_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_token_b_account: Account<'info, TokenAccount>,
    
    /// CHECK: CLMM program
    pub clmm_program: UncheckedAccount<'info>,
    
    /// CHECK: Position account
    pub position_account: UncheckedAccount<'info>,
    
    /// CHECK: Lending program
    pub lending_program: UncheckedAccount<'info>,
    
    /// CHECK: Lending obligation
    pub lending_obligation: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EmergencyLiquidate<'info> {
    #[account(
        mut,
        seeds = [b"leverage_state"],
        bump = leverage_state.bump
    )]
    pub leverage_state: Account<'info, LeverageState>,
    
    /// CHECK: Emergency authority (could be user or liquidator)
    pub authority: Signer<'info>,
    
    // Emergency liquidation accounts
    #[account(mut)]
    pub user_token_a_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_token_b_account: Account<'info, TokenAccount>,
    
    /// CHECK: CLMM program
    pub clmm_program: UncheckedAccount<'info>,
    
    /// CHECK: Position account
    pub position_account: UncheckedAccount<'info>,
    
    /// CHECK: Lending program
    pub lending_program: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}