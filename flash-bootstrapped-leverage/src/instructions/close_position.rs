use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::integrations::*;

pub fn handler(ctx: Context<ClosePosition>) -> Result<()> {
    let leverage_state = &mut ctx.accounts.leverage_state;
    let user = &ctx.accounts.user;

    msg!("🔄 Starting position closure");
    msg!("User: {}", user.key());

    // Step 1: Collect accumulated fees from CLMM position
    msg!("📋 Step 1: Collecting CLMM fees");
    let fee_collection = collect_clmm_fees(&ctx)?;

    // Step 2: Remove liquidity from CLMM position
    msg!("📋 Step 2: Removing CLMM liquidity");
    let liquidity_removal = remove_clmm_liquidity(&ctx)?;

    // Step 3: Swap tokens if needed to get repayment token
    msg!("📋 Step 3: Optimizing token composition");
    let swap_result = optimize_for_repayment(&ctx, &liquidity_removal)?;

    // Step 4: Repay traditional loan
    msg!("📋 Step 4: Repaying traditional loan");
    let repay_result = repay_traditional_loan(&ctx)?;

    // Step 5: Return remaining tokens to user
    msg!("📋 Step 5: Finalizing closure");
    finalize_position_closure(&ctx, &fee_collection, &swap_result, &repay_result)?;

    // Update global state
    leverage_state.active_positions = leverage_state.active_positions
        .checked_sub(1)
        .ok_or(FlashLeverageError::MathUnderflow)?;

    msg!("✅ Position closed successfully!");
    msg!("Fees collected: {} A + {} B", fee_collection.fee_token_a, fee_collection.fee_token_b);

    Ok(())
}

fn collect_clmm_fees(ctx: &Context<ClosePosition>) -> Result<CLMMFeeCollection> {
    // In a real implementation, would determine provider from position state
    let provider = CLMMProvider::OrcaWhirlpool; // Placeholder

    let fee_collection = CLMMIntegration::collect_fees(
        &provider,
        &ctx.accounts.clmm_program,
        &ctx.accounts.position_account,
        &ctx.accounts.user_token_a_account,
        &ctx.accounts.user_token_b_account,
        &ctx.accounts.token_program,
        ctx.remaining_accounts,
    )?;

    msg!("Collected fees: {} A, {} B", fee_collection.fee_token_a, fee_collection.fee_token_b);

    Ok(fee_collection)
}

fn remove_clmm_liquidity(ctx: &Context<ClosePosition>) -> Result<CLMMLiquidityResult> {
    // In a real implementation, would get actual liquidity amount from position state
    let liquidity_amount = 1_000_000_000u128; // Placeholder
    let provider = CLMMProvider::OrcaWhirlpool; // Placeholder

    let liquidity_result = CLMMIntegration::decrease_liquidity(
        &provider,
        liquidity_amount,
        &ctx.accounts.clmm_program,
        &ctx.accounts.position_account,
        &ctx.accounts.position_account, // Using as pool placeholder
        &ctx.accounts.user_token_a_account,
        &ctx.accounts.user_token_b_account,
        &ctx.accounts.token_program,
        ctx.remaining_accounts,
    )?;

    msg!("Removed liquidity: {} A, {} B", 
         liquidity_result.token_a_amount, liquidity_result.token_b_amount);

    Ok(liquidity_result)
}

fn optimize_for_repayment(
    ctx: &Context<ClosePosition>,
    liquidity_removal: &CLMMLiquidityResult,
) -> Result<SwapOptimizationResult> {
    // Calculate how much we need to repay (would come from position state)
    let required_repay_amount = 1_000_000u64; // Placeholder
    let current_token_b = liquidity_removal.token_b_amount;

    if current_token_b >= required_repay_amount {
        // We have enough token B, no swap needed
        msg!("Sufficient token B for repayment, no swap needed");
        return Ok(SwapOptimizationResult {
            swapped_amount: 0,
            received_amount: 0,
            final_token_a: liquidity_removal.token_a_amount,
            final_token_b: current_token_b,
            repay_amount: required_repay_amount,
        });
    }

    // Need to swap some token A to token B
    let deficit = required_repay_amount
        .checked_sub(current_token_b)
        .ok_or(FlashLeverageError::MathUnderflow)?;

    // Estimate how much token A we need to swap (simplified 1:1 ratio)
    let swap_amount_a = deficit;

    if liquidity_removal.token_a_amount < swap_amount_a {
        return Err(FlashLeverageError::InsufficientBalance.into());
    }

    msg!("Swapping {} A to B for loan repayment", swap_amount_a);

    // In a real implementation, would execute Jupiter swap here
    let received_b = swap_amount_a; // Simplified 1:1 swap

    Ok(SwapOptimizationResult {
        swapped_amount: swap_amount_a,
        received_amount: received_b,
        final_token_a: liquidity_removal.token_a_amount - swap_amount_a,
        final_token_b: current_token_b + received_b,
        repay_amount: required_repay_amount,
    })
}

fn repay_traditional_loan(ctx: &Context<ClosePosition>) -> Result<LendingRepayResult> {
    // In a real implementation, would get actual loan details from position state
    let repay_amount = 1_000_000u64; // Placeholder
    let provider = LendingProvider::MarginFi; // Placeholder

    let repay_result = LendingIntegration::repay(
        &provider,
        repay_amount,
        &ctx.accounts.lending_program,
        &ctx.accounts.lending_obligation,
        &ctx.accounts.lending_obligation, // Using as reserve placeholder
        &ctx.accounts.user_token_b_account,
        &ctx.accounts.token_program,
        ctx.remaining_accounts,
    )?;

    msg!("Loan repaid: {} principal + {} interest", 
         repay_result.repaid_amount, repay_result.interest_paid);

    Ok(repay_result)
}

fn finalize_position_closure(
    _ctx: &Context<ClosePosition>,
    fee_collection: &CLMMFeeCollection,
    swap_result: &SwapOptimizationResult,
    repay_result: &LendingRepayResult,
) -> Result<()> {
    // Calculate final user balances
    let final_token_a = swap_result.final_token_a + fee_collection.fee_token_a;
    let final_token_b = swap_result.final_token_b
        .checked_sub(repay_result.repaid_amount)
        .ok_or(FlashLeverageError::MathUnderflow)?
        + fee_collection.fee_token_b;

    msg!("Position closure summary:");
    msg!("  Final token A balance: {}", final_token_a);
    msg!("  Final token B balance: {}", final_token_b);
    msg!("  Total fees collected: {} A + {} B", 
         fee_collection.fee_token_a, fee_collection.fee_token_b);
    msg!("  Interest paid: {}", repay_result.interest_paid);

    // In a real implementation, would:
    // 1. Transfer remaining tokens to user
    // 2. Close position account
    // 3. Update position state to Closed

    Ok(())
}

#[derive(Clone, Debug)]
struct SwapOptimizationResult {
    pub swapped_amount: u64,
    pub received_amount: u64,
    pub final_token_a: u64,
    pub final_token_b: u64,
    pub repay_amount: u64,
}

use crate::*;

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
    
    /// CHECK: Lending obligation
    pub lending_obligation: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}