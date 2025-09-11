use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint, MintTo};

/**
 * Модуль для реализации покупки токенов на заложенные средства
 * 
 * Ключевая идея: Поскольку мы контролируем кастомный токен,
 * мы можем минтить его в обмен на залог по нашим правилам
 */

#[derive(Accounts)]
pub struct CollateralSwap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// Lending pool для размещения залога
    #[account(mut)]
    pub lending_pool: Account<'info, LendingPool>,
    
    /// USDC аккаунт пользователя (залог)
    #[account(mut)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    
    /// Кастомный токен аккаунт пользователя
    #[account(mut)]
    pub user_custom_token_account: Account<'info, TokenAccount>,
    
    /// Минт кастомного токена
    #[account(mut)]
    pub custom_token_mint: Account<'info, Mint>,
    
    /// Authority для минтинга кастомного токена
    /// CHECK: Проверяется через seeds
    pub mint_authority: AccountInfo<'info>,
    
    /// Хранилище залога в lending pool
    #[account(mut)]
    pub collateral_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub lending_pool: Account<'info, LendingPool>,
    
    #[account(mut)]
    pub user_usdc_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_custom_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub custom_token_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub collateral_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct LendingPool {
    pub authority: Pubkey,
    pub usdc_mint: Pubkey,
    pub custom_token_mint: Pubkey,
    pub collateral_vault: Pubkey,
    pub total_deposited: u64,
    pub total_borrowed: u64,
    pub ltv_ratio: u16,           // В базисных пунктах (6250 = 62.5%)
    pub collateral_ratio: u16,    // Коэффициент обмена залога на токены
}

#[account]
pub struct UserPosition {
    pub user: Pubkey,
    pub collateral_amount: u64,   // Размер залога в USDC
    pub borrowed_amount: u64,     // Размер займа в USDC
    pub custom_tokens: u64,       // Количество полученных кастомных токенов
    pub created_at: i64,
    pub is_active: bool,
}

impl<'info> CollateralSwap<'info> {
    /**
     * Основная функция: размещение залога и получение кастомных токенов
     */
    pub fn swap_collateral_to_tokens(
        ctx: Context<CollateralSwap>,
        collateral_amount: u64,
        min_custom_tokens: u64,
    ) -> Result<()> {
        let lending_pool = &mut ctx.accounts.lending_pool;
        
        // 1. Переводим USDC в залоговое хранилище
        let transfer_cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_usdc_account.to_account_info(),
                to: ctx.accounts.collateral_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(transfer_cpi, collateral_amount)?;
        
        // 2. Рассчитываем количество кастомных токенов за залог
        let custom_tokens_for_collateral = calculate_tokens_for_collateral(
            collateral_amount,
            lending_pool.collateral_ratio
        );
        
        require!(
            custom_tokens_for_collateral >= min_custom_tokens,
            ErrorCode::InsufficientTokenOutput
        );
        
        // 3. Минтим кастомные токены пользователю
        let mint_seeds = &[
            b"mint_authority",
            &[*ctx.bumps.get("mint_authority").unwrap()],
        ];
        let signer = &[&mint_seeds[..]];
        
        let mint_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.custom_token_mint.to_account_info(),
                to: ctx.accounts.user_custom_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer,
        );
        token::mint_to(mint_cpi, custom_tokens_for_collateral)?;
        
        // 4. Обновляем состояние пула
        lending_pool.total_deposited += collateral_amount;
        
        msg!(
            "Swapped {} USDC collateral for {} CUSTOM tokens",
            collateral_amount,
            custom_tokens_for_collateral
        );
        
        Ok(())
    }
    
    /**
     * Получение займа против залога
     */
    pub fn borrow_against_collateral(
        ctx: Context<CollateralSwap>,
        borrow_amount: u64,
    ) -> Result<()> {
        let lending_pool = &ctx.accounts.lending_pool;
        
        // Проверяем, что у пользователя достаточно залога
        let user_collateral = ctx.accounts.collateral_vault.amount;
        let max_borrow = calculate_max_borrow(user_collateral, lending_pool.ltv_ratio);
        
        require!(
            borrow_amount <= max_borrow,
            ErrorCode::ExceedsLTV
        );
        
        // Переводим USDC из пула пользователю
        let pool_seeds = &[
            b"lending_pool",
            lending_pool.usdc_mint.as_ref(),
            &[*ctx.bumps.get("lending_pool").unwrap()],
        ];
        let signer = &[&pool_seeds[..]];
        
        let transfer_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.collateral_vault.to_account_info(),
                to: ctx.accounts.user_usdc_account.to_account_info(),
                authority: ctx.accounts.lending_pool.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_cpi, borrow_amount)?;
        
        msg!("Borrowed {} USDC against collateral", borrow_amount);
        
        Ok(())
    }
}

impl<'info> WithdrawCollateral<'info> {
    /**
     * Вывод залога путем сжигания кастомных токенов
     */
    pub fn withdraw_collateral(
        ctx: Context<WithdrawCollateral>,
        custom_token_amount: u64,
    ) -> Result<()> {
        let lending_pool = &ctx.accounts.lending_pool;
        
        // 1. Сжигаем кастомные токены
        let burn_cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.custom_token_mint.to_account_info(),
                from: ctx.accounts.user_custom_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::burn(burn_cpi, custom_token_amount)?;
        
        // 2. Рассчитываем количество USDC для возврата
        let usdc_amount = calculate_usdc_for_tokens(
            custom_token_amount,
            lending_pool.collateral_ratio
        );
        
        // 3. Переводим USDC из хранилища пользователю
        let pool_seeds = &[
            b"lending_pool",
            lending_pool.usdc_mint.as_ref(),
            &[*ctx.bumps.get("lending_pool").unwrap()],
        ];
        let signer = &[&pool_seeds[..]];
        
        let transfer_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.collateral_vault.to_account_info(),
                to: ctx.accounts.user_usdc_account.to_account_info(),
                authority: ctx.accounts.lending_pool.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_cpi, usdc_amount)?;
        
        // 4. Обновляем состояние пула
        let lending_pool = &mut ctx.accounts.lending_pool;
        lending_pool.total_deposited -= usdc_amount;
        
        msg!(
            "Withdrew {} USDC by burning {} CUSTOM tokens",
            usdc_amount,
            custom_token_amount
        );
        
        Ok(())
    }
}

// Вспомогательные функции для расчетов
fn calculate_tokens_for_collateral(collateral_amount: u64, collateral_ratio: u16) -> u64 {
    collateral_amount
        .checked_mul(collateral_ratio as u64)
        .unwrap()
        .checked_div(10000)
        .unwrap()
}

fn calculate_usdc_for_tokens(token_amount: u64, collateral_ratio: u16) -> u64 {
    token_amount
        .checked_mul(10000)
        .unwrap()
        .checked_div(collateral_ratio as u64)
        .unwrap()
}

fn calculate_max_borrow(collateral_amount: u64, ltv_ratio: u16) -> u64 {
    collateral_amount
        .checked_mul(ltv_ratio as u64)
        .unwrap()
        .checked_div(10000)
        .unwrap()
}

// Коды ошибок
#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient token output for the given collateral")]
    InsufficientTokenOutput,
    #[msg("Borrow amount exceeds LTV ratio")]
    ExceedsLTV,
    #[msg("Insufficient collateral for withdrawal")]
    InsufficientCollateral,
}