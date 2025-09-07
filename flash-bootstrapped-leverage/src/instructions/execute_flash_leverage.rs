use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::integrations::*;

/// Main coordinator instruction that orchestrates the entire flash-bootstrapped leverage flow
pub fn handler(ctx: Context<ExecuteFlashLeverage>, params: FlashLeverageParams) -> Result<()> {
    let leverage_state = &mut ctx.accounts.leverage_state;
    let user = &ctx.accounts.user;
    
    msg!("🚀 Starting flash-bootstrapped leverage execution");
    msg!("User: {}", user.key());
    msg!("Flash borrow amount: {}", params.flash_borrow_amount);
    msg!("Leverage multiplier: {}x", params.leverage_multiplier as f64 / 100.0);

    // Step 0: Validate parameters and check safety constraints
    validate_flash_leverage_params(&params, &ctx)?;

    // Create user position account
    let position_data = create_position_data(&params, user.key())?;

    // Step 1: Execute flash loan for token B
    msg!("📋 Step 1: Executing flash loan");
    let flash_loan_fee = execute_flash_loan_step(&ctx, &params)?;

    // Step 2: Calculate optimal swap amounts for A/B ratio
    msg!("📋 Step 2: Calculating optimal swap ratios");
    let swap_calculation = calculate_optimal_swap(&ctx, &params)?;

    // Step 3: Execute Jupiter swap if needed
    msg!("📋 Step 3: Executing Jupiter swap");
    let swap_result = execute_jupiter_swap_step(&ctx, &params, &swap_calculation)?;

    // Step 4: Add liquidity to CLMM pool
    msg!("📋 Step 4: Adding liquidity to CLMM pool");
    let clmm_result = execute_clmm_liquidity_step(&ctx, &params, &swap_result)?;

    // Step 5: Take traditional loan to repay flash loan
    msg!("📋 Step 5: Taking traditional loan");
    let lending_result = execute_traditional_loan_step(&ctx, &params, flash_loan_fee)?;

    // Step 6: Repay flash loan
    msg!("📋 Step 6: Repaying flash loan");
    repay_flash_loan_step(&ctx, &params, flash_loan_fee)?;

    // Step 7: Update position state and finalize
    msg!("📋 Step 7: Finalizing position");
    finalize_position(&ctx, position_data, clmm_result, lending_result)?;

    // Update global state
    leverage_state.active_positions = leverage_state.active_positions
        .checked_add(1)
        .ok_or(FlashLeverageError::MathOverflow)?;

    msg!("✅ Flash-bootstrapped leverage execution completed successfully!");
    msg!("Position created with {}x leverage", params.leverage_multiplier as f64 / 100.0);

    Ok(())
}

fn validate_flash_leverage_params(
    params: &FlashLeverageParams,
    ctx: &Context<ExecuteFlashLeverage>,
) -> Result<()> {
    let leverage_state = &ctx.accounts.leverage_state;
    
    // Validate leverage multiplier
    if params.leverage_multiplier == 0 || params.leverage_multiplier > leverage_state.config.max_leverage {
        return Err(FlashLeverageError::InvalidLeverageMultiplier.into());
    }

    // Validate slippage tolerance
    if params.slippage_tolerance_bps > leverage_state.config.max_slippage_bps {
        return Err(FlashLeverageError::SlippageToleranceExceeded.into());
    }

    // Validate price range
    if params.price_range.lower_price >= params.price_range.upper_price {
        return Err(FlashLeverageError::InvalidPriceRange.into());
    }

    // Validate flash loan amount
    FlashLoanIntegration::validate_flash_loan_params(
        &params.flash_loan_provider,
        params.flash_borrow_amount,
        &ctx.accounts.token_b_mint.key(),
    )?;

    // Validate lending parameters
    LendingIntegration::validate_lending_params(
        &params.lending_provider,
        params.flash_borrow_amount,
        params.flash_borrow_amount, // Using same amount as collateral for simplicity
        &ctx.accounts.token_a_mint.key(),
        &ctx.accounts.token_b_mint.key(),
    )?;

    // Validate position size
    let position_value_usd = estimate_position_value_usd(params)?;
    if position_value_usd < leverage_state.config.min_position_size {
        return Err(FlashLeverageError::PositionSizeTooSmall.into());
    }
    if position_value_usd > leverage_state.config.max_position_size {
        return Err(FlashLeverageError::PositionSizeTooLarge.into());
    }

    Ok(())
}

fn create_position_data(params: &FlashLeverageParams, user: Pubkey) -> Result<PositionData> {
    Ok(PositionData {
        token_a_mint: Pubkey::default(), // Will be set from context
        token_b_mint: Pubkey::default(), // Will be set from context
        initial_token_a_amount: 0, // Will be calculated
        initial_token_b_amount: params.flash_borrow_amount,
        current_token_a_amount: 0,
        current_token_b_amount: params.flash_borrow_amount,
        price_range: params.price_range.clone(),
        leverage_multiplier: params.leverage_multiplier,
    })
}

fn execute_flash_loan_step(
    ctx: &Context<ExecuteFlashLeverage>,
    params: &FlashLeverageParams,
) -> Result<u64> {
    // Calculate flash loan fee
    let fee = FlashLoanIntegration::calculate_flash_loan_fee(
        &params.flash_loan_provider,
        params.flash_borrow_amount,
    )?;

    // Execute flash loan
    FlashLoanIntegration::execute_flash_loan(
        &params.flash_loan_provider,
        params.flash_borrow_amount,
        &ctx.accounts.token_b_mint,
        &ctx.accounts.flash_loan_program,
        &ctx.accounts.user_token_b_account,
        &ctx.accounts.token_program,
        ctx.remaining_accounts,
    )?;

    msg!("Flash loan executed: {} tokens borrowed, {} fee", 
         params.flash_borrow_amount, fee);

    Ok(fee)
}

fn calculate_optimal_swap(
    ctx: &Context<ExecuteFlashLeverage>,
    params: &FlashLeverageParams,
) -> Result<SwapCalculation> {
    // Get current token balances
    let current_token_a = ctx.accounts.user_token_a_account.amount;
    let current_token_b = ctx.accounts.user_token_b_account.amount;

    // Calculate target ratio based on price range
    let liquidity_ratio = JupiterIntegration::estimate_liquidity_ratio(
        current_token_a,
        current_token_b,
        params.price_range.lower_price,
        params.price_range.upper_price,
        params.price_range.current_price,
    )?;

    // Calculate optimal swap amounts
    let swap_calculation = JupiterIntegration::calculate_optimal_swap_amounts(
        current_token_a,
        current_token_b,
        liquidity_ratio.ratio_a,
        liquidity_ratio.ratio_b,
        params.price_range.current_price,
    )?;

    msg!("Swap calculation: needs_swap={}, direction={:?}, amount={}", 
         swap_calculation.needs_swap, swap_calculation.swap_direction, swap_calculation.input_amount);

    Ok(swap_calculation)
}

fn execute_jupiter_swap_step(
    ctx: &Context<ExecuteFlashLeverage>,
    params: &FlashLeverageParams,
    swap_calculation: &SwapCalculation,
) -> Result<SwapResult> {
    if !swap_calculation.needs_swap {
        msg!("No swap needed, optimal ratio already achieved");
        return Ok(SwapResult {
            swapped_amount: 0,
            received_amount: 0,
            final_token_a: swap_calculation.final_token_a,
            final_token_b: swap_calculation.final_token_b,
        });
    }

    let (input_mint, output_mint, input_account, output_account) = match swap_calculation.swap_direction {
        SwapDirection::AToB => (
            &ctx.accounts.token_a_mint,
            &ctx.accounts.token_b_mint,
            &ctx.accounts.user_token_a_account,
            &ctx.accounts.user_token_b_account,
        ),
        SwapDirection::BToA => (
            &ctx.accounts.token_b_mint,
            &ctx.accounts.token_a_mint,
            &ctx.accounts.user_token_b_account,
            &ctx.accounts.user_token_a_account,
        ),
    };

    let received_amount = JupiterIntegration::execute_swap(
        input_mint,
        output_mint,
        swap_calculation.input_amount,
        swap_calculation.expected_output,
        params.slippage_tolerance_bps,
        &ctx.accounts.jupiter_program,
        input_account,
        output_account,
        &ctx.accounts.token_program,
        &params.jupiter_route_data,
        ctx.remaining_accounts,
    )?;

    msg!("Jupiter swap completed: {} -> {} tokens", 
         swap_calculation.input_amount, received_amount);

    Ok(SwapResult {
        swapped_amount: swap_calculation.input_amount,
        received_amount,
        final_token_a: swap_calculation.final_token_a,
        final_token_b: swap_calculation.final_token_b,
    })
}

fn execute_clmm_liquidity_step(
    ctx: &Context<ExecuteFlashLeverage>,
    params: &FlashLeverageParams,
    swap_result: &SwapResult,
) -> Result<CLMMLiquidityResult> {
    // Calculate optimal tick range
    let (tick_lower, tick_upper) = CLMMIntegration::calculate_optimal_ticks(
        params.price_range.lower_price,
        params.price_range.upper_price,
        64, // Standard tick spacing
    )?;

    msg!("Adding CLMM liquidity: {} A, {} B, ticks: {}-{}", 
         swap_result.final_token_a, swap_result.final_token_b, tick_lower, tick_upper);

    // Add liquidity to CLMM pool
    let clmm_result = CLMMIntegration::increase_liquidity(
        &params.clmm_provider,
        swap_result.final_token_a,
        swap_result.final_token_b,
        tick_lower,
        tick_upper,
        &ctx.accounts.clmm_program,
        &ctx.accounts.pool_account,
        &ctx.accounts.position_account,
        &ctx.accounts.user_token_a_account,
        &ctx.accounts.user_token_b_account,
        &ctx.accounts.token_a_mint,
        &ctx.accounts.token_b_mint,
        &ctx.accounts.token_program,
        ctx.remaining_accounts,
    )?;

    msg!("CLMM liquidity added: {} liquidity units", clmm_result.liquidity_amount);

    Ok(clmm_result)
}

fn execute_traditional_loan_step(
    ctx: &Context<ExecuteFlashLeverage>,
    params: &FlashLeverageParams,
    flash_loan_fee: u64,
) -> Result<LendingBorrowResult> {
    let total_repay_amount = params.flash_borrow_amount
        .checked_add(flash_loan_fee)
        .ok_or(FlashLeverageError::MathOverflow)?;

    msg!("Taking traditional loan to repay flash loan: {} total", total_repay_amount);

    // Use CLMM position as collateral and borrow the repay amount
    let lending_result = LendingIntegration::borrow(
        &params.lending_provider,
        total_repay_amount,
        params.flash_borrow_amount, // Collateral amount (simplified)
        &ctx.accounts.lending_program,
        &ctx.accounts.lending_market,
        &ctx.accounts.reserve_account,
        &ctx.accounts.lending_market, // Using market as obligation for simplicity
        &ctx.accounts.user_token_a_account, // Collateral account
        &ctx.accounts.user_token_b_account, // Borrow account
        &ctx.accounts.token_a_mint,
        &ctx.accounts.token_b_mint,
        &ctx.accounts.token_program,
        ctx.remaining_accounts,
    )?;

    msg!("Traditional loan executed: {} borrowed at {}% APR", 
         lending_result.borrowed_amount, lending_result.interest_rate_bps as f64 / 100.0);

    Ok(lending_result)
}

fn repay_flash_loan_step(
    ctx: &Context<ExecuteFlashLeverage>,
    params: &FlashLeverageParams,
    flash_loan_fee: u64,
) -> Result<()> {
    msg!("Repaying flash loan: {} principal + {} fee", params.flash_borrow_amount, flash_loan_fee);

    FlashLoanIntegration::repay_flash_loan(
        &params.flash_loan_provider,
        params.flash_borrow_amount,
        flash_loan_fee,
        &ctx.accounts.token_b_mint,
        &ctx.accounts.flash_loan_program,
        &ctx.accounts.user_token_b_account,
        &ctx.accounts.token_program,
        ctx.remaining_accounts,
    )?;

    msg!("Flash loan repaid successfully");
    Ok(())
}

fn finalize_position(
    ctx: &Context<ExecuteFlashLeverage>,
    mut position_data: PositionData,
    clmm_result: CLMMLiquidityResult,
    lending_result: LendingBorrowResult,
) -> Result<()> {
    // Update position data with actual results
    position_data.token_a_mint = ctx.accounts.token_a_mint.key();
    position_data.token_b_mint = ctx.accounts.token_b_mint.key();
    position_data.current_token_a_amount = clmm_result.token_a_amount;
    position_data.current_token_b_amount = clmm_result.token_b_amount;

    // Create CLMM data
    let clmm_data = CLMMData {
        provider: CLMMProvider::OrcaWhirlpool, // From params in real implementation
        pool_address: ctx.accounts.pool_account.key(),
        position_nft: clmm_result.position_nft,
        liquidity_amount: clmm_result.liquidity_amount,
        tick_lower: -1000, // From calculation in real implementation
        tick_upper: 1000,
        fee_tier: 3000, // 0.3% fee tier
        accumulated_fees_a: 0,
        accumulated_fees_b: 0,
    };

    // Create lending data
    let lending_data = LendingData {
        provider: LendingProvider::MarginFi, // From params in real implementation
        obligation_account: lending_result.obligation_account,
        reserve_account: ctx.accounts.reserve_account.key(),
        borrowed_amount: lending_result.borrowed_amount,
        interest_rate_bps: lending_result.interest_rate_bps,
        collateral_amount: lending_result.collateral_deposited,
        health_factor: lending_result.health_factor,
    };

    msg!("Position finalized:");
    msg!("  CLMM liquidity: {} units", clmm_data.liquidity_amount);
    msg!("  Traditional debt: {} tokens at {}% APR", lending_data.borrowed_amount, lending_data.interest_rate_bps as f64 / 100.0);
    msg!("  Health factor: {}", lending_data.health_factor as f64 / 1_000_000.0);

    // In a real implementation, you would save this to a UserPosition account
    // For now, we just log the successful creation

    Ok(())
}

// Helper functions and structs

fn estimate_position_value_usd(params: &FlashLeverageParams) -> Result<u64> {
    // Simplified USD value estimation
    // In real implementation, would use oracle prices
    let base_value = params.flash_borrow_amount / 10000; // Assume $0.0001 per token unit
    let leveraged_value = base_value
        .checked_mul(params.leverage_multiplier)
        .ok_or(FlashLeverageError::MathOverflow)?
        .checked_div(100)
        .ok_or(FlashLeverageError::DivisionByZero)?;
    
    Ok(leveraged_value)
}

#[derive(Clone, Debug)]
struct SwapResult {
    pub swapped_amount: u64,
    pub received_amount: u64,
    pub final_token_a: u64,
    pub final_token_b: u64,
}

use crate::*;

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
    
    // Flash loan provider accounts
    /// CHECK: Flash loan provider program
    pub flash_loan_program: UncheckedAccount<'info>,
    
    // Jupiter swap accounts
    /// CHECK: Jupiter program
    pub jupiter_program: UncheckedAccount<'info>,
    
    // CLMM accounts
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