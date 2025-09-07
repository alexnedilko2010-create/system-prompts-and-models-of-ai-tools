use anchor_lang::prelude::*;

#[error_code]
pub enum FlashLeverageError {
    #[msg("Invalid leverage multiplier")]
    InvalidLeverageMultiplier,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageToleranceExceeded,
    
    #[msg("Insufficient flash loan amount")]
    InsufficientFlashLoanAmount,
    
    #[msg("Flash loan repayment failed")]
    FlashLoanRepaymentFailed,
    
    #[msg("Jupiter swap failed")]
    JupiterSwapFailed,
    
    #[msg("CLMM liquidity addition failed")]
    CLMMLiquidityAdditionFailed,
    
    #[msg("Traditional loan failed")]
    TraditionalLoanFailed,
    
    #[msg("Position health factor too low")]
    PositionHealthTooLow,
    
    #[msg("Position already exists")]
    PositionAlreadyExists,
    
    #[msg("Position not found")]
    PositionNotFound,
    
    #[msg("Position not active")]
    PositionNotActive,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    
    #[msg("Invalid price range")]
    InvalidPriceRange,
    
    #[msg("Price out of range")]
    PriceOutOfRange,
    
    #[msg("Insufficient balance")]
    InsufficientBalance,
    
    #[msg("Position size too small")]
    PositionSizeTooSmall,
    
    #[msg("Position size too large")]
    PositionSizeTooLarge,
    
    #[msg("Liquidation threshold reached")]
    LiquidationThresholdReached,
    
    #[msg("Flash loan provider not supported")]
    FlashLoanProviderNotSupported,
    
    #[msg("CLMM provider not supported")]
    CLMMProviderNotSupported,
    
    #[msg("Lending provider not supported")]
    LendingProviderNotSupported,
    
    #[msg("Invalid oracle price")]
    InvalidOraclePrice,
    
    #[msg("Oracle price too stale")]
    OraclePriceTooStale,
    
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Math underflow")]
    MathUnderflow,
    
    #[msg("Division by zero")]
    DivisionByZero,
    
    #[msg("Invalid instruction data")]
    InvalidInstructionData,
    
    #[msg("Account validation failed")]
    AccountValidationFailed,
    
    #[msg("Deadline exceeded")]
    DeadlineExceeded,
    
    #[msg("Emergency shutdown active")]
    EmergencyShutdownActive,
    
    #[msg("Feature not implemented")]
    FeatureNotImplemented,
    
    #[msg("Invalid fee calculation")]
    InvalidFeeCalculation,
    
    #[msg("Protocol fee too high")]
    ProtocolFeeTooHigh,
    
    #[msg("Interest rate too high")]
    InterestRateTooHigh,
    
    #[msg("Collateral ratio insufficient")]
    CollateralRatioInsufficient,
    
    #[msg("Position cannot be liquidated")]
    PositionCannotBeLiquidated,
    
    #[msg("Invalid tick range")]
    InvalidTickRange,
    
    #[msg("Tick range too wide")]
    TickRangeTooWide,
    
    #[msg("Tick range too narrow")]
    TickRangeTooNarrow,
    
    #[msg("Pool not found")]
    PoolNotFound,
    
    #[msg("Reserve not found")]
    ReserveNotFound,
    
    #[msg("Market not found")]
    MarketNotFound,
    
    #[msg("Invalid program ID")]
    InvalidProgramId,
    
    #[msg("Program account mismatch")]
    ProgramAccountMismatch,
    
    #[msg("Token account mismatch")]
    TokenAccountMismatch,
    
    #[msg("Mint mismatch")]
    MintMismatch,
    
    #[msg("Owner mismatch")]
    OwnerMismatch,
    
    #[msg("Invalid signature")]
    InvalidSignature,
    
    #[msg("Transaction too large")]
    TransactionTooLarge,
    
    #[msg("Computation budget exceeded")]
    ComputationBudgetExceeded,
    
    #[msg("Network congestion")]
    NetworkCongestion,
    
    #[msg("RPC error")]
    RPCError,
    
    #[msg("Serialization error")]
    SerializationError,
    
    #[msg("Deserialization error")]
    DeserializationError,
    
    #[msg("Invalid Jupiter route")]
    InvalidJupiterRoute,
    
    #[msg("Jupiter route expired")]
    JupiterRouteExpired,
    
    #[msg("No liquidity available")]
    NoLiquidityAvailable,
    
    #[msg("Impermanent loss too high")]
    ImpermanentLossTooHigh,
    
    #[msg("Fee collection failed")]
    FeeCollectionFailed,
    
    #[msg("Position rebalancing needed")]
    PositionRebalancingNeeded,
    
    #[msg("Health factor calculation failed")]
    HealthFactorCalculationFailed,
    
    #[msg("Risk assessment failed")]
    RiskAssessmentFailed,
    
    #[msg("Market conditions unfavorable")]
    MarketConditionsUnfavorable,
    
    #[msg("Volatility too high")]
    VolatilityTooHigh,
    
    #[msg("Correlation too high")]
    CorrelationTooHigh,
    
    #[msg("Liquidity too low")]
    LiquidityTooLow,
    
    #[msg("Spread too wide")]
    SpreadTooWide,
    
    #[msg("Volume too low")]
    VolumeTooLow,
    
    #[msg("Time window expired")]
    TimeWindowExpired,
    
    #[msg("Cooldown period active")]
    CooldownPeriodActive,
    
    #[msg("Maximum positions reached")]
    MaximumPositionsReached,
    
    #[msg("Minimum hold period not met")]
    MinimumHoldPeriodNotMet,
    
    #[msg("Strategy optimization required")]
    StrategyOptimizationRequired,
}