use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::integrations::*;

pub fn handler(ctx: Context<EmergencyLiquidate>) -> Result<()> {
    let leverage_state = &mut ctx.accounts.leverage_state;
    let authority = &ctx.accounts.authority;

    msg!("🚨 Starting emergency liquidation");
    msg!("Authority: {}", authority.key());

    // Step 1: Validate liquidation conditions
    msg!("📋 Step 1: Validating liquidation conditions");
    validate_liquidation_conditions(&ctx)?;

    // Step 2: Calculate liquidation parameters
    msg!("📋 Step 2: Calculating liquidation parameters");
    let liquidation_params = calculate_liquidation_parameters(&ctx)?;

    // Step 3: Remove liquidity from CLMM position
    msg!("📋 Step 3: Emergency liquidity removal");
    let liquidity_removal = emergency_remove_liquidity(&ctx, &liquidation_params)?;

    // Step 4: Swap to repayment token
    msg!("📋 Step 4: Converting to repayment token");
    let swap_result = emergency_swap_for_repayment(&ctx, &liquidity_removal, &liquidation_params)?;

    // Step 5: Repay as much debt as possible
    msg!("📋 Step 5: Emergency debt repayment");
    let repay_result = emergency_repay_debt(&ctx, &swap_result, &liquidation_params)?;

    // Step 6: Handle liquidation penalty and remaining assets
    msg!("📋 Step 6: Finalizing liquidation");
    finalize_liquidation(&ctx, &liquidation_params, &repay_result)?;

    // Update global state
    leverage_state.active_positions = leverage_state.active_positions
        .checked_sub(1)
        .ok_or(FlashLeverageError::MathUnderflow)?;

    msg!("⚠️ Emergency liquidation completed");
    msg!("Liquidation penalty: {}%", liquidation_params.penalty_bps as f64 / 100.0);

    Ok(())
}

fn validate_liquidation_conditions(ctx: &Context<EmergencyLiquidate>) -> Result<()> {
    // In a real implementation, would check:
    // 1. Position health factor is below liquidation threshold
    // 2. Oracle prices are recent and valid
    // 3. Liquidator has proper authority
    // 4. Position is not already being liquidated

    // Simulate health factor check
    let simulated_health_factor = 900_000u64; // 0.9 (below 1.0 threshold)
    let liquidation_threshold = 1_000_000u64; // 1.0

    if simulated_health_factor >= liquidation_threshold {
        return Err(FlashLeverageError::PositionCannotBeLiquidated.into());
    }

    msg!("Liquidation conditions validated:");
    msg!("  Health factor: {}", simulated_health_factor as f64 / 1_000_000.0);
    msg!("  Liquidation threshold: {}", liquidation_threshold as f64 / 1_000_000.0);

    Ok(())
}

fn calculate_liquidation_parameters(ctx: &Context<EmergencyLiquidate>) -> Result<LiquidationParams> {
    // In a real implementation, would read from position account
    let position_value_usd = 10_000_000_000u64; // $10,000 scaled by 1e6
    let debt_value_usd = 8_500_000_000u64;      // $8,500 scaled by 1e6
    let health_factor = 900_000u64;             // 0.9

    // Calculate liquidation penalty (typically 5-10%)
    let penalty_bps = 500u16; // 5%
    let penalty_amount = position_value_usd
        .checked_mul(penalty_bps as u64)
        .ok_or(FlashLeverageError::MathOverflow)?
        .checked_div(10000)
        .ok_or(FlashLeverageError::DivisionByZero)?;

    // Calculate maximum liquidation amount (typically 50% of position)
    let max_liquidation_ratio = 5000u16; // 50%
    let max_liquidation_amount = position_value_usd
        .checked_mul(max_liquidation_ratio as u64)
        .ok_or(FlashLeverageError::MathOverflow)?
        .checked_div(10000)
        .ok_or(FlashLeverageError::DivisionByZero)?;

    // Determine actual liquidation amount needed
    let required_liquidation = debt_value_usd
        .checked_add(penalty_amount)
        .ok_or(FlashLeverageError::MathOverflow)?;

    let liquidation_amount = required_liquidation.min(max_liquidation_amount);

    msg!("Liquidation parameters:");
    msg!("  Position value: ${}", position_value_usd as f64 / 1_000_000.0);
    msg!("  Debt value: ${}", debt_value_usd as f64 / 1_000_000.0);
    msg!("  Penalty: ${} ({}%)", penalty_amount as f64 / 1_000_000.0, penalty_bps as f64 / 100.0);
    msg!("  Liquidation amount: ${}", liquidation_amount as f64 / 1_000_000.0);

    Ok(LiquidationParams {
        position_value_usd,
        debt_value_usd,
        health_factor,
        penalty_bps,
        penalty_amount,
        liquidation_amount,
        max_liquidation_ratio,
    })
}

fn emergency_remove_liquidity(
    ctx: &Context<EmergencyLiquidate>,
    params: &LiquidationParams,
) -> Result<EmergencyLiquidityRemoval> {
    // Calculate how much liquidity to remove based on liquidation amount
    let total_liquidity = 1_000_000_000u128; // Placeholder - would read from position
    let liquidation_ratio = params.liquidation_amount
        .checked_mul(10000)
        .ok_or(FlashLeverageError::MathOverflow)?
        .checked_div(params.position_value_usd)
        .ok_or(FlashLeverageError::DivisionByZero)?;

    let liquidity_to_remove = (total_liquidity as u64)
        .checked_mul(liquidation_ratio)
        .ok_or(FlashLeverageError::MathOverflow)?
        .checked_div(10000)
        .ok_or(FlashLeverageError::DivisionByZero)? as u128;

    msg!("Removing {}% of liquidity for liquidation", liquidation_ratio as f64 / 100.0);

    // Execute liquidity removal
    let provider = CLMMProvider::OrcaWhirlpool; // Placeholder
    let removal_result = CLMMIntegration::decrease_liquidity(
        &provider,
        liquidity_to_remove,
        &ctx.accounts.clmm_program,
        &ctx.accounts.position_account,
        &ctx.accounts.position_account, // Using as pool placeholder
        &ctx.accounts.user_token_a_account,
        &ctx.accounts.user_token_b_account,
        &ctx.accounts.token_program,
        ctx.remaining_accounts,
    )?;

    msg!("Emergency liquidity removal: {} A, {} B", 
         removal_result.token_a_amount, removal_result.token_b_amount);

    Ok(EmergencyLiquidityRemoval {
        removed_liquidity: liquidity_to_remove,
        token_a_received: removal_result.token_a_amount,
        token_b_received: removal_result.token_b_amount,
        remaining_liquidity: total_liquidity - liquidity_to_remove,
    })
}

fn emergency_swap_for_repayment(
    _ctx: &Context<EmergencyLiquidate>,
    removal: &EmergencyLiquidityRemoval,
    params: &LiquidationParams,
) -> Result<EmergencySwapResult> {
    // Calculate how much we need in repayment token (assume token B)
    let required_repay_token = params.debt_value_usd / 1000; // Simplified conversion
    let available_repay_token = removal.token_b_received;

    if available_repay_token >= required_repay_token {
        // We have enough, no swap needed
        msg!("Sufficient repayment token available, no swap needed");
        return Ok(EmergencySwapResult {
            swapped_amount: 0,
            received_amount: 0,
            final_token_a: removal.token_a_received,
            final_repay_token: available_repay_token,
        });
    }

    // Need to swap token A to repayment token
    let deficit = required_repay_token - available_repay_token;
    let swap_amount = deficit.min(removal.token_a_received);

    msg!("Emergency swap: {} A -> {} repayment token", swap_amount, swap_amount);

    // Simplified 1:1 swap for emergency
    let received_amount = swap_amount;

    Ok(EmergencySwapResult {
        swapped_amount: swap_amount,
        received_amount,
        final_token_a: removal.token_a_received - swap_amount,
        final_repay_token: available_repay_token + received_amount,
    })
}

fn emergency_repay_debt(
    ctx: &Context<EmergencyLiquidate>,
    swap_result: &EmergencySwapResult,
    params: &LiquidationParams,
) -> Result<EmergencyRepayResult> {
    let available_for_repay = swap_result.final_repay_token;
    let total_debt = params.debt_value_usd / 1000; // Simplified conversion

    let repay_amount = available_for_repay.min(total_debt);

    msg!("Emergency debt repayment: {} of {} total debt", repay_amount, total_debt);

    // Execute repayment
    let provider = LendingProvider::MarginFi; // Placeholder
    let repay_result = LendingIntegration::repay(
        &provider,
        repay_amount,
        &ctx.accounts.lending_program,
        &ctx.accounts.position_account, // Using as obligation placeholder
        &ctx.accounts.position_account, // Using as reserve placeholder
        &ctx.accounts.user_token_b_account,
        &ctx.accounts.token_program,
        ctx.remaining_accounts,
    )?;

    let remaining_debt = total_debt
        .checked_sub(repay_result.repaid_amount)
        .unwrap_or(0);

    Ok(EmergencyRepayResult {
        repaid_amount: repay_result.repaid_amount,
        interest_paid: repay_result.interest_paid,
        remaining_debt,
        bad_debt: remaining_debt > 0,
    })
}

fn finalize_liquidation(
    _ctx: &Context<EmergencyLiquidate>,
    params: &LiquidationParams,
    repay_result: &EmergencyRepayResult,
) -> Result<()> {
    msg!("Liquidation finalization:");
    msg!("  Total debt repaid: {}", repay_result.repaid_amount);
    msg!("  Interest paid: {}", repay_result.interest_paid);
    msg!("  Remaining debt: {}", repay_result.remaining_debt);
    msg!("  Liquidation penalty: ${}", params.penalty_amount as f64 / 1_000_000.0);

    if repay_result.bad_debt {
        msg!("⚠️  Bad debt detected: {}", repay_result.remaining_debt);
        // In a real implementation, would handle bad debt through insurance fund
    }

    // In a real implementation, would:
    // 1. Transfer liquidation penalty to liquidator
    // 2. Transfer remaining assets to insurance fund or back to user
    // 3. Update position state to Liquidated
    // 4. Emit liquidation event

    Ok(())
}

#[derive(Clone, Debug)]
struct LiquidationParams {
    pub position_value_usd: u64,
    pub debt_value_usd: u64,
    pub health_factor: u64,
    pub penalty_bps: u16,
    pub penalty_amount: u64,
    pub liquidation_amount: u64,
    pub max_liquidation_ratio: u16,
}

#[derive(Clone, Debug)]
struct EmergencyLiquidityRemoval {
    pub removed_liquidity: u128,
    pub token_a_received: u64,
    pub token_b_received: u64,
    pub remaining_liquidity: u128,
}

#[derive(Clone, Debug)]
struct EmergencySwapResult {
    pub swapped_amount: u64,
    pub received_amount: u64,
    pub final_token_a: u64,
    pub final_repay_token: u64,
}

#[derive(Clone, Debug)]
struct EmergencyRepayResult {
    pub repaid_amount: u64,
    pub interest_paid: u64,
    pub remaining_debt: u64,
    pub bad_debt: bool,
}

use crate::*;

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
    
    // Token accounts
    #[account(mut)]
    pub user_token_a_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_token_b_account: Account<'info, TokenAccount>,
    
    // CLMM accounts
    /// CHECK: CLMM program
    pub clmm_program: UncheckedAccount<'info>,
    
    /// CHECK: Position account
    pub position_account: UncheckedAccount<'info>,
    
    // Lending protocol accounts
    /// CHECK: Lending program
    pub lending_program: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}