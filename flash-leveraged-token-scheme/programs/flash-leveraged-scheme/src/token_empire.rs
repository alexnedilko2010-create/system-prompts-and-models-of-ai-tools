use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint, MintTo, Burn};

/**
 * Реализация токенной империи - экосистемы взаимосвязанных токенов
 * 
 * Цель: Создать token ecosystem стоимостью $500M-10B+
 * Подход: Множественные токены с cross-utility и network effects
 */

#[derive(Accounts)]
pub struct InitializeTokenEmpire<'info> {
    #[account(
        init,
        payer = founder,
        space = 8 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 1,
        seeds = [b"token_empire"],
        bump,
    )]
    pub token_empire: Account<'info, TokenEmpire>,
    
    #[account(mut)]
    pub founder: Signer<'info>,
    
    /// Основной governance токен
    #[account(
        init,
        payer = founder,
        mint::decimals = 9,
        mint::authority = token_empire,
    )]
    pub empire_mint: Account<'info, Mint>,
    
    /// Utility токены
    #[account(
        init,
        payer = founder,
        mint::decimals = 9,
        mint::authority = token_empire,
    )]
    pub defi_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = founder,
        mint::decimals = 9,
        mint::authority = token_empire,
    )]
    pub game_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = founder,
        mint::decimals = 9,
        mint::authority = token_empire,
    )]
    pub ai_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct StakeForUtility<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"token_empire"],
        bump = token_empire.bump,
    )]
    pub token_empire: Account<'info, TokenEmpire>,
    
    #[account(
        init_if_needed,
        payer = staker,
        space = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 8,
        seeds = [b"staker_position", staker.key().as_ref()],
        bump,
    )]
    pub staker_position: Account<'info, StakerPosition>,
    
    #[account(mut)]
    pub staker_empire_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub staker_utility_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub empire_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub utility_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CrossTokenYieldFarming<'info> {
    #[account(mut)]
    pub farmer: Signer<'info>,
    
    #[account(mut)]
    pub token_empire: Account<'info, TokenEmpire>,
    
    #[account(
        init_if_needed,
        payer = farmer,
        space = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8,
        seeds = [b"yield_farmer", farmer.key().as_ref()],
        bump,
    )]
    pub yield_farmer: Account<'info, YieldFarmer>,
    
    #[account(mut)]
    pub farmer_empire_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub farmer_defi_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub farmer_game_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub farmer_ai_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Структуры данных
#[account]
pub struct TokenEmpire {
    pub founder: Pubkey,
    pub empire_mint: Pubkey,
    pub defi_mint: Pubkey,
    pub game_mint: Pubkey,
    pub ai_mint: Pubkey,
    pub total_staked_empire: u64,
    pub total_ecosystem_fees: u64,
    pub empire_price_oracle: u64,      // Цена в micro-USDC
    pub total_users: u64,
    pub bump: u8,
}

#[account]
pub struct StakerPosition {
    pub staker: Pubkey,
    pub staked_empire: u64,
    pub earned_defi: u64,
    pub earned_game: u64,
    pub earned_ai: u64,
    pub last_claim: i64,
    pub total_rewards_claimed: u64,
    pub referral_rewards: u64,
}

#[account]
pub struct YieldFarmer {
    pub farmer: Pubkey,
    pub empire_staked: u64,
    pub defi_staked: u64,
    pub game_staked: u64,
    pub ai_staked: u64,
    pub combo_multiplier: u16,         // Bonus за комбинированный стейкинг
    pub total_yield_earned: u64,
    pub farmer_level: u16,             // Level system для gamification
    pub last_harvest: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum UtilityTokenType {
    DEFI,
    GAME, 
    AI,
    REAL,
}

impl<'info> TokenEmpire {
    /// Инициализация токенной империи
    pub fn initialize_empire(
        ctx: Context<InitializeTokenEmpire>,
        empire_supply: u64,
        bump: u8,
    ) -> Result<()> {
        let empire = &mut ctx.accounts.token_empire;
        empire.founder = ctx.accounts.founder.key();
        empire.empire_mint = ctx.accounts.empire_mint.key();
        empire.defi_mint = ctx.accounts.defi_mint.key();
        empire.game_mint = ctx.accounts.game_mint.key();
        empire.ai_mint = ctx.accounts.ai_mint.key();
        empire.total_staked_empire = 0;
        empire.total_ecosystem_fees = 0;
        empire.empire_price_oracle = 1_000_000; // $1.00 начальная цена
        empire.total_users = 0;
        empire.bump = bump;
        
        // Минтим начальный supply EMPIRE токенов
        let empire_seeds = &[b"token_empire", &[bump]];
        let signer = &[&empire_seeds[..]];
        
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.empire_mint.to_account_info(),
                    to: ctx.accounts.founder.to_account_info(), // Нужен token account
                    authority: ctx.accounts.token_empire.to_account_info(),
                },
                signer,
            ),
            empire_supply,
        )?;
        
        msg!("👑 TOKEN EMPIRE INITIALIZED!");
        msg!("EMPIRE supply: {}", empire_supply);
        msg!("Founder: {}", ctx.accounts.founder.key());
        
        Ok(())
    }
    
    /// Стейкинг EMPIRE для получения utility токенов
    pub fn stake_empire_for_utility(
        ctx: Context<StakeForUtility>,
        empire_amount: u64,
        utility_type: UtilityTokenType,
    ) -> Result<()> {
        let empire = &mut ctx.accounts.token_empire;
        let position = &mut ctx.accounts.staker_position;
        
        // Переводим EMPIRE токены в стейкинг
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.staker_empire_account.to_account_info(),
                    to: ctx.accounts.token_empire.to_account_info(), // Нужен empire vault
                    authority: ctx.accounts.staker.to_account_info(),
                },
            ),
            empire_amount,
        )?;
        
        // Рассчитываем utility токены
        let utility_multiplier = match utility_type {
            UtilityTokenType::DEFI => 100,  // 1 EMP = 100 DEFI/месяц
            UtilityTokenType::GAME => 1000, // 1 EMP = 1000 GAME/месяц
            UtilityTokenType::AI => 10,     // 1 EMP = 10 AI/месяц
            UtilityTokenType::REAL => 1,    // 1 EMP = 1 REAL/месяц
        };
        
        let utility_rewards = empire_amount * utility_multiplier;
        
        // Минтим utility токены
        let empire_seeds = &[b"token_empire", &[empire.bump]];
        let signer = &[&empire_seeds[..]];
        
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.utility_mint.to_account_info(),
                    to: ctx.accounts.staker_utility_account.to_account_info(),
                    authority: ctx.accounts.token_empire.to_account_info(),
                },
                signer,
            ),
            utility_rewards,
        )?;
        
        // Обновляем позицию стейкера
        position.staker = ctx.accounts.staker.key();
        position.staked_empire += empire_amount;
        position.last_claim = Clock::get()?.unix_timestamp;
        
        match utility_type {
            UtilityTokenType::DEFI => position.earned_defi += utility_rewards,
            UtilityTokenType::GAME => position.earned_game += utility_rewards,
            UtilityTokenType::AI => position.earned_ai += utility_rewards,
            _ => {}
        }
        
        // Обновляем общую статистику
        empire.total_staked_empire += empire_amount;
        if position.staked_empire == empire_amount {
            empire.total_users += 1; // Новый пользователь
        }
        
        msg!("💎 Staked {} EMPIRE for {} {:?} tokens", empire_amount, utility_rewards, utility_type);
        
        Ok(())
    }
    
    /// Cross-token yield farming с бонусами
    pub fn cross_token_yield_farming(
        ctx: Context<CrossTokenYieldFarming>,
        empire_amount: u64,
        defi_amount: u64,
        game_amount: u64,
        ai_amount: u64,
    ) -> Result<()> {
        let empire = &mut ctx.accounts.token_empire;
        let farmer = &mut ctx.accounts.yield_farmer;
        
        // Рассчитываем combo bonus
        let combo_multiplier = Self::calculate_combo_multiplier(
            empire_amount,
            defi_amount, 
            game_amount,
            ai_amount
        )?;
        
        // Базовый yield
        let base_yield = (empire_amount + defi_amount + game_amount + ai_amount) * 12 / 100; // 12% APY
        
        // Применяем combo bonus
        let total_yield = base_yield * combo_multiplier as u64 / 100;
        
        // Обновляем farmer position
        farmer.farmer = ctx.accounts.farmer.key();
        farmer.empire_staked += empire_amount;
        farmer.defi_staked += defi_amount;
        farmer.game_staked += game_amount;
        farmer.ai_staked += ai_amount;
        farmer.combo_multiplier = combo_multiplier;
        farmer.total_yield_earned += total_yield;
        farmer.last_harvest = Clock::get()?.unix_timestamp;
        
        // Увеличиваем farmer level
        farmer.farmer_level = Self::calculate_farmer_level(farmer.total_yield_earned)?;
        
        msg!("🌾 Cross-token yield farming activated!");
        msg!("Combo multiplier: {}%, Total yield: {}", combo_multiplier, total_yield);
        msg!("Farmer level: {}", farmer.farmer_level);
        
        Ok(())
    }
    
    /// Burn utility токенов для EMP buyback (deflationary mechanism)
    pub fn burn_utility_for_empire_buyback(
        ctx: Context<BurnForBuyback>,
        utility_type: UtilityTokenType,
        burn_amount: u64,
    ) -> Result<()> {
        let empire = &mut ctx.accounts.token_empire;
        
        // Сжигаем utility токены
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.utility_mint.to_account_info(),
                    from: ctx.accounts.user_utility_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            burn_amount,
        )?;
        
        // Рассчитываем EMP buyback
        let buyback_rate = match utility_type {
            UtilityTokenType::DEFI => 1,    // 100 DEFI = 1 EMP buyback
            UtilityTokenType::GAME => 10,   // 1000 GAME = 1 EMP buyback  
            UtilityTokenType::AI => 100,    // 10 AI = 1 EMP buyback
            UtilityTokenType::REAL => 1000, // 1 REAL = 1 EMP buyback
        };
        
        let empire_buyback = burn_amount / buyback_rate;
        
        // Покупаем EMP с рынка (имитация)
        let buyback_cost = empire_buyback * empire.empire_price_oracle / 1_000_000;
        
        // Сжигаем купленные EMP (deflationary!)
        token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.empire_mint.to_account_info(),
                    from: ctx.accounts.empire_treasury_account.to_account_info(),
                    authority: ctx.accounts.token_empire.to_account_info(),
                },
                &[&[b"token_empire", &[empire.bump]]],
            ),
            empire_buyback,
        )?;
        
        msg!("🔥 Burned {} {:?} tokens, bought back and burned {} EMPIRE", 
             burn_amount, utility_type, empire_buyback);
        
        // Deflationary pressure увеличивает цену EMP
        let price_impact = empire_buyback * 1000 / empire.total_staked_empire; // Примерный расчет
        empire.empire_price_oracle += price_impact;
        
        msg!("📈 EMPIRE price increased to ${}", empire.empire_price_oracle as f64 / 1_000_000.0);
        
        Ok(())
    }
    
    /// Распределение ecosystem fees между стейкерами
    pub fn distribute_ecosystem_fees(
        ctx: Context<FeeDistribution>,
        total_fees: u64,
    ) -> Result<()> {
        let empire = &mut ctx.accounts.token_empire;
        
        // 50% fees → EMP stakers
        let staker_rewards = total_fees * 50 / 100;
        
        // 30% → EMP buyback and burn  
        let buyback_amount = total_fees * 30 / 100;
        
        // 20% → Treasury для развития
        let treasury_amount = total_fees * 20 / 100;
        
        // Обновляем статистику
        empire.total_ecosystem_fees += total_fees;
        
        msg!("💰 Ecosystem fees distributed: {} total", total_fees);
        msg!("Stakers: {}, Buyback: {}, Treasury: {}", 
             staker_rewards, buyback_amount, treasury_amount);
        
        // Buyback создает deflationary pressure
        let price_increase = buyback_amount * 1000 / empire.total_staked_empire;
        empire.empire_price_oracle += price_increase;
        
        Ok(())
    }
    
    /// Расчет combo multiplier для cross-token staking
    fn calculate_combo_multiplier(
        empire: u64,
        defi: u64, 
        game: u64,
        ai: u64,
    ) -> Result<u16> {
        let mut multiplier = 100; // Базовый 100%
        
        // Bonus за каждый тип токена
        if empire > 0 { multiplier += 20; } // +20% за EMPIRE
        if defi > 0 { multiplier += 15; }   // +15% за DEFI
        if game > 0 { multiplier += 10; }   // +10% за GAME  
        if ai > 0 { multiplier += 25; }     // +25% за AI
        
        // Bonus за balanced portfolio
        let total = empire + defi + game + ai;
        if total > 0 {
            let balance_score = Self::calculate_balance_score(empire, defi, game, ai, total)?;
            multiplier += balance_score; // До +50% за баланс
        }
        
        // Maximum 300% (3x multiplier)
        Ok(multiplier.min(300))
    }
    
    fn calculate_balance_score(empire: u64, defi: u64, game: u64, ai: u64, total: u64) -> Result<u16> {
        // Идеальный баланс: 40% EMPIRE, 20% DEFI, 20% GAME, 20% AI
        let empire_ratio = empire * 100 / total;
        let defi_ratio = defi * 100 / total;
        let game_ratio = game * 100 / total;
        let ai_ratio = ai * 100 / total;
        
        let empire_deviation = (empire_ratio as i64 - 40).abs() as u64;
        let defi_deviation = (defi_ratio as i64 - 20).abs() as u64;
        let game_deviation = (game_ratio as i64 - 20).abs() as u64;
        let ai_deviation = (ai_ratio as i64 - 20).abs() as u64;
        
        let total_deviation = empire_deviation + defi_deviation + game_deviation + ai_deviation;
        
        // Чем меньше deviation, тем больше bonus (max 50%)
        let balance_bonus = 50 - (total_deviation * 50 / 200).min(50);
        
        Ok(balance_bonus as u16)
    }
    
    fn calculate_farmer_level(total_yield_earned: u64) -> Result<u16> {
        // Level system: каждые 10,000 earned = +1 level
        let level = (total_yield_earned / 10_000_000_000).min(100) as u16; // Max level 100
        Ok(level)
    }
    
    /// Viral referral system
    pub fn process_referral_rewards(
        ctx: Context<ProcessReferrals>,
        new_user_stake: u64,
        referrer_levels: Vec<Pubkey>, // До 5 уровней рефералов
    ) -> Result<()> {
        let empire = &ctx.accounts.token_empire;
        
        let referral_rates = [100, 50, 25, 12, 12]; // 1%, 0.5%, 0.25%, 0.125%, 0.125%
        
        for (level, referrer) in referrer_levels.iter().enumerate() {
            if level >= referral_rates.len() {
                break;
            }
            
            let referral_reward = new_user_stake * referral_rates[level] / 10000;
            
            // Минтим referral rewards
            let empire_seeds = &[b"token_empire", &[empire.bump]];
            let signer = &[&empire_seeds[..]];
            
            // В реальности здесь был бы transfer на аккаунт реферера
            msg!("💰 Referral level {}: {} EMPIRE to {}", level + 1, referral_reward, referrer);
        }
        
        Ok(())
    }
}

#[error_code]
pub enum TokenEmpireError {
    #[msg("Invalid utility token type")]
    InvalidUtilityTokenType,
    #[msg("Insufficient staked amount")]
    InsufficientStakedAmount,
    #[msg("Combo multiplier calculation failed")]
    ComboCalculationFailed,
    #[msg("Farmer level calculation failed")]
    LevelCalculationFailed,
    #[msg("Too many referral levels (max 5)")]
    TooManyReferralLevels,
}