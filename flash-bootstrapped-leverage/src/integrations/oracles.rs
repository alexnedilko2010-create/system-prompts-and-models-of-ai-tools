use anchor_lang::prelude::*;
use crate::errors::*;

/// Oracle integration for price feeds
pub struct OracleIntegration;

impl OracleIntegration {
    /// Get price from various oracle providers
    pub fn get_price(
        oracle_provider: OracleProvider,
        price_feed_account: &AccountInfo,
    ) -> Result<OraclePrice> {
        match oracle_provider {
            OracleProvider::Pyth => Self::get_pyth_price(price_feed_account),
            OracleProvider::Switchboard => Self::get_switchboard_price(price_feed_account),
            OracleProvider::Chainlink => Self::get_chainlink_price(price_feed_account),
            OracleProvider::Custom(_) => Err(FlashLeverageError::FeatureNotImplemented.into()),
        }
    }

    /// Get Pyth price
    fn get_pyth_price(price_feed_account: &AccountInfo) -> Result<OraclePrice> {
        // In a real implementation, this would parse Pyth price feed data
        // For now, we'll simulate price data
        
        let current_time = Clock::get()?.unix_timestamp;
        
        // Simulate price data (would parse actual Pyth account data)
        let price = 100_000_000; // $100.00 scaled by 1e6
        let confidence = 50_000;  // $0.05 confidence interval
        let last_updated = current_time - 5; // Updated 5 seconds ago
        
        // Validate price staleness
        if current_time - last_updated > 60 { // 60 seconds max staleness
            return Err(FlashLeverageError::OraclePriceTooStale.into());
        }
        
        Ok(OraclePrice {
            price,
            confidence,
            last_updated,
            decimals: 6,
            status: PriceStatus::Valid,
        })
    }

    /// Get Switchboard price
    fn get_switchboard_price(price_feed_account: &AccountInfo) -> Result<OraclePrice> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Simulate Switchboard price data
        let price = 99_800_000; // $99.80
        let confidence = 30_000;  // $0.03 confidence
        let last_updated = current_time - 3;
        
        if current_time - last_updated > 120 { // 2 minutes max staleness
            return Err(FlashLeverageError::OraclePriceTooStale.into());
        }
        
        Ok(OraclePrice {
            price,
            confidence,
            last_updated,
            decimals: 6,
            status: PriceStatus::Valid,
        })
    }

    /// Get Chainlink price
    fn get_chainlink_price(price_feed_account: &AccountInfo) -> Result<OraclePrice> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Simulate Chainlink price data
        let price = 100_200_000; // $100.20
        let confidence = 40_000;   // $0.04 confidence
        let last_updated = current_time - 10;
        
        if current_time - last_updated > 300 { // 5 minutes max staleness
            return Err(FlashLeverageError::OraclePriceTooStale.into());
        }
        
        Ok(OraclePrice {
            price,
            confidence,
            last_updated,
            decimals: 6,
            status: PriceStatus::Valid,
        })
    }

    /// Get aggregated price from multiple oracles
    pub fn get_aggregated_price(
        oracle_feeds: &[(OracleProvider, AccountInfo)],
        max_deviation_bps: u16,
    ) -> Result<AggregatedPrice> {
        if oracle_feeds.is_empty() {
            return Err(FlashLeverageError::InvalidOraclePrice.into());
        }

        let mut prices = Vec::new();
        let mut total_weight = 0u64;
        let mut weighted_sum = 0u128;

        for (provider, account) in oracle_feeds {
            match Self::get_price(*provider, account) {
                Ok(oracle_price) => {
                    if oracle_price.status != PriceStatus::Valid {
                        continue;
                    }

                    let weight = Self::get_oracle_weight(*provider);
                    let weighted_price = (oracle_price.price as u128) * (weight as u128);
                    
                    weighted_sum = weighted_sum
                        .checked_add(weighted_price)
                        .ok_or(FlashLeverageError::MathOverflow)?;
                    
                    total_weight = total_weight
                        .checked_add(weight)
                        .ok_or(FlashLeverageError::MathOverflow)?;
                    
                    prices.push(oracle_price);
                }
                Err(_) => continue, // Skip failed oracle feeds
            }
        }

        if prices.is_empty() {
            return Err(FlashLeverageError::InvalidOraclePrice.into());
        }

        let weighted_average = (weighted_sum / total_weight as u128) as u64;

        // Check price deviation
        let max_price = prices.iter().map(|p| p.price).max().unwrap();
        let min_price = prices.iter().map(|p| p.price).min().unwrap();
        
        let deviation = if max_price > min_price {
            ((max_price - min_price) as u128 * 10000) / weighted_average as u128
        } else {
            0
        };

        if deviation > max_deviation_bps as u128 {
            return Err(FlashLeverageError::InvalidOraclePrice.into());
        }

        let confidence = Self::calculate_confidence(&prices)?;
        let latest_update = prices.iter().map(|p| p.last_updated).max().unwrap();

        Ok(AggregatedPrice {
            price: weighted_average,
            confidence,
            last_updated: latest_update,
            num_oracles: prices.len() as u8,
            deviation_bps: deviation as u16,
            status: PriceStatus::Valid,
        })
    }

    /// Calculate TWAP (Time Weighted Average Price)
    pub fn calculate_twap(
        price_history: &[HistoricalPrice],
        time_window_seconds: i64,
    ) -> Result<u64> {
        if price_history.is_empty() {
            return Err(FlashLeverageError::InvalidOraclePrice.into());
        }

        let current_time = Clock::get()?.unix_timestamp;
        let start_time = current_time - time_window_seconds;

        let mut weighted_sum = 0u128;
        let mut total_time = 0i64;

        for i in 0..price_history.len() {
            let current = &price_history[i];
            
            if current.timestamp < start_time {
                continue;
            }

            let time_weight = if i == price_history.len() - 1 {
                current_time - current.timestamp
            } else {
                price_history[i + 1].timestamp - current.timestamp
            };

            if time_weight <= 0 {
                continue;
            }

            weighted_sum = weighted_sum
                .checked_add((current.price as u128) * (time_weight as u128))
                .ok_or(FlashLeverageError::MathOverflow)?;
            
            total_time = total_time
                .checked_add(time_weight)
                .ok_or(FlashLeverageError::MathOverflow)?;
        }

        if total_time == 0 {
            return Err(FlashLeverageError::InvalidOraclePrice.into());
        }

        let twap = (weighted_sum / total_time as u128) as u64;
        Ok(twap)
    }

    /// Validate price feed health
    pub fn validate_price_feed(
        oracle_price: &OraclePrice,
        max_staleness_seconds: i64,
        max_confidence_bps: u16,
    ) -> Result<()> {
        // Check staleness
        let current_time = Clock::get()?.unix_timestamp;
        if current_time - oracle_price.last_updated > max_staleness_seconds {
            return Err(FlashLeverageError::OraclePriceTooStale.into());
        }

        // Check confidence interval
        let confidence_bps = (oracle_price.confidence as u128 * 10000) / oracle_price.price as u128;
        if confidence_bps > max_confidence_bps as u128 {
            return Err(FlashLeverageError::InvalidOraclePrice.into());
        }

        // Check price status
        if oracle_price.status != PriceStatus::Valid {
            return Err(FlashLeverageError::InvalidOraclePrice.into());
        }

        // Check for zero or negative price
        if oracle_price.price == 0 {
            return Err(FlashLeverageError::InvalidOraclePrice.into());
        }

        Ok(())
    }

    // Helper functions

    fn get_oracle_weight(provider: OracleProvider) -> u64 {
        match provider {
            OracleProvider::Pyth => 100,        // Highest weight
            OracleProvider::Switchboard => 80,  // High weight
            OracleProvider::Chainlink => 90,    // High weight
            OracleProvider::Custom(_) => 50,    // Lower weight for custom oracles
        }
    }

    fn calculate_confidence(prices: &[OraclePrice]) -> Result<u64> {
        if prices.is_empty() {
            return Ok(0);
        }

        let mut total_confidence = 0u128;
        let mut total_weight = 0u64;

        for price in prices {
            let weight = Self::get_oracle_weight(
                // This would need to be passed in or stored
                OracleProvider::Pyth // Placeholder
            );
            
            total_confidence = total_confidence
                .checked_add((price.confidence as u128) * (weight as u128))
                .ok_or(FlashLeverageError::MathOverflow)?;
            
            total_weight = total_weight
                .checked_add(weight)
                .ok_or(FlashLeverageError::MathOverflow)?;
        }

        let weighted_confidence = (total_confidence / total_weight as u128) as u64;
        Ok(weighted_confidence)
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum OracleProvider {
    Pyth,
    Switchboard,
    Chainlink,
    Custom(Pubkey),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PriceStatus {
    Valid,
    Stale,
    Invalid,
    Unavailable,
}

#[derive(Clone, Debug)]
pub struct OraclePrice {
    pub price: u64,           // Price scaled by 10^decimals
    pub confidence: u64,      // Confidence interval scaled by 10^decimals
    pub last_updated: i64,    // Unix timestamp
    pub decimals: u8,         // Number of decimal places
    pub status: PriceStatus,
}

#[derive(Clone, Debug)]
pub struct AggregatedPrice {
    pub price: u64,           // Weighted average price
    pub confidence: u64,      // Aggregated confidence
    pub last_updated: i64,    // Latest update timestamp
    pub num_oracles: u8,      // Number of oracles used
    pub deviation_bps: u16,   // Price deviation in basis points
    pub status: PriceStatus,
}

#[derive(Clone, Debug)]
pub struct HistoricalPrice {
    pub price: u64,
    pub timestamp: i64,
    pub volume: u64,
}

/// Price validation configuration
#[derive(Clone, Debug)]
pub struct PriceValidationConfig {
    pub max_staleness_seconds: i64,
    pub max_confidence_bps: u16,
    pub max_deviation_bps: u16,
    pub min_oracles: u8,
    pub required_providers: Vec<OracleProvider>,
}