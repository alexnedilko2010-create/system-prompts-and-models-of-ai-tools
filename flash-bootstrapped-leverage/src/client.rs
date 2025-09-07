use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
        system_program,
    },
    Client, Program,
};
use anchor_lang::prelude::*;
use flash_bootstrapped_leverage::{
    accounts::*,
    instruction::*,
    state::*,
    FlashLeverageParams,
    ID as PROGRAM_ID,
};
use std::rc::Rc;
use tokio;

/// Client for interacting with the Flash Bootstrapped Leverage program
pub struct FlashLeverageClient {
    pub program: Program<Rc<Keypair>>,
    pub payer: Rc<Keypair>,
}

impl FlashLeverageClient {
    /// Create a new client instance
    pub fn new(
        cluster_url: &str,
        payer: Keypair,
        commitment: CommitmentConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let payer = Rc::new(payer);
        let client = Client::new_with_options(cluster_url, payer.clone(), commitment);
        let program = client.program(PROGRAM_ID)?;

        Ok(Self { program, payer })
    }

    /// Initialize the flash leverage program
    pub async fn initialize(&self) -> Result<Signature, Box<dyn std::error::Error>> {
        let leverage_state = Pubkey::find_program_address(
            &[b"leverage_state"],
            &PROGRAM_ID,
        ).0;

        let tx = self
            .program
            .request()
            .accounts(Initialize {
                leverage_state,
                authority: self.payer.pubkey(),
                system_program: system_program::ID,
            })
            .args(initialize {})
            .send()
            .await?;

        println!("✅ Program initialized: {}", tx);
        Ok(tx)
    }

    /// Execute flash-bootstrapped leverage strategy
    pub async fn execute_flash_leverage(
        &self,
        params: FlashLeverageParams,
        accounts: ExecuteFlashLeverageAccounts,
    ) -> Result<Signature, Box<dyn std::error::Error>> {
        let leverage_state = Pubkey::find_program_address(
            &[b"leverage_state"],
            &PROGRAM_ID,
        ).0;

        let tx = self
            .program
            .request()
            .accounts(ExecuteFlashLeverage {
                leverage_state,
                user: self.payer.pubkey(),
                user_token_a_account: accounts.user_token_a_account,
                user_token_b_account: accounts.user_token_b_account,
                token_a_mint: accounts.token_a_mint,
                token_b_mint: accounts.token_b_mint,
                flash_loan_program: accounts.flash_loan_program,
                jupiter_program: accounts.jupiter_program,
                clmm_program: accounts.clmm_program,
                pool_account: accounts.pool_account,
                position_account: accounts.position_account,
                lending_program: accounts.lending_program,
                lending_market: accounts.lending_market,
                reserve_account: accounts.reserve_account,
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
            })
            .args(execute_flash_leverage { params })
            .send()
            .await?;

        println!("🚀 Flash leverage executed: {}", tx);
        Ok(tx)
    }

    /// Close a leveraged position
    pub async fn close_position(
        &self,
        accounts: ClosePositionAccounts,
    ) -> Result<Signature, Box<dyn std::error::Error>> {
        let leverage_state = Pubkey::find_program_address(
            &[b"leverage_state"],
            &PROGRAM_ID,
        ).0;

        let tx = self
            .program
            .request()
            .accounts(ClosePosition {
                leverage_state,
                user: self.payer.pubkey(),
                user_token_a_account: accounts.user_token_a_account,
                user_token_b_account: accounts.user_token_b_account,
                clmm_program: accounts.clmm_program,
                position_account: accounts.position_account,
                lending_program: accounts.lending_program,
                lending_obligation: accounts.lending_obligation,
                token_program: anchor_spl::token::ID,
            })
            .args(close_position {})
            .send()
            .await?;

        println!("🔄 Position closed: {}", tx);
        Ok(tx)
    }

    /// Emergency liquidation of a position
    pub async fn emergency_liquidate(
        &self,
        accounts: EmergencyLiquidateAccounts,
    ) -> Result<Signature, Box<dyn std::error::Error>> {
        let leverage_state = Pubkey::find_program_address(
            &[b"leverage_state"],
            &PROGRAM_ID,
        ).0;

        let tx = self
            .program
            .request()
            .accounts(EmergencyLiquidate {
                leverage_state,
                authority: self.payer.pubkey(),
                user_token_a_account: accounts.user_token_a_account,
                user_token_b_account: accounts.user_token_b_account,
                clmm_program: accounts.clmm_program,
                position_account: accounts.position_account,
                lending_program: accounts.lending_program,
                token_program: anchor_spl::token::ID,
            })
            .args(emergency_liquidate {})
            .send()
            .await?;

        println!("🚨 Emergency liquidation completed: {}", tx);
        Ok(tx)
    }

    /// Get leverage state
    pub async fn get_leverage_state(&self) -> Result<LeverageState, Box<dyn std::error::Error>> {
        let leverage_state_pubkey = Pubkey::find_program_address(
            &[b"leverage_state"],
            &PROGRAM_ID,
        ).0;

        let state: LeverageState = self.program.account(leverage_state_pubkey).await?;
        Ok(state)
    }

    /// Create a sample flash leverage transaction for testing
    pub async fn create_sample_transaction(
        &self,
        token_a_mint: Pubkey,
        token_b_mint: Pubkey,
        flash_borrow_amount: u64,
        leverage_multiplier: u64,
    ) -> Result<FlashLeverageParams, Box<dyn std::error::Error>> {
        // Create sample parameters
        let params = FlashLeverageParams {
            flash_borrow_amount,
            leverage_multiplier,
            slippage_tolerance_bps: 100, // 1%
            price_range: PriceRange {
                lower_price: 95_000_000,  // $95
                upper_price: 105_000_000, // $105
                current_price: 100_000_000, // $100
            },
            flash_loan_provider: FlashLoanProvider::Solend,
            clmm_provider: CLMMProvider::OrcaWhirlpool,
            lending_provider: LendingProvider::MarginFi,
            min_token_a_amount: 0,
            max_token_b_swap_amount: flash_borrow_amount / 2,
            jupiter_route_data: create_sample_jupiter_route(token_a_mint, token_b_mint)?,
        };

        Ok(params)
    }
}

/// Helper structs for organizing account inputs
#[derive(Clone, Debug)]
pub struct ExecuteFlashLeverageAccounts {
    pub user_token_a_account: Pubkey,
    pub user_token_b_account: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub flash_loan_program: Pubkey,
    pub jupiter_program: Pubkey,
    pub clmm_program: Pubkey,
    pub pool_account: Pubkey,
    pub position_account: Pubkey,
    pub lending_program: Pubkey,
    pub lending_market: Pubkey,
    pub reserve_account: Pubkey,
}

#[derive(Clone, Debug)]
pub struct ClosePositionAccounts {
    pub user_token_a_account: Pubkey,
    pub user_token_b_account: Pubkey,
    pub clmm_program: Pubkey,
    pub position_account: Pubkey,
    pub lending_program: Pubkey,
    pub lending_obligation: Pubkey,
}

#[derive(Clone, Debug)]
pub struct EmergencyLiquidateAccounts {
    pub user_token_a_account: Pubkey,
    pub user_token_b_account: Pubkey,
    pub clmm_program: Pubkey,
    pub position_account: Pubkey,
    pub lending_program: Pubkey,
}

/// Create sample Jupiter route data for testing
fn create_sample_jupiter_route(
    input_mint: Pubkey,
    output_mint: Pubkey,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut route_data = Vec::new();
    
    // Input mint (32 bytes)
    route_data.extend_from_slice(&input_mint.to_bytes());
    
    // Output mint (32 bytes)
    route_data.extend_from_slice(&output_mint.to_bytes());
    
    // Valid until timestamp (8 bytes)
    let valid_until = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64 + 30; // Valid for 30 seconds
    route_data.extend_from_slice(&valid_until.to_le_bytes());
    
    // Route info (simplified)
    route_data.extend_from_slice(&[1u8; 32]); // Route step 1
    route_data.extend_from_slice(&[2u8; 32]); // Route step 2
    
    Ok(route_data)
}

/// Example usage and testing
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Flash Bootstrapped Leverage Client Example");
    
    // This would typically be loaded from environment or config
    let cluster_url = "https://api.devnet.solana.com";
    let payer = Keypair::new(); // In practice, load from file or environment
    
    // Create client
    let client = FlashLeverageClient::new(
        cluster_url,
        payer,
        CommitmentConfig::confirmed(),
    )?;
    
    println!("📋 Client initialized");
    println!("Program ID: {}", PROGRAM_ID);
    println!("Payer: {}", client.payer.pubkey());
    
    // Example: Initialize program (only needed once)
    match client.initialize().await {
        Ok(signature) => println!("✅ Program initialized: {}", signature),
        Err(e) => println!("⚠️  Program already initialized or error: {}", e),
    }
    
    // Example: Get program state
    match client.get_leverage_state().await {
        Ok(state) => {
            println!("📊 Program State:");
            println!("  Authority: {}", state.authority);
            println!("  Active positions: {}", state.active_positions);
            println!("  Max leverage: {}x", state.config.max_leverage as f64 / 100.0);
        }
        Err(e) => println!("❌ Failed to get state: {}", e),
    }
    
    // Example token mints (these would be real token mints in practice)
    let token_a_mint = Pubkey::new_unique();
    let token_b_mint = Pubkey::new_unique();
    
    // Create sample transaction parameters
    let params = client.create_sample_transaction(
        token_a_mint,
        token_b_mint,
        1_000_000_000, // 1,000 tokens
        300,           // 3x leverage
    ).await?;
    
    println!("📋 Sample transaction parameters created:");
    println!("  Flash borrow amount: {}", params.flash_borrow_amount);
    println!("  Leverage: {}x", params.leverage_multiplier as f64 / 100.0);
    println!("  Price range: ${} - ${}", 
             params.price_range.lower_price as f64 / 1_000_000.0,
             params.price_range.upper_price as f64 / 1_000_000.0);
    
    println!("🎯 Example completed successfully!");
    println!("💡 In a real implementation, you would:");
    println!("   1. Fund the payer account with SOL");
    println!("   2. Create actual token accounts");
    println!("   3. Get real program addresses for integrations");
    println!("   4. Execute the flash leverage transaction");
    
    Ok(())
}

/// Utility functions for common operations
impl FlashLeverageClient {
    /// Get program addresses for common integrations
    pub fn get_integration_addresses() -> IntegrationAddresses {
        IntegrationAddresses {
            // Solend
            solend_program: "So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo"
                .parse()
                .unwrap(),
            
            // Kamino
            kamino_program: "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"
                .parse()
                .unwrap(),
            
            // MarginFi
            marginfi_program: "MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA"
                .parse()
                .unwrap(),
            
            // Jupiter
            jupiter_program: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
                .parse()
                .unwrap(),
            
            // Orca Whirlpool
            orca_whirlpool_program: "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"
                .parse()
                .unwrap(),
            
            // Raydium CLMM
            raydium_clmm_program: "CAMMCzo5YL8w4VFF8KVHrK22GGUQpMDdHZe1ALfMiKNj"
                .parse()
                .unwrap(),
            
            // Meteora DLMM
            meteora_dlmm_program: "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"
                .parse()
                .unwrap(),
        }
    }
    
    /// Validate transaction parameters before execution
    pub fn validate_params(&self, params: &FlashLeverageParams) -> Result<(), String> {
        // Validate leverage multiplier
        if params.leverage_multiplier == 0 || params.leverage_multiplier > 1000 {
            return Err("Invalid leverage multiplier (must be 1-1000)".to_string());
        }
        
        // Validate slippage tolerance
        if params.slippage_tolerance_bps > 1000 {
            return Err("Slippage tolerance too high (max 10%)".to_string());
        }
        
        // Validate price range
        if params.price_range.lower_price >= params.price_range.upper_price {
            return Err("Invalid price range".to_string());
        }
        
        // Validate flash borrow amount
        if params.flash_borrow_amount == 0 {
            return Err("Flash borrow amount cannot be zero".to_string());
        }
        
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct IntegrationAddresses {
    pub solend_program: Pubkey,
    pub kamino_program: Pubkey,
    pub marginfi_program: Pubkey,
    pub jupiter_program: Pubkey,
    pub orca_whirlpool_program: Pubkey,
    pub raydium_clmm_program: Pubkey,
    pub meteora_dlmm_program: Pubkey,
}