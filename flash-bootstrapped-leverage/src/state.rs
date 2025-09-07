use anchor_lang::prelude::*;
use rust_decimal::Decimal;

#[account]
#[derive(InitSpace)]
pub struct LeverageState {
    /// Authority that can initialize and manage the program
    pub authority: Pubkey,
    
    /// Bump seed for PDA derivation
    pub bump: u8,
    
    /// Total number of active positions
    pub active_positions: u64,
    
    /// Program configuration
    pub config: LeverageConfig,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct LeverageConfig {
    /// Maximum leverage multiplier allowed (e.g., 3.0 for 3x leverage)
    pub max_leverage: u64, // Stored as basis points (300 = 3.0x)
    
    /// Maximum slippage tolerance for swaps (in basis points)
    pub max_slippage_bps: u16,
    
    /// Emergency liquidation threshold (in basis points)
    pub liquidation_threshold_bps: u16,
    
    /// Protocol fee (in basis points)
    pub protocol_fee_bps: u16,
    
    /// Minimum position size in USD (scaled by 1e6)
    pub min_position_size: u64,
    
    /// Maximum position size in USD (scaled by 1e6)
    pub max_position_size: u64,
}

#[account]
#[derive(InitSpace)]
pub struct UserPosition {
    /// User who owns this position
    pub user: Pubkey,
    
    /// Bump seed for PDA derivation
    pub bump: u8,
    
    /// Position details
    pub position_data: PositionData,
    
    /// Flash loan details
    pub flash_loan_data: FlashLoanData,
    
    /// CLMM position details
    pub clmm_data: CLMMData,
    
    /// Traditional lending details
    pub lending_data: LendingData,
    
    /// Position status
    pub status: PositionStatus,
    
    /// Timestamps
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct PositionData {
    /// Token A mint
    pub token_a_mint: Pubkey,
    
    /// Token B mint
    pub token_b_mint: Pubkey,
    
    /// Initial token A amount
    pub initial_token_a_amount: u64,
    
    /// Initial token B amount
    pub initial_token_b_amount: u64,
    
    /// Current token A amount in position
    pub current_token_a_amount: u64,
    
    /// Current token B amount in position
    pub current_token_b_amount: u64,
    
    /// Target price range for CLMM
    pub price_range: PriceRange,
    
    /// Leverage multiplier used (in basis points)
    pub leverage_multiplier: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct FlashLoanData {
    /// Flash loan provider used
    pub provider: FlashLoanProvider,
    
    /// Amount borrowed in token B
    pub borrowed_amount: u64,
    
    /// Fee paid for flash loan
    pub fee_amount: u64,
    
    /// Flash loan program ID
    pub program_id: Pubkey,
    
    /// Status of flash loan
    pub status: FlashLoanStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct CLMMData {
    /// CLMM provider (Orca/Raydium/Meteora)
    pub provider: CLMMProvider,
    
    /// Pool address
    pub pool_address: Pubkey,
    
    /// Position NFT address (if applicable)
    pub position_nft: Option<Pubkey>,
    
    /// Liquidity amount provided
    pub liquidity_amount: u128,
    
    /// Tick range for concentrated liquidity
    pub tick_lower: i32,
    pub tick_upper: i32,
    
    /// Fee tier
    pub fee_tier: u16,
    
    /// Accumulated fees
    pub accumulated_fees_a: u64,
    pub accumulated_fees_b: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct LendingData {
    /// Lending provider used
    pub provider: LendingProvider,
    
    /// Obligation account
    pub obligation_account: Pubkey,
    
    /// Reserve account
    pub reserve_account: Pubkey,
    
    /// Amount borrowed
    pub borrowed_amount: u64,
    
    /// Interest rate at time of borrowing (in basis points)
    pub interest_rate_bps: u16,
    
    /// Collateral deposited
    pub collateral_amount: u64,
    
    /// Current health factor
    pub health_factor: u64, // Scaled by 1e6
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct PriceRange {
    /// Lower price bound (scaled by 1e6)
    pub lower_price: u64,
    
    /// Upper price bound (scaled by 1e6)
    pub upper_price: u64,
    
    /// Current price (scaled by 1e6)
    pub current_price: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq)]
pub enum PositionStatus {
    /// Position is being created
    Creating,
    /// Position is active and earning fees
    Active,
    /// Position is being closed
    Closing,
    /// Position has been closed
    Closed,
    /// Position is in liquidation
    Liquidating,
    /// Position has been liquidated
    Liquidated,
    /// Position encountered an error
    Error,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq)]
pub enum FlashLoanProvider {
    /// Solend flash loans
    Solend,
    /// Kamino flash loans
    Kamino,
    /// MarginFi flash loans
    MarginFi,
    /// Port Finance flash loans
    Port,
    /// Custom flash loan provider
    Custom(Pubkey),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq)]
pub enum CLMMProvider {
    /// Orca Whirlpools
    OrcaWhirlpool,
    /// Raydium CLMM
    RaydiumCLMM,
    /// Meteora DLMM
    MeteoraDLMM,
    /// Custom CLMM provider
    Custom(Pubkey),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq)]
pub enum LendingProvider {
    /// MarginFi lending
    MarginFi,
    /// Solend lending
    Solend,
    /// Kamino lending
    Kamino,
    /// Port Finance lending
    Port,
    /// Custom lending provider
    Custom(Pubkey),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq)]
pub enum FlashLoanStatus {
    /// Flash loan not yet initiated
    NotStarted,
    /// Flash loan is active
    Active,
    /// Flash loan has been repaid
    Repaid,
    /// Flash loan failed to repay
    Failed,
}

/// Parameters for executing flash leverage strategy
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FlashLeverageParams {
    /// Amount of token B to flash borrow
    pub flash_borrow_amount: u64,
    
    /// Target leverage multiplier (in basis points)
    pub leverage_multiplier: u64,
    
    /// Slippage tolerance for Jupiter swaps (in basis points)
    pub slippage_tolerance_bps: u16,
    
    /// Price range for CLMM position
    pub price_range: PriceRange,
    
    /// Preferred flash loan provider
    pub flash_loan_provider: FlashLoanProvider,
    
    /// Preferred CLMM provider
    pub clmm_provider: CLMMProvider,
    
    /// Preferred lending provider
    pub lending_provider: LendingProvider,
    
    /// Minimum amount of token A to receive after swaps
    pub min_token_a_amount: u64,
    
    /// Maximum amount of token B to use for swaps
    pub max_token_b_swap_amount: u64,
    
    /// Jupiter route data (serialized)
    pub jupiter_route_data: Vec<u8>,
}

impl Default for LeverageConfig {
    fn default() -> Self {
        Self {
            max_leverage: 500, // 5.0x max leverage
            max_slippage_bps: 100, // 1% max slippage
            liquidation_threshold_bps: 8500, // 85% liquidation threshold
            protocol_fee_bps: 10, // 0.1% protocol fee
            min_position_size: 100_000_000, // $100 minimum
            max_position_size: 10_000_000_000, // $10,000 maximum
        }
    }
}