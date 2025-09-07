use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let leverage_state = &mut ctx.accounts.leverage_state;
    let authority = &ctx.accounts.authority;

    msg!("Initializing Flash Bootstrapped Leverage Program");
    msg!("Authority: {}", authority.key());

    // Initialize the leverage state
    leverage_state.authority = authority.key();
    leverage_state.bump = ctx.bumps.leverage_state;
    leverage_state.active_positions = 0;
    leverage_state.config = LeverageConfig::default();

    msg!("Program initialized successfully");
    msg!("Configuration:");
    msg!("  Max leverage: {}x", leverage_state.config.max_leverage as f64 / 100.0);
    msg!("  Max slippage: {}%", leverage_state.config.max_slippage_bps as f64 / 100.0);
    msg!("  Liquidation threshold: {}%", leverage_state.config.liquidation_threshold_bps as f64 / 100.0);
    msg!("  Protocol fee: {}%", leverage_state.config.protocol_fee_bps as f64 / 100.0);
    msg!("  Min position size: ${}", leverage_state.config.min_position_size as f64 / 1_000_000.0);
    msg!("  Max position size: ${}", leverage_state.config.max_position_size as f64 / 1_000_000.0);

    Ok(())
}

use crate::*;

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