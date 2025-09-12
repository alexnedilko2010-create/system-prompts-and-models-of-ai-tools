use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint, MintTo};

/**
 * Реализация стратегий yield farming с использованием флеш-займов
 * 
 * Легальные и прибыльные стратегии для заработка в DeFi
 */

#[derive(Accounts)]
pub struct InitializeYieldFarming<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 2 + 1,
        seeds = [b"yield_farming", base_token_mint.key().as_ref()],
        bump,
    )]
    pub farming_pool: Account<'info, YieldFarmingPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub base_token_mint: Account<'info, Mint>,
    pub reward_token_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        token::mint = base_token_mint,
        token::authority = farming_pool,
    )]
    pub vault: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = authority,
        token::mint = reward_token_mint,
        token::authority = farming_pool,
    )]
    pub reward_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct FlashYieldFarming<'info> {
    #[account(mut)]
    pub farmer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"yield_farming", farming_pool.base_token_mint.as_ref()],
        bump = farming_pool.bump,
    )]
    pub farming_pool: Account<'info, YieldFarmingPool>,
    
    #[account(
        init_if_needed,
        payer = farmer,
        space = 8 + 32 + 8 + 8 + 8 + 8,
        seeds = [b"farmer_position", farmer.key().as_ref(), farming_pool.key().as_ref()],
        bump,
    )]
    pub farmer_position: Account<'info, FarmerPosition>,
    
    /// Flash loan pool для получения займа
    #[account(mut)]
    pub flash_pool: Account<'info, FlashPool>,
    
    #[account(mut)]
    pub farmer_base_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub farmer_reward_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    
    /// Loan state для отслеживания флеш-займа
    #[account(
        init,
        payer = farmer,
        space = 8 + 32 + 8 + 8 + 1,
    )]
    pub loan_state: Account<'info, LoanState>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ArbitrageYieldRates<'info> {
    #[account(mut)]
    pub arbitrageur: Signer<'info>,
    
    /// Протокол с высокой ставкой supply
    #[account(mut)]
    pub high_yield_pool: Account<'info, YieldFarmingPool>,
    
    /// Протокол с низкой ставкой borrow
    #[account(mut)]
    pub low_borrow_pool: Account<'info, YieldFarmingPool>,
    
    #[account(
        init_if_needed,
        payer = arbitrageur,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8,
        seeds = [b"arbitrage_position", arbitrageur.key().as_ref()],
        bump,
    )]
    pub arbitrage_position: Account<'info, ArbitragePosition>,
    
    #[account(mut)]
    pub arbitrageur_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub high_yield_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub low_borrow_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CompoundYieldStrategy<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub compound_pool: Account<'info, YieldFarmingPool>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 8 + 8 + 8 + 1,
        seeds = [b"compound_position", user.key().as_ref(), compound_pool.key().as_ref()],
        bump,
    )]
    pub compound_position: Account<'info, CompoundPosition>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_ctoken_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub compound_vault: Account<'info, TokenAccount>,
    
    pub compound_token_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Структуры данных
#[account]
pub struct YieldFarmingPool {
    pub authority: Pubkey,
    pub base_token_mint: Pubkey,
    pub reward_token_mint: Pubkey,
    pub vault: Pubkey,
    pub reward_vault: Pubkey,
    pub total_staked: u64,
    pub reward_rate: u64,        // Rewards per second
    pub apy: u16,               // APY in basis points (500 = 5%)
    pub bump: u8,
}

#[account]
pub struct FarmerPosition {
    pub farmer: Pubkey,
    pub staked_amount: u64,
    pub reward_debt: u64,
    pub last_harvest: i64,
    pub total_rewards_earned: u64,
    pub flash_loan_used: u64,
}

#[account]
pub struct ArbitragePosition {
    pub arbitrageur: Pubkey,
    pub high_yield_pool: Pubkey,
    pub low_borrow_pool: Pubkey,
    pub supply_amount: u64,
    pub borrow_amount: u64,
    pub expected_spread: u64,     // Expected profit in basis points
    pub actual_spread: u64,       // Actual profit earned
}

#[account]
pub struct CompoundPosition {
    pub user: Pubkey,
    pub supplied_amount: u64,
    pub compound_tokens: u64,
    pub leverage_multiplier: u8,  // 1x, 2x, 3x, etc.
    pub effective_apy: u16,       // Effective APY after compounding
}

impl<'info> YieldFarmingPool {
    /// Инициализация yield farming pool
    pub fn initialize_farming_pool(
        ctx: Context<InitializeYieldFarming>,
        reward_rate: u64,
        apy: u16,
        bump: u8,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.farming_pool;
        pool.authority = ctx.accounts.authority.key();
        pool.base_token_mint = ctx.accounts.base_token_mint.key();
        pool.reward_token_mint = ctx.accounts.reward_token_mint.key();
        pool.vault = ctx.accounts.vault.key();
        pool.reward_vault = ctx.accounts.reward_vault.key();
        pool.total_staked = 0;
        pool.reward_rate = reward_rate;
        pool.apy = apy;
        pool.bump = bump;
        
        msg!("Yield farming pool initialized with {}% APY", apy as f64 / 100.0);
        Ok(())
    }
    
    /// 🚀 СТРАТЕГИЯ 1: Flash Loan Yield Farming
    pub fn execute_flash_yield_farming(
        ctx: Context<FlashYieldFarming>,
        flash_loan_amount: u64,
        farming_duration: i64, // В секундах
    ) -> Result<()> {
        msg!("🚀 EXECUTING FLASH LOAN YIELD FARMING");
        msg!("Flash loan amount: {}, Duration: {} seconds", flash_loan_amount, farming_duration);
        
        let farming_pool = &mut ctx.accounts.farming_pool;
        let farmer_position = &mut ctx.accounts.farmer_position;
        let loan_state = &mut ctx.accounts.loan_state;
        
        // Этап 1: Записываем информацию о флеш-займе
        loan_state.borrower = ctx.accounts.farmer.key();
        loan_state.amount = flash_loan_amount;
        loan_state.fee = flash_loan_amount * 50 / 10000; // 0.5% fee
        loan_state.is_active = true;
        
        // Этап 2: Размещаем флеш-займ в farming
        let stake_cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.farmer_base_token_account.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.farmer.to_account_info(),
            },
        );
        token::transfer(stake_cpi, flash_loan_amount)?;
        
        // Обновляем позицию фермера
        farmer_position.farmer = ctx.accounts.farmer.key();
        farmer_position.staked_amount += flash_loan_amount;
        farmer_position.last_harvest = Clock::get()?.unix_timestamp;
        farmer_position.flash_loan_used = flash_loan_amount;
        
        // Обновляем пул
        farming_pool.total_staked += flash_loan_amount;
        
        // Этап 3: Рассчитываем мгновенные rewards
        let instant_rewards = Self::calculate_instant_rewards(
            flash_loan_amount,
            farming_pool.reward_rate,
            farming_duration
        )?;
        
        // Этап 4: Минтим rewards фермеру
        let pool_seeds = &[
            b"yield_farming",
            farming_pool.base_token_mint.as_ref(),
            &[farming_pool.bump],
        ];
        let signer = &[&pool_seeds[..]];
        
        let mint_rewards_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.farming_pool.reward_token_mint.to_account_info(),
                to: ctx.accounts.farmer_reward_token_account.to_account_info(),
                authority: ctx.accounts.farming_pool.to_account_info(),
            },
            signer,
        );
        token::mint_to(mint_rewards_cpi, instant_rewards)?;
        
        // Этап 5: Выводим изначальную сумму из farming
        let unstake_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.farmer_base_token_account.to_account_info(),
                authority: ctx.accounts.farming_pool.to_account_info(),
            },
            signer,
        );
        token::transfer(unstake_cpi, flash_loan_amount)?;
        
        // Обновляем статистику
        farmer_position.total_rewards_earned += instant_rewards;
        farmer_position.staked_amount = 0;
        farming_pool.total_staked -= flash_loan_amount;
        
        msg!("✅ Flash yield farming completed!");
        msg!("Earned {} reward tokens in {} seconds", instant_rewards, farming_duration);
        
        // Примечание: Возврат флеш-займа происходит в отдельной функции
        Ok(())
    }
    
    /// 📊 СТРАТЕГИЯ 2: Yield Rate Arbitrage
    pub fn execute_yield_arbitrage(
        ctx: Context<ArbitrageYieldRates>,
        arbitrage_amount: u64,
    ) -> Result<()> {
        msg!("📊 EXECUTING YIELD RATE ARBITRAGE");
        
        let high_pool = &ctx.accounts.high_yield_pool;
        let low_pool = &ctx.accounts.low_borrow_pool;
        let position = &mut ctx.accounts.arbitrage_position;
        
        // Рассчитываем spread между протоколами
        let yield_spread = high_pool.apy.saturating_sub(low_pool.apy);
        
        require!(
            yield_spread > 100, // Минимум 1% spread
            YieldFarmingError::InsufficientSpread
        );
        
        // Записываем позицию арбитража
        position.arbitrageur = ctx.accounts.arbitrageur.key();
        position.high_yield_pool = high_pool.key();
        position.low_borrow_pool = low_pool.key();
        position.supply_amount = arbitrage_amount;
        position.borrow_amount = arbitrage_amount * 80 / 100; // 80% LTV
        position.expected_spread = yield_spread as u64;
        
        msg!("Arbitrage setup: {}% high yield vs {}% low borrow = {}% spread",
             high_pool.apy as f64 / 100.0,
             low_pool.apy as f64 / 100.0,
             yield_spread as f64 / 100.0);
        
        // Этап 1: Supply в высокодоходный протокол
        let supply_cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.arbitrageur_token_account.to_account_info(),
                to: ctx.accounts.high_yield_vault.to_account_info(),
                authority: ctx.accounts.arbitrageur.to_account_info(),
            },
        );
        token::transfer(supply_cpi, arbitrage_amount)?;
        
        // Этап 2: Borrow из низкодоходного протокола
        let high_pool_seeds = &[
            b"yield_farming",
            high_pool.base_token_mint.as_ref(),
            &[high_pool.bump],
        ];
        let high_signer = &[&high_pool_seeds[..]];
        
        let borrow_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.low_borrow_vault.to_account_info(),
                to: ctx.accounts.arbitrageur_token_account.to_account_info(),
                authority: ctx.accounts.low_borrow_pool.to_account_info(),
            },
            high_signer,
        );
        token::transfer(borrow_cpi, position.borrow_amount)?;
        
        msg!("✅ Yield arbitrage position opened");
        msg!("Supplied: {} to high yield pool", arbitrage_amount);
        msg!("Borrowed: {} from low yield pool", position.borrow_amount);
        
        Ok(())
    }
    
    /// 🔄 СТРАТЕГИЯ 3: Compound Yield Strategy
    pub fn execute_compound_strategy(
        ctx: Context<CompoundYieldStrategy>,
        initial_amount: u64,
        target_leverage: u8,
    ) -> Result<()> {
        msg!("🔄 EXECUTING COMPOUND YIELD STRATEGY");
        msg!("Initial: {}, Target leverage: {}x", initial_amount, target_leverage);
        
        let compound_pool = &mut ctx.accounts.compound_pool;
        let position = &mut ctx.accounts.compound_position;
        
        require!(
            target_leverage <= 10, // Максимум 10x leverage
            YieldFarmingError::ExcessiveLeverage
        );
        
        // Инициализируем позицию
        position.user = ctx.accounts.user.key();
        position.supplied_amount = initial_amount;
        position.leverage_multiplier = target_leverage;
        
        let mut current_supply = initial_amount;
        let mut total_supplied = initial_amount;
        
        // Цикл компаундинга
        for cycle in 1..target_leverage {
            msg!("Compound cycle {}: supplying {}", cycle, current_supply);
            
            // Supply текущей суммы
            let supply_cpi = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.compound_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            );
            token::transfer(supply_cpi, current_supply)?;
            
            // Получаем compound токены (cTokens)
            let ctoken_amount = current_supply; // 1:1 для простоты
            
            let pool_seeds = &[
                b"yield_farming",
                compound_pool.base_token_mint.as_ref(),
                &[compound_pool.bump],
            ];
            let signer = &[&pool_seeds[..]];
            
            let mint_ctoken_cpi = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.compound_token_mint.to_account_info(),
                    to: ctx.accounts.user_ctoken_account.to_account_info(),
                    authority: ctx.accounts.compound_pool.to_account_info(),
                },
                signer,
            );
            token::mint_to(mint_ctoken_cpi, ctoken_amount)?;
            
            position.compound_tokens += ctoken_amount;
            
            // Занимаем против cTokens для следующего цикла
            if cycle < target_leverage - 1 {
                let borrow_amount = current_supply * 75 / 100; // 75% LTV
                current_supply = borrow_amount;
                total_supplied += borrow_amount;
                
                let borrow_cpi = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.compound_vault.to_account_info(),
                        to: ctx.accounts.user_token_account.to_account_info(),
                        authority: ctx.accounts.compound_pool.to_account_info(),
                    },
                    signer,
                );
                token::transfer(borrow_cpi, borrow_amount)?;
            }
        }
        
        // Рассчитываем эффективный APY
        let base_apy = compound_pool.apy;
        let effective_apy = (base_apy as u32 * target_leverage as u32 * 85 / 100) as u16; // 85% efficiency
        position.effective_apy = effective_apy;
        
        msg!("✅ Compound strategy completed!");
        msg!("Total supplied: {}, Effective APY: {}%", 
             total_supplied, effective_apy as f64 / 100.0);
        msg!("Leverage achieved: {}x", target_leverage);
        
        Ok(())
    }
    
    /// Расчет мгновенных rewards
    fn calculate_instant_rewards(
        staked_amount: u64,
        reward_rate: u64,
        duration: i64,
    ) -> Result<u64> {
        // Формула: rewards = (staked_amount * reward_rate * duration) / total_staked
        // Для простоты предполагаем что мы единственные в пуле на момент операции
        let rewards = staked_amount
            .checked_mul(reward_rate)
            .unwrap()
            .checked_mul(duration as u64)
            .unwrap()
            .checked_div(365 * 24 * 60 * 60) // Приводим к секундам в году
            .unwrap();
        
        Ok(rewards)
    }
    
    /// Расчет прибыльности стратегии
    pub fn calculate_strategy_profitability(
        flash_loan_amount: u64,
        apy: u16,
        duration_hours: u64,
        flash_loan_fee_bps: u16, // В базисных пунктах
    ) -> Result<(u64, bool)> {
        // Доход от farming
        let yearly_income = flash_loan_amount
            .checked_mul(apy as u64)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        
        let hourly_income = yearly_income
            .checked_div(365 * 24)
            .unwrap();
        
        let total_income = hourly_income
            .checked_mul(duration_hours)
            .unwrap();
        
        // Расходы
        let flash_loan_fee = flash_loan_amount
            .checked_mul(flash_loan_fee_bps as u64)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        
        let gas_costs = 50_000; // Примерная стоимость газа в lamports
        
        let total_costs = flash_loan_fee + gas_costs;
        
        // Прибыль
        let is_profitable = total_income > total_costs;
        let net_profit = if is_profitable {
            total_income - total_costs
        } else {
            0
        };
        
        Ok((net_profit, is_profitable))
    }
}

#[error_code]
pub enum YieldFarmingError {
    #[msg("Insufficient yield spread for profitable arbitrage")]
    InsufficientSpread,
    #[msg("Leverage multiplier too high (max 10x)")]
    ExcessiveLeverage,
    #[msg("Strategy not profitable with current parameters")]
    UnprofitableStrategy,
    #[msg("Insufficient liquidity in farming pool")]
    InsufficientLiquidity,
    #[msg("Farming duration too short")]
    DurationTooShort,
}