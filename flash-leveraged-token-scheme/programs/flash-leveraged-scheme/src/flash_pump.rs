use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint, MintTo, Burn};

/**
 * Реализация Flash Pump Creation механизмов
 * 
 * ВНИМАНИЕ: Данный код предназначен ИСКЛЮЧИТЕЛЬНО для образовательных целей!
 * Pump and dump схемы могут нарушать законодательство!
 * Используйте только для изучения механизмов и создания защит!
 */

#[derive(Accounts)]
pub struct InitializePumpToken<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 2 + 2 + 1 + 8 + 4 + 1,
        seeds = [b"pump_token", creator.key().as_ref()],
        bump,
    )]
    pub pump_token: Account<'info, PumpTokenData>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        init,
        payer = creator,
        mint::decimals = 9,
        mint::authority = pump_token,
    )]
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = creator,
        token::mint = token_mint,
        token::authority = creator,
    )]
    pub creator_token_account: Account<'info, TokenAccount>,
    
    /// Vault для хранения USDC ликвидности
    #[account(
        init,
        payer = creator,
        token::mint = usdc_mint,
        token::authority = pump_token,
    )]
    pub liquidity_vault: Account<'info, TokenAccount>,
    
    pub usdc_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ExecuteFlashPump<'info> {
    #[account(mut)]
    pub pumper: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pump_token", pump_token.creator.as_ref()],
        bump = pump_token.bump,
    )]
    pub pump_token: Account<'info, PumpTokenData>,
    
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub pumper_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub pumper_usdc_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub liquidity_vault: Account<'info, TokenAccount>,
    
    /// Flash loan pool
    #[account(mut)]
    pub flash_pool: Account<'info, FlashPool>,
    
    #[account(mut)]
    pub flash_vault: Account<'info, TokenAccount>,
    
    /// Loan state для tracking
    #[account(
        init,
        payer = pumper,
        space = 8 + 32 + 8 + 8 + 1,
    )]
    pub loan_state: Account<'info, LoanState>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct TriggerFOMO<'info> {
    #[account(mut)]
    pub fomo_trigger: Signer<'info>,
    
    #[account(mut)]
    pub pump_token: Account<'info, PumpTokenData>,
    
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    
    /// Simulated FOMO buyers accounts
    #[account(mut)]
    pub fomo_buyer_1: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub fomo_buyer_2: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub fomo_buyer_3: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StrategicExit<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        mut,
        has_one = creator,
    )]
    pub pump_token: Account<'info, PumpTokenData>,
    
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub creator_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub creator_usdc_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub liquidity_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

// Структуры данных
#[account]
pub struct PumpTokenData {
    pub creator: Pubkey,
    pub token_mint: Pubkey,
    pub total_supply: u64,
    pub team_allocation: u64,
    pub circulating_supply: u64,
    pub current_price: u64,        // В micro-USDC
    pub pump_phase: PumpPhase,
    pub pump_multiplier: u16,      // Multiplier от начальной цены
    pub fomo_level: u16,           // 0-1000 FOMO intensity
    pub flash_boost_count: u8,     // Количество flash boost'ов
    pub total_volume: u64,
    pub holder_count: u32,
    pub bump: u8,
}

#[account]
pub struct PumpPhaseData {
    pub phase_number: u8,
    pub start_time: i64,
    pub end_time: i64,
    pub start_price: u64,
    pub end_price: u64,
    pub volume_in_phase: u64,
    pub flash_loan_used: u64,
    pub new_holders_attracted: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum PumpPhase {
    PreLaunch,
    InitialBoost,
    Growth,
    FOMO,
    Peak,
    Distribution,
    Decline,
}

impl<'info> PumpTokenData {
    /// Инициализация pump токена
    pub fn initialize_pump_token(
        ctx: Context<InitializePumpToken>,
        total_supply: u64,
        team_percentage: u8,
        initial_price: u64,
        bump: u8,
    ) -> Result<()> {
        let pump_token = &mut ctx.accounts.pump_token;
        
        pump_token.creator = ctx.accounts.creator.key();
        pump_token.token_mint = ctx.accounts.token_mint.key();
        pump_token.total_supply = total_supply;
        pump_token.team_allocation = total_supply * team_percentage as u64 / 100;
        pump_token.circulating_supply = total_supply - pump_token.team_allocation;
        pump_token.current_price = initial_price;
        pump_token.pump_phase = PumpPhase::PreLaunch;
        pump_token.pump_multiplier = 100; // 1x базовая цена
        pump_token.fomo_level = 0;
        pump_token.flash_boost_count = 0;
        pump_token.total_volume = 0;
        pump_token.holder_count = 1;
        pump_token.bump = bump;
        
        // Минтим team allocation создателю
        let pump_seeds = &[
            b"pump_token",
            pump_token.creator.as_ref(),
            &[bump],
        ];
        let signer = &[&pump_seeds[..]];
        
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.token_mint.to_account_info(),
                    to: ctx.accounts.creator_token_account.to_account_info(),
                    authority: ctx.accounts.pump_token.to_account_info(),
                },
                signer,
            ),
            pump_token.team_allocation,
        )?;
        
        // Минтим circulating supply в liquidity vault
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.token_mint.to_account_info(),
                    to: ctx.accounts.liquidity_vault.to_account_info(),
                    authority: ctx.accounts.pump_token.to_account_info(),
                },
                signer,
            ),
            pump_token.circulating_supply,
        )?;
        
        msg!("🚀 PUMP TOKEN INITIALIZED!");
        msg!("Supply: {}, Team: {}, Price: ${}", 
             total_supply, pump_token.team_allocation, initial_price as f64 / 1_000_000.0);
        
        Ok(())
    }
    
    /// Выполнение flash pump boost
    pub fn execute_flash_pump_boost(
        ctx: Context<ExecuteFlashPump>,
        flash_loan_amount: u64,
        target_price_multiplier: u16,
        pump_intensity: u8, // 1-10
    ) -> Result<()> {
        let pump_token = &mut ctx.accounts.pump_token;
        let loan_state = &mut ctx.accounts.loan_state;
        
        require!(
            pump_intensity <= 10,
            FlashPumpError::IntensityTooHigh
        );
        
        require!(
            pump_token.flash_boost_count < 10,
            FlashPumpError::TooManyBoosts
        );
        
        msg!("⚡ EXECUTING FLASH PUMP BOOST #{}", pump_token.flash_boost_count + 1);
        msg!("Flash loan: {}, Target: {}x, Intensity: {}/10",
             flash_loan_amount, target_price_multiplier as f64 / 100.0, pump_intensity);
        
        // Записываем flash loan state
        loan_state.borrower = ctx.accounts.pumper.key();
        loan_state.amount = flash_loan_amount;
        loan_state.fee = flash_loan_amount * 50 / 10000; // 0.5%
        loan_state.is_active = true;
        
        // Рассчитываем buying pressure
        let buying_pressure = flash_loan_amount * pump_intensity as u64 / 10;
        let tokens_to_buy = buying_pressure * 1_000_000 / pump_token.current_price;
        
        // Имитируем покупку токенов (создание pump'а)
        let price_impact = Self::calculate_pump_price_impact(
            tokens_to_buy,
            pump_token.circulating_supply,
            pump_intensity
        )?;
        
        let new_price = pump_token.current_price * (10000 + price_impact) / 10000;
        pump_token.current_price = new_price;
        pump_token.pump_multiplier = new_price * 100 / (pump_token.current_price / pump_token.pump_multiplier as u64 * 100);
        
        // Обновляем pump metrics
        pump_token.total_volume += buying_pressure;
        pump_token.flash_boost_count += 1;
        
        // Увеличиваем FOMO level
        let fomo_increase = pump_intensity as u16 * 50; // 50-500 points
        pump_token.fomo_level = (pump_token.fomo_level + fomo_increase).min(1000);
        
        // Переходим в следующую фазу pump'а
        pump_token.pump_phase = match pump_token.flash_boost_count {
            1 => PumpPhase::InitialBoost,
            2..=3 => PumpPhase::Growth,
            4..=6 => PumpPhase::FOMO,
            _ => PumpPhase::Peak,
        };
        
        // Имитируем привлечение новых holders
        let new_holders = pump_intensity as u32 * pump_token.flash_boost_count as u32 * 10;
        pump_token.holder_count += new_holders;
        
        msg!("📈 PUMP BOOST RESULTS:");
        msg!("Price: ${} → ${} ({}x total multiplier)",
             (pump_token.current_price / (price_impact + 10000) * 10000) as f64 / 1_000_000.0,
             new_price as f64 / 1_000_000.0,
             pump_token.pump_multiplier as f64 / 100.0);
        msg!("Phase: {:?}, FOMO: {}/1000, Holders: {}", 
             pump_token.pump_phase, pump_token.fomo_level, pump_token.holder_count);
        
        Ok(())
    }
    
    /// Trigger FOMO cascade
    pub fn trigger_fomo_cascade(
        ctx: Context<TriggerFOMO>,
        fomo_multiplier: u16, // 150-1000 (1.5x-10x)
        fomo_duration: i64,   // Продолжительность FOMO в секундах
    ) -> Result<()> {
        let pump_token = &mut ctx.accounts.pump_token;
        
        require!(
            pump_token.pump_phase == PumpPhase::Growth || pump_token.pump_phase == PumpPhase::FOMO,
            FlashPumpError::WrongPhaseForFOMO
        );
        
        msg!("🔥 TRIGGERING FOMO CASCADE!");
        msg!("FOMO multiplier: {}x, Duration: {} seconds", 
             fomo_multiplier as f64 / 100.0, fomo_duration);
        
        // FOMO exponentially увеличивает цену
        let fomo_price = pump_token.current_price * fomo_multiplier as u64 / 100;
        pump_token.current_price = fomo_price;
        pump_token.pump_multiplier = pump_token.pump_multiplier * fomo_multiplier / 100;
        
        // FOMO level достигает максимума
        pump_token.fomo_level = 1000; // Maximum FOMO
        pump_token.pump_phase = PumpPhase::Peak;
        
        // Massive holder influx
        let fomo_holder_influx = fomo_multiplier as u32 * 100; // 150-10,000 новых holders
        pump_token.holder_count += fomo_holder_influx;
        
        // Volume explosion
        let fomo_volume = fomo_price * fomo_holder_influx as u64 * 50; // Avg $50 per new holder
        pump_token.total_volume += fomo_volume;
        
        msg!("🚀 FOMO CASCADE RESULTS:");
        msg!("Peak price: ${} ({}x total pump)", 
             fomo_price as f64 / 1_000_000.0, pump_token.pump_multiplier as f64 / 100.0);
        msg!("FOMO holders: {}, FOMO volume: {}", fomo_holder_influx, fomo_volume);
        
        Ok(())
    }
    
    /// Стратегический exit с team allocation
    pub fn execute_strategic_exit(
        ctx: Context<StrategicExit>,
        exit_percentage: u8,  // Процент team allocation для продажи
        min_exit_price: u64,  // Минимальная цена для exit
        exit_strategy: ExitStrategy,
    ) -> Result<()> {
        let pump_token = &mut ctx.accounts.pump_token;
        
        require!(
            pump_token.creator == ctx.accounts.creator.key(),
            FlashPumpError::UnauthorizedExit
        );
        
        require!(
            pump_token.current_price >= min_exit_price,
            FlashPumpError::PriceBelowMinimum
        );
        
        require!(
            exit_percentage <= 100,
            FlashPumpError::InvalidExitPercentage
        );
        
        msg!("💰 EXECUTING STRATEGIC EXIT");
        msg!("Strategy: {:?}, Selling: {}% at ${}", 
             exit_strategy, exit_percentage, pump_token.current_price as f64 / 1_000_000.0);
        
        let tokens_to_sell = pump_token.team_allocation * exit_percentage as u64 / 100;
        
        // Рассчитываем exit value с учетом стратегии
        let exit_value = match exit_strategy {
            ExitStrategy::Immediate => {
                // Мгновенная продажа по текущей цене
                tokens_to_sell * pump_token.current_price / 1_000_000
            },
            ExitStrategy::Gradual => {
                // Постепенная продажа с меньшим price impact
                let avg_price = pump_token.current_price * 90 / 100; // 10% discount для gradual
                tokens_to_sell * avg_price / 1_000_000
            },
            ExitStrategy::Peak => {
                // Ждем peak и продаем по максимальной цене
                let peak_price = pump_token.current_price * 120 / 100; // 20% premium на peak
                tokens_to_sell * peak_price / 1_000_000
            },
        };
        
        // Выполняем продажу (transfer токенов за USDC)
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.creator_token_account.to_account_info(),
                    to: ctx.accounts.liquidity_vault.to_account_info(), // Продаем в liquidity
                    authority: ctx.accounts.creator.to_account_info(),
                },
            ),
            tokens_to_sell,
        )?;
        
        // Получаем USDC за проданные токены
        let pump_seeds = &[
            b"pump_token",
            pump_token.creator.as_ref(),
            &[pump_token.bump],
        ];
        let signer = &[&pump_seeds[..]];
        
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.liquidity_vault.to_account_info(),
                    to: ctx.accounts.creator_usdc_account.to_account_info(),
                    authority: ctx.accounts.pump_token.to_account_info(),
                },
                signer,
            ),
            exit_value,
        )?;
        
        // Обновляем team allocation
        pump_token.team_allocation -= tokens_to_sell;
        
        // Price impact от продажи
        let price_impact = Self::calculate_sell_pressure_impact(tokens_to_sell, exit_strategy)?;
        let new_price = pump_token.current_price * (10000 - price_impact) / 10000;
        pump_token.current_price = new_price;
        
        // Переходим в distribution фазу
        if exit_percentage >= 50 {
            pump_token.pump_phase = PumpPhase::Distribution;
        }
        
        msg!("💸 STRATEGIC EXIT COMPLETED:");
        msg!("Tokens sold: {}, USDC received: {}", tokens_to_sell, exit_value);
        msg!("Price impact: -{}%, New price: ${}", 
             price_impact as f64 / 100.0, new_price as f64 / 1_000_000.0);
        
        let roi = exit_value * 100 / 10000; // Assuming $10k initial investment
        msg!("🎉 EXIT ROI: {}x", roi);
        
        Ok(())
    }
    
    /// Расчет price impact от pump'а
    fn calculate_pump_price_impact(
        tokens_bought: u64,
        circulating_supply: u64,
        intensity: u8,
    ) -> Result<u16> {
        // Price impact зависит от размера покупки и интенсивности
        let buy_ratio = tokens_bought * 10000 / circulating_supply;
        let intensity_multiplier = intensity as u16 * 10; // 10-100 multiplier
        
        let base_impact = buy_ratio * intensity_multiplier / 100;
        let price_impact = base_impact.min(9000) as u16; // Max 90% price increase per boost
        
        Ok(price_impact)
    }
    
    fn calculate_sell_pressure_impact(
        tokens_sold: u64,
        exit_strategy: ExitStrategy,
    ) -> Result<u16> {
        // Price impact от продажи зависит от стратегии
        let base_impact = tokens_sold / 10000; // Базовый impact
        
        let strategy_multiplier = match exit_strategy {
            ExitStrategy::Immediate => 200, // 2x impact для immediate dump
            ExitStrategy::Gradual => 50,    // 0.5x impact для gradual exit
            ExitStrategy::Peak => 100,      // 1x impact для peak exit
        };
        
        let total_impact = (base_impact * strategy_multiplier / 100).min(5000) as u16; // Max 50% impact
        
        Ok(total_impact)
    }
    
    /// Анализ optimal timing для pump phases
    pub fn analyze_optimal_pump_timing(
        ctx: Context<AnalyzeTiming>,
    ) -> Result<PumpTimingAnalysis> {
        let pump_token = &ctx.accounts.pump_token;
        
        let analysis = PumpTimingAnalysis {
            current_phase: pump_token.pump_phase,
            optimal_next_boost_time: Clock::get()?.unix_timestamp + 3600, // 1 час
            fomo_readiness_score: pump_token.fomo_level,
            market_conditions_score: 750, // Simplified score
            recommended_flash_size: pump_token.current_price * 50, // 50x current price
            exit_window_start: Clock::get()?.unix_timestamp + 86400, // 24 часа
            exit_window_end: Clock::get()?.unix_timestamp + 604800,   // 7 дней
        };
        
        msg!("📊 PUMP TIMING ANALYSIS:");
        msg!("Phase: {:?}, FOMO readiness: {}/1000", analysis.current_phase, analysis.fomo_readiness_score);
        msg!("Recommended flash size: {}, Exit window: {} - {}", 
             analysis.recommended_flash_size, analysis.exit_window_start, analysis.exit_window_end);
        
        Ok(analysis)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum ExitStrategy {
    Immediate,  // Мгновенная продажа (высокий impact)
    Gradual,    // Постепенная продажа (низкий impact)
    Peak,       // Ждем peak и продаем (optimal timing)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PumpTimingAnalysis {
    pub current_phase: PumpPhase,
    pub optimal_next_boost_time: i64,
    pub fomo_readiness_score: u16,
    pub market_conditions_score: u16,
    pub recommended_flash_size: u64,
    pub exit_window_start: i64,
    pub exit_window_end: i64,
}

#[error_code]
pub enum FlashPumpError {
    #[msg("Pump intensity too high (max 10)")]
    IntensityTooHigh,
    #[msg("Too many flash boosts (max 10)")]
    TooManyBoosts,
    #[msg("Wrong phase for FOMO trigger")]
    WrongPhaseForFOMO,
    #[msg("Unauthorized exit attempt")]
    UnauthorizedExit,
    #[msg("Price below minimum exit threshold")]
    PriceBelowMinimum,
    #[msg("Invalid exit percentage")]
    InvalidExitPercentage,
    #[msg("Flash loan amount too large")]
    FlashLoanTooLarge,
    #[msg("Pump token not ready for boost")]
    NotReadyForBoost,
}