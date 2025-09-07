use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    },
};
use flash_bootstrapped_leverage::{
    state::*,
    FlashLeverageParams,
};
use std::str::FromStr;

/// Example demonstrating basic usage of the Flash Bootstrapped Leverage program
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Flash Bootstrapped Leverage - Basic Usage Example");
    println!("=====================================================");

    // Configuration
    let cluster_url = "https://api.devnet.solana.com";
    let payer = Keypair::new(); // In practice, load from file
    
    // Example token mints (use real ones in practice)
    let token_a_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?; // SOL
    let token_b_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?; // USDC

    println!("📋 Configuration:");
    println!("  Cluster: {}", cluster_url);
    println!("  Payer: {}", payer.pubkey());
    println!("  Token A (SOL): {}", token_a_mint);
    println!("  Token B (USDC): {}", token_b_mint);

    // Example 1: Conservative 2x Leverage Strategy
    println!("\n💼 Example 1: Conservative 2x Leverage Strategy");
    println!("===============================================");
    
    let conservative_strategy = create_conservative_strategy(
        token_a_mint,
        token_b_mint,
        1_000_000_000, // 1,000 USDC
    )?;
    
    print_strategy_details("Conservative", &conservative_strategy);
    
    // Example 2: Moderate 3x Leverage Strategy
    println!("\n⚡ Example 2: Moderate 3x Leverage Strategy");
    println!("==========================================");
    
    let moderate_strategy = create_moderate_strategy(
        token_a_mint,
        token_b_mint,
        2_000_000_000, // 2,000 USDC
    )?;
    
    print_strategy_details("Moderate", &moderate_strategy);
    
    // Example 3: Aggressive 5x Leverage Strategy
    println!("\n🔥 Example 3: Aggressive 5x Leverage Strategy");
    println!("=============================================");
    
    let aggressive_strategy = create_aggressive_strategy(
        token_a_mint,
        token_b_mint,
        5_000_000_000, // 5,000 USDC
    )?;
    
    print_strategy_details("Aggressive", &aggressive_strategy);

    // Risk Analysis
    println!("\n📊 Risk Analysis");
    println!("================");
    
    analyze_strategy_risks(&conservative_strategy, "Conservative")?;
    analyze_strategy_risks(&moderate_strategy, "Moderate")?;
    analyze_strategy_risks(&aggressive_strategy, "Aggressive")?;

    // Expected Returns
    println!("\n💰 Expected Returns Analysis");
    println!("============================");
    
    calculate_expected_returns(&conservative_strategy, "Conservative")?;
    calculate_expected_returns(&moderate_strategy, "Moderate")?;
    calculate_expected_returns(&aggressive_strategy, "Aggressive")?;

    println!("\n✅ Example completed successfully!");
    println!("\n💡 Next Steps:");
    println!("  1. Fund your wallet with SOL for transaction fees");
    println!("  2. Create token accounts for your chosen tokens");
    println!("  3. Initialize the program (one-time setup)");
    println!("  4. Execute your chosen strategy");
    println!("  5. Monitor position health and collect fees");

    Ok(())
}

/// Create a conservative 2x leverage strategy
fn create_conservative_strategy(
    token_a_mint: Pubkey,
    token_b_mint: Pubkey,
    flash_borrow_amount: u64,
) -> Result<FlashLeverageParams, Box<dyn std::error::Error>> {
    Ok(FlashLeverageParams {
        flash_borrow_amount,
        leverage_multiplier: 200, // 2x leverage
        slippage_tolerance_bps: 50, // 0.5% slippage
        price_range: PriceRange {
            lower_price: 90_000_000,  // $90 (±10% range)
            upper_price: 110_000_000, // $110
            current_price: 100_000_000, // $100
        },
        flash_loan_provider: FlashLoanProvider::Kamino, // Lower fees
        clmm_provider: CLMMProvider::OrcaWhirlpool,     // Most stable
        lending_provider: LendingProvider::MarginFi,    // Good rates
        min_token_a_amount: flash_borrow_amount / 4,    // 25% minimum
        max_token_b_swap_amount: flash_borrow_amount / 3, // 33% maximum swap
        jupiter_route_data: create_sample_route_data(token_a_mint, token_b_mint)?,
    })
}

/// Create a moderate 3x leverage strategy
fn create_moderate_strategy(
    token_a_mint: Pubkey,
    token_b_mint: Pubkey,
    flash_borrow_amount: u64,
) -> Result<FlashLeverageParams, Box<dyn std::error::Error>> {
    Ok(FlashLeverageParams {
        flash_borrow_amount,
        leverage_multiplier: 300, // 3x leverage
        slippage_tolerance_bps: 75, // 0.75% slippage
        price_range: PriceRange {
            lower_price: 95_000_000,  // $95 (±5% range)
            upper_price: 105_000_000, // $105
            current_price: 100_000_000, // $100
        },
        flash_loan_provider: FlashLoanProvider::Solend,
        clmm_provider: CLMMProvider::RaydiumCLMM,       // Higher fees but more volume
        lending_provider: LendingProvider::Kamino,      // Competitive rates
        min_token_a_amount: flash_borrow_amount / 3,    // 33% minimum
        max_token_b_swap_amount: flash_borrow_amount / 2, // 50% maximum swap
        jupiter_route_data: create_sample_route_data(token_a_mint, token_b_mint)?,
    })
}

/// Create an aggressive 5x leverage strategy
fn create_aggressive_strategy(
    token_a_mint: Pubkey,
    token_b_mint: Pubkey,
    flash_borrow_amount: u64,
) -> Result<FlashLeverageParams, Box<dyn std::error::Error>> {
    Ok(FlashLeverageParams {
        flash_borrow_amount,
        leverage_multiplier: 500, // 5x leverage
        slippage_tolerance_bps: 100, // 1% slippage
        price_range: PriceRange {
            lower_price: 98_000_000,  // $98 (±2% range)
            upper_price: 102_000_000, // $102
            current_price: 100_000_000, // $100
        },
        flash_loan_provider: FlashLoanProvider::MarginFi,
        clmm_provider: CLMMProvider::MeteoraDLMM,       // Dynamic fees
        lending_provider: LendingProvider::Solend,      // Established protocol
        min_token_a_amount: flash_borrow_amount / 2,    // 50% minimum
        max_token_b_swap_amount: (flash_borrow_amount * 3) / 4, // 75% maximum swap
        jupiter_route_data: create_sample_route_data(token_a_mint, token_b_mint)?,
    })
}

/// Print detailed strategy information
fn print_strategy_details(name: &str, params: &FlashLeverageParams) {
    println!("Strategy: {}", name);
    println!("  Flash Borrow: {} tokens", params.flash_borrow_amount);
    println!("  Leverage: {}x", params.leverage_multiplier as f64 / 100.0);
    println!("  Slippage Tolerance: {}%", params.slippage_tolerance_bps as f64 / 100.0);
    println!("  Price Range: ${} - ${}", 
             params.price_range.lower_price as f64 / 1_000_000.0,
             params.price_range.upper_price as f64 / 1_000_000.0);
    println!("  Current Price: ${}", params.price_range.current_price as f64 / 1_000_000.0);
    println!("  Flash Loan Provider: {:?}", params.flash_loan_provider);
    println!("  CLMM Provider: {:?}", params.clmm_provider);
    println!("  Lending Provider: {:?}", params.lending_provider);
    
    // Calculate range width
    let range_width = ((params.price_range.upper_price - params.price_range.lower_price) as f64 / params.price_range.current_price as f64) * 100.0;
    println!("  Range Width: ±{:.1}%", range_width / 2.0);
}

/// Analyze strategy risks
fn analyze_strategy_risks(
    params: &FlashLeverageParams,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Strategy Risk Analysis:", name);
    
    // Calculate risk metrics
    let leverage_risk = calculate_leverage_risk(params.leverage_multiplier);
    let range_risk = calculate_range_risk(&params.price_range);
    let provider_risk = calculate_provider_risk(&params.flash_loan_provider, &params.lending_provider);
    
    println!("  Leverage Risk: {} ({}x multiplier)", leverage_risk.0, params.leverage_multiplier as f64 / 100.0);
    println!("  Range Risk: {} (±{:.1}% range)", range_risk.0, range_risk.1);
    println!("  Provider Risk: {}", provider_risk);
    
    // Overall risk assessment
    let overall_risk = match (leverage_risk.1 + range_risk.2) / 2 {
        0..=2 => "LOW",
        3..=5 => "MEDIUM", 
        6..=7 => "HIGH",
        _ => "CRITICAL",
    };
    
    println!("  Overall Risk: {}", overall_risk);
    
    // Risk recommendations
    match overall_risk {
        "LOW" => println!("  ✅ Suitable for beginners"),
        "MEDIUM" => println!("  ⚠️  Requires active monitoring"),
        "HIGH" => println!("  🚨 For experienced users only"),
        "CRITICAL" => println!("  ⛔ Extreme risk - not recommended"),
        _ => {}
    }
    
    println!();
    Ok(())
}

/// Calculate expected returns for a strategy
fn calculate_expected_returns(
    params: &FlashLeverageParams,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} Strategy Expected Returns:", name);
    
    // Base assumptions
    let pool_volume_24h = 10_000_000_000_000u64; // $10M daily volume
    let fee_tier = 3000u16; // 0.3% fee tier
    let total_pool_liquidity = 100_000_000_000_000u128; // $100M total liquidity
    
    // Calculate position size
    let position_size = params.flash_borrow_amount as f64 * (params.leverage_multiplier as f64 / 100.0);
    
    // Calculate concentration factor (tighter ranges = higher concentration)
    let range_width = (params.price_range.upper_price - params.price_range.lower_price) as f64 / params.price_range.current_price as f64;
    let concentration_factor = (0.2 / range_width).min(10.0); // Cap at 10x concentration
    
    // Calculate daily fees
    let pool_daily_fees = (pool_volume_24h as f64) * (fee_tier as f64 / 1_000_000.0);
    let position_share = position_size / (total_pool_liquidity as f64 / 1_000_000.0);
    let daily_fees = pool_daily_fees * position_share * concentration_factor;
    
    // Calculate APY
    let annual_fees = daily_fees * 365.0;
    let fee_apy = (annual_fees / position_size) * 100.0;
    
    // Calculate borrowing costs
    let borrow_amount = params.flash_borrow_amount as f64;
    let borrow_rate = match params.lending_provider {
        LendingProvider::MarginFi => 5.0,  // 5% APR
        LendingProvider::Solend => 6.0,    // 6% APR
        LendingProvider::Kamino => 4.5,    // 4.5% APR
        LendingProvider::Port => 7.0,      // 7% APR
        _ => 5.5,                          // Average
    };
    let annual_borrow_cost = borrow_amount * (borrow_rate / 100.0);
    
    // Net APY
    let net_apy = ((annual_fees - annual_borrow_cost) / position_size) * 100.0;
    
    println!("  Position Size: ${:.0}", position_size);
    println!("  Concentration Factor: {:.1}x", concentration_factor);
    println!("  Expected Fee APY: {:.1}%", fee_apy);
    println!("  Borrowing Cost: {:.1}%", borrow_rate);
    println!("  Net Expected APY: {:.1}%", net_apy);
    
    // Risk-adjusted returns
    let risk_factor = match name {
        "Conservative" => 0.9,
        "Moderate" => 0.7,
        "Aggressive" => 0.5,
        _ => 0.6,
    };
    let risk_adjusted_apy = net_apy * risk_factor;
    
    println!("  Risk-Adjusted APY: {:.1}%", risk_adjusted_apy);
    
    if net_apy > 0.0 {
        println!("  ✅ Potentially profitable");
    } else {
        println!("  ❌ May not be profitable");
    }
    
    println!();
    Ok(())
}

/// Calculate leverage risk
fn calculate_leverage_risk(leverage_multiplier: u64) -> (&'static str, u8) {
    match leverage_multiplier {
        100..=200 => ("LOW", 1),      // 1-2x
        201..=300 => ("MEDIUM", 3),   // 2-3x
        301..=400 => ("HIGH", 6),     // 3-4x
        401..=500 => ("CRITICAL", 8), // 4-5x
        _ => ("EXTREME", 10),         // >5x
    }
}

/// Calculate range risk
fn calculate_range_risk(price_range: &PriceRange) -> (&'static str, f64, u8) {
    let range_width = ((price_range.upper_price - price_range.lower_price) as f64 / price_range.current_price as f64) * 100.0;
    let half_width = range_width / 2.0;
    
    match half_width as u64 {
        10.. => ("LOW", half_width, 1),      // ±10%+
        5..=9 => ("MEDIUM", half_width, 3),  // ±5-10%
        2..=4 => ("HIGH", half_width, 6),    // ±2-5%
        _ => ("CRITICAL", half_width, 8),    // <±2%
    }
}

/// Calculate provider risk
fn calculate_provider_risk(
    flash_provider: &FlashLoanProvider,
    lending_provider: &LendingProvider,
) -> &'static str {
    let flash_risk = match flash_provider {
        FlashLoanProvider::Solend => 1,   // Established
        FlashLoanProvider::Kamino => 2,   // Newer but solid
        FlashLoanProvider::MarginFi => 1, // Established
        FlashLoanProvider::Port => 3,     // Less volume
        _ => 5,
    };
    
    let lending_risk = match lending_provider {
        LendingProvider::MarginFi => 1,   // Established
        LendingProvider::Solend => 1,     // Established
        LendingProvider::Kamino => 2,     // Newer but solid
        LendingProvider::Port => 3,       // Less volume
        _ => 5,
    };
    
    match (flash_risk + lending_risk) / 2 {
        1 => "LOW",
        2 => "MEDIUM",
        3..=4 => "HIGH",
        _ => "CRITICAL",
    }
}

/// Create sample Jupiter route data
fn create_sample_route_data(
    input_mint: Pubkey,
    output_mint: Pubkey,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut route_data = Vec::new();
    
    // Input mint (32 bytes)
    route_data.extend_from_slice(&input_mint.to_bytes());
    
    // Output mint (32 bytes)
    route_data.extend_from_slice(&output_mint.to_bytes());
    
    // Valid until timestamp (8 bytes) - 30 seconds from now
    let valid_until = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64 + 30;
    route_data.extend_from_slice(&valid_until.to_le_bytes());
    
    // Sample route steps (simplified)
    route_data.extend_from_slice(&[1u8; 32]); // Route step 1
    route_data.extend_from_slice(&[2u8; 32]); // Route step 2
    
    Ok(route_data)
}

/// Print warning about risks
fn print_risk_warning() {
    println!("\n⚠️  IMPORTANT RISK WARNING ⚠️");
    println!("==============================");
    println!("Flash-bootstrapped leverage is an advanced DeFi strategy with significant risks:");
    println!("• LIQUIDATION RISK: Positions can be liquidated if health factor drops");
    println!("• IMPERMANENT LOSS: Concentrated positions have higher IL exposure");  
    println!("• SMART CONTRACT RISK: Protocol vulnerabilities could cause losses");
    println!("• MARKET RISK: Extreme volatility can cause rapid losses");
    println!("• COMPLEXITY RISK: Strategy requires active monitoring and management");
    println!("\nOnly use funds you can afford to lose. Start small and understand all risks.");
    println!("This is experimental software - use at your own risk!");
}