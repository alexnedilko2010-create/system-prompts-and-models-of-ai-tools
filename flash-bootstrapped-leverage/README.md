# Flash-Bootstrapped Leverage (Мгновенное плечо)

A sophisticated Solana program that implements flash-bootstrapped leverage for concentrated liquidity positions. This strategy allows users to increase their CLMM positions using flash loans, then convert the debt to traditional lending for long-term fee collection.

## 🎯 Strategy Overview

The flash-bootstrapped leverage strategy works in these atomic steps:

1. **Flash Loan** - Borrow token B instantly without collateral
2. **Jupiter Swap** - Optimize token A/B ratio for target price range
3. **CLMM Liquidity** - Add concentrated liquidity to earn fees
4. **Traditional Loan** - Borrow from lending protocol to repay flash loan
5. **Flash Repayment** - Complete the atomic transaction

**Result**: Leveraged CLMM position with traditional debt that earns fees over time.

## 🏗️ Architecture

### Core Components

- **Flash Loan Integration**: Support for Solend, Kamino, MarginFi, Port Finance
- **Jupiter Integration**: Optimal token swapping with slippage protection
- **CLMM Integration**: Orca Whirlpool, Raydium CLMM, Meteora DLMM
- **Lending Integration**: Traditional borrowing for flash loan repayment
- **Oracle Integration**: Pyth, Switchboard, Chainlink price feeds
- **Risk Management**: Health factor monitoring and liquidation protection

### Program Instructions

```rust
// Initialize the program
initialize()

// Execute flash-bootstrapped leverage
execute_flash_leverage(params: FlashLeverageParams)

// Close leveraged position
close_position()

// Emergency liquidation
emergency_liquidate()
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- Solana CLI 1.18+
- Anchor Framework 0.29+

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd flash-bootstrapped-leverage

# Build the program
anchor build

# Run tests
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet
```

### Basic Usage

```rust
use flash_bootstrapped_leverage::{FlashLeverageClient, FlashLeverageParams};

// Create client
let client = FlashLeverageClient::new(
    "https://api.devnet.solana.com",
    payer_keypair,
    CommitmentConfig::confirmed(),
)?;

// Initialize program (one-time)
client.initialize().await?;

// Create leverage parameters
let params = FlashLeverageParams {
    flash_borrow_amount: 1_000_000_000,    // 1,000 tokens
    leverage_multiplier: 300,              // 3x leverage
    slippage_tolerance_bps: 100,           // 1% slippage
    price_range: PriceRange {
        lower_price: 95_000_000,           // $95
        upper_price: 105_000_000,          // $105
        current_price: 100_000_000,        // $100
    },
    flash_loan_provider: FlashLoanProvider::Solend,
    clmm_provider: CLMMProvider::OrcaWhirlpool,
    lending_provider: LendingProvider::MarginFi,
    // ... other parameters
};

// Execute strategy
let signature = client.execute_flash_leverage(params, accounts).await?;
println!("Strategy executed: {}", signature);
```

## 🔧 Configuration

### Leverage Settings

```rust
pub struct LeverageConfig {
    pub max_leverage: u64,              // Maximum leverage (e.g., 500 = 5x)
    pub max_slippage_bps: u16,          // Maximum slippage (e.g., 100 = 1%)
    pub liquidation_threshold_bps: u16, // Liquidation threshold (e.g., 8500 = 85%)
    pub protocol_fee_bps: u16,          // Protocol fee (e.g., 10 = 0.1%)
    pub min_position_size: u64,         // Minimum position size in USD
    pub max_position_size: u64,         // Maximum position size in USD
}
```

### Provider Configuration

The program supports multiple providers for each integration:

- **Flash Loans**: Solend, Kamino, MarginFi, Port Finance
- **DEX Aggregation**: Jupiter (with route optimization)
- **CLMM**: Orca Whirlpool, Raydium CLMM, Meteora DLMM
- **Lending**: MarginFi, Solend, Kamino, Port Finance
- **Oracles**: Pyth, Switchboard, Chainlink

## 📊 Risk Management

### Health Factor Monitoring

The program continuously monitors position health:

```rust
health_factor = (collateral_value * liquidation_threshold) / debt_value
```

- **> 2.0**: Very Safe
- **1.5 - 2.0**: Safe
- **1.2 - 1.5**: Moderate Risk
- **1.05 - 1.2**: High Risk
- **< 1.05**: Liquidation Risk

### Impermanent Loss Protection

Built-in IL monitoring for concentrated positions:

```rust
let il_info = Utils::calculate_impermanent_loss(
    initial_price,
    current_price,
    price_lower,
    price_upper,
)?;
```

### Automatic Rebalancing

Smart rebalancing thresholds based on volatility:

```rust
let rebalancing = Utils::calculate_rebalancing_threshold(
    current_price,
    price_lower,
    price_upper,
    volatility_bps,
)?;
```

## 🔒 Security Features

### Multi-layer Validation

1. **Parameter Validation**: All inputs validated before execution
2. **Oracle Staleness Checks**: Price feeds must be recent
3. **Slippage Protection**: Maximum slippage enforcement
4. **Health Factor Guards**: Prevent unhealthy positions
5. **Deadline Enforcement**: Transaction time limits

### Emergency Procedures

```rust
// Emergency liquidation when health factor is critical
emergency_liquidate()

// Graceful position closure
close_position()
```

### Audit Considerations

- All math operations use checked arithmetic
- Comprehensive error handling with custom error types
- Input validation at every step
- Oracle price validation and staleness checks
- Reentrancy protection through program design

## 📈 Economics

### Fee Structure

- **Protocol Fee**: 0.1% of position value
- **Flash Loan Fees**: 0.03% - 0.10% depending on provider
- **Swap Fees**: Jupiter aggregation fees
- **CLMM Fees**: 0.05% - 1% depending on pool
- **Lending Rates**: 2% - 7% APR depending on provider

### Expected Returns

Concentrated liquidity positions can achieve higher fee APY:

```rust
let fee_apy = Utils::calculate_fee_apy(
    liquidity_amount,
    pool_volume_24h,
    fee_tier_bps,
    total_liquidity,
    concentration_factor,
)?;
```

Typical returns:
- **Conservative Range (±10%)**: 5-15% APY
- **Moderate Range (±5%)**: 10-30% APY  
- **Aggressive Range (±2%)**: 20-80% APY

*Note: Higher concentration = higher fees but more IL risk*

## 🧪 Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test test_flash_loan_integration

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Test with local validator
anchor test

# Test specific scenarios
anchor test --skip-deploy --skip-local-validator
```

### Simulation Testing

The program includes comprehensive simulation tools:

```rust
// Simulate strategy execution
let simulation = client.simulate_flash_leverage(params).await?;
println!("Expected profit: {}", simulation.expected_profit);
println!("Risk score: {}", simulation.risk_assessment.composite_score);
```

## 📚 Examples

### Example 1: Conservative Strategy

```rust
let conservative_params = FlashLeverageParams {
    flash_borrow_amount: 500_000_000,     // 500 tokens
    leverage_multiplier: 200,             // 2x leverage
    slippage_tolerance_bps: 50,           // 0.5% slippage
    price_range: PriceRange {
        lower_price: 90_000_000,          // ±10% range
        upper_price: 110_000_000,
        current_price: 100_000_000,
    },
    flash_loan_provider: FlashLoanProvider::Kamino,  // Low fees
    clmm_provider: CLMMProvider::OrcaWhirlpool,      // Stable
    lending_provider: LendingProvider::MarginFi,     // Good rates
    // ...
};
```

### Example 2: Aggressive Strategy

```rust
let aggressive_params = FlashLeverageParams {
    flash_borrow_amount: 2_000_000_000,   // 2,000 tokens
    leverage_multiplier: 500,             // 5x leverage
    slippage_tolerance_bps: 100,          // 1% slippage
    price_range: PriceRange {
        lower_price: 98_000_000,          // ±2% range
        upper_price: 102_000_000,
        current_price: 100_000_000,
    },
    flash_loan_provider: FlashLoanProvider::Solend,
    clmm_provider: CLMMProvider::RaydiumCLMM,
    lending_provider: LendingProvider::Kamino,
    // ...
};
```

## 🔍 Monitoring & Analytics

### Position Tracking

```rust
// Get position details
let position = client.get_user_position(user_pubkey).await?;
println!("Health factor: {}", position.lending_data.health_factor);
println!("Fees earned: {} A + {} B", 
         position.clmm_data.accumulated_fees_a,
         position.clmm_data.accumulated_fees_b);

// Risk assessment
let risk = Utils::assess_position_risk(
    position.lending_data.health_factor,
    &il_info,
    market_volatility_bps,
    liquidity_utilization_bps,
)?;
```

### Performance Metrics

- **Total Return**: Position value change + fees collected
- **APY Calculation**: Annualized returns including IL
- **Risk-Adjusted Returns**: Sharpe ratio for position
- **Fee Efficiency**: Fees earned vs. IL incurred

## 🚨 Risks & Disclaimers

### Primary Risks

1. **Impermanent Loss**: Concentrated positions have higher IL risk
2. **Liquidation Risk**: Leveraged positions can be liquidated
3. **Smart Contract Risk**: Protocol vulnerabilities
4. **Oracle Risk**: Price feed manipulation or failures
5. **Market Risk**: Extreme volatility or low liquidity

### Risk Mitigation

- Start with lower leverage (2-3x)
- Use wider price ranges initially
- Monitor health factor regularly
- Set up alerts for rebalancing
- Understand IL implications

### Important Notes

⚠️ **This is experimental software. Use at your own risk.**

- Thoroughly test on devnet before mainnet
- Start with small amounts
- Understand all risks involved
- Keep emergency funds for liquidation protection
- Monitor positions actively

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone and setup
git clone <repository-url>
cd flash-bootstrapped-leverage
npm install

# Install dependencies
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked

# Run development environment
anchor test --skip-deploy
```

## 📞 Support

- **Documentation**: [docs/](docs/)
- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Discord**: [Community Discord](https://discord.gg/community)

## 🗺️ Roadmap

### Phase 1: Core Implementation ✅
- Flash loan integrations
- CLMM integrations  
- Basic lending integrations
- Safety mechanisms

### Phase 2: Advanced Features 🚧
- Auto-rebalancing
- Strategy optimization
- Advanced risk management
- Performance analytics

### Phase 3: Ecosystem Integration 📋
- Additional DEX integrations
- More lending protocols
- Cross-chain support
- Institutional features

---

**Built with ❤️ for the Solana DeFi ecosystem**

*Flash-bootstrapped leverage enables sophisticated DeFi strategies while maintaining atomic transaction safety. Perfect for advanced users looking to maximize capital efficiency in concentrated liquidity markets.*