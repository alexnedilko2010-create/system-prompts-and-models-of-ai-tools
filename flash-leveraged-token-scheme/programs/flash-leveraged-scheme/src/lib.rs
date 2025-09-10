use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use solana_program::program_pack::Pack;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod flash_leveraged_scheme {
    use super::*;

    /// Инициализация программы и создание пула для флеш-займов
    pub fn initialize_flash_pool(
        ctx: Context<InitializeFlashPool>,
        pool_bump: u8,
        amount: u64,
    ) -> Result<()> {
        let flash_pool = &mut ctx.accounts.flash_pool;
        flash_pool.authority = ctx.accounts.authority.key();
        flash_pool.token_mint = ctx.accounts.token_mint.key();
        flash_pool.token_vault = ctx.accounts.token_vault.key();
        flash_pool.fee_rate = 50; // 0.05% комиссия
        flash_pool.bump = pool_bump;
        flash_pool.total_liquidity = amount;

        // Переводим начальную ликвидность в пул
        let cpi_accounts = Transfer {
            from: ctx.accounts.authority_token_account.to_account_info(),
            to: ctx.accounts.token_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    /// Взятие флеш-займа
    pub fn flash_loan(
        ctx: Context<FlashLoan>,
        amount: u64,
    ) -> Result<()> {
        let flash_pool = &ctx.accounts.flash_pool;
        
        // Проверяем, что в пуле достаточно ликвидности
        require!(
            ctx.accounts.token_vault.amount >= amount,
            ErrorCode::InsufficientLiquidity
        );

        // Рассчитываем комиссию
        let fee = amount
            .checked_mul(flash_pool.fee_rate as u64)
            .unwrap()
            .checked_div(10000)
            .unwrap();

        // Создаем PDA для подписи
        let seeds = &[
            b"flash_pool",
            flash_pool.token_mint.as_ref(),
            &[flash_pool.bump],
        ];
        let signer = &[&seeds[..]];

        // Переводим токены заемщику
        let cpi_accounts = Transfer {
            from: ctx.accounts.token_vault.to_account_info(),
            to: ctx.accounts.borrower_token_account.to_account_info(),
            authority: ctx.accounts.flash_pool.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        // Сохраняем информацию о займе для проверки возврата
        let loan_state = &mut ctx.accounts.loan_state;
        loan_state.borrower = ctx.accounts.borrower.key();
        loan_state.amount = amount;
        loan_state.fee = fee;
        loan_state.is_active = true;

        msg!("Flash loan issued: {} tokens with {} fee", amount, fee);
        Ok(())
    }

    /// Возврат флеш-займа
    pub fn repay_flash_loan(ctx: Context<RepayFlashLoan>) -> Result<()> {
        let loan_state = &ctx.accounts.loan_state;
        
        // Проверяем, что займ активен
        require!(loan_state.is_active, ErrorCode::LoanNotActive);
        
        // Проверяем, что заемщик правильный
        require!(
            loan_state.borrower == ctx.accounts.borrower.key(),
            ErrorCode::UnauthorizedBorrower
        );

        let total_repay = loan_state.amount + loan_state.fee;

        // Переводим токены обратно в пул
        let cpi_accounts = Transfer {
            from: ctx.accounts.borrower_token_account.to_account_info(),
            to: ctx.accounts.token_vault.to_account_info(),
            authority: ctx.accounts.borrower.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, total_repay)?;

        // Закрываем займ
        let loan_state = &mut ctx.accounts.loan_state;
        loan_state.is_active = false;

        msg!("Flash loan repaid: {} total", total_repay);
        Ok(())
    }

    /// Создание собственного токена для схемы
    pub fn create_custom_token(
        ctx: Context<CreateCustomToken>,
        decimals: u8,
        initial_supply: u64,
    ) -> Result<()> {
        // Минтим начальное количество токенов на аккаунт создателя
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, initial_supply)?;

        msg!("Custom token created with supply: {}", initial_supply);
        Ok(())
    }

    /// Обмен SOL на кастомный токен
    pub fn swap_sol_for_custom_token(
        ctx: Context<SwapSolForCustomToken>,
        sol_amount: u64,
        min_token_amount: u64,
    ) -> Result<()> {
        // Простой расчет курса обмена (в реальности нужен AMM)
        let exchange_rate = 1000; // 1 SOL = 1000 кастомных токенов
        let token_amount = sol_amount * exchange_rate;

        require!(
            token_amount >= min_token_amount,
            ErrorCode::SlippageTooHigh
        );

        // Переводим SOL в пул
        let ix = solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.sol_vault.key(),
            sol_amount,
        );
        solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.sol_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Минтим токены пользователю
        let seeds = &[b"token_authority", &[ctx.accounts.token_pool.authority_bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.custom_token_mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.token_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx, token_amount)?;

        msg!("Swapped {} SOL for {} custom tokens", sol_amount, token_amount);
        Ok(())
    }

    /// Вывод позиции через кастомный токен с премией
    pub fn exit_position_via_custom_token(
        ctx: Context<ExitPosition>,
        token_amount: u64,
    ) -> Result<()> {
        let token_pool = &ctx.accounts.token_pool;
        
        // Сжигаем кастомные токены
        let cpi_accounts = token::Burn {
            mint: ctx.accounts.custom_token_mint.to_account_info(),
            from: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, token_amount)?;

        // Рассчитываем количество SOL для вывода с премией
        let base_exchange_rate = 1000; // 1 SOL = 1000 кастомных токенов
        let premium_rate = token_pool.exit_premium; // Премия при выводе (в базисных пунктах)
        
        let base_sol_amount = token_amount / base_exchange_rate;
        let premium = base_sol_amount
            .checked_mul(premium_rate as u64)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        
        let total_sol_amount = base_sol_amount + premium;

        // Переводим SOL пользователю
        **ctx.accounts.sol_vault.to_account_info().try_borrow_mut_lamports()? -= total_sol_amount;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += total_sol_amount;

        msg!("Exited position: {} tokens for {} SOL (with {}% premium)", 
             token_amount, total_sol_amount, premium_rate as f64 / 100.0);
        Ok(())
    }

    /// Покупка максимального количества кастомного токена на всю позицию
    pub fn buy_max_custom_tokens(
        ctx: Context<BuyMaxCustomTokens>,
        sol_amount: u64,
    ) -> Result<()> {
        // Переводим ВСЕ SOL в пул
        let ix = solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.sol_vault.key(),
            sol_amount,
        );
        solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.sol_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Рассчитываем максимальное количество токенов
        let exchange_rate = 1000; // 1 SOL = 1000 кастомных токенов
        let token_amount = sol_amount * exchange_rate;

        // Минтим МАКСИМАЛЬНОЕ количество токенов пользователю
        let seeds = &[b"token_authority", &[ctx.accounts.token_pool.authority_bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.custom_token_mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.token_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx, token_amount)?;

        msg!("Bought MAXIMUM: {} SOL converted to {} custom tokens", sol_amount, token_amount);
        Ok(())
    }

    /// Установка премии при выводе (только владелец токена)
    pub fn set_exit_premium(
        ctx: Context<SetExitPremium>,
        premium_rate: u16,
    ) -> Result<()> {
        require!(premium_rate <= 5000, ErrorCode::ExcessivePremium); // Максимум 50%
        
        let token_pool = &mut ctx.accounts.token_pool;
        token_pool.exit_premium = premium_rate;
        
        msg!("Exit premium set to {}%", premium_rate as f64 / 100.0);
        Ok(())
    }

    /// Инициализация токен-пула с настройками премии
    pub fn initialize_token_pool(
        ctx: Context<InitializeTokenPool>,
        authority_bump: u8,
        initial_premium: u16,
    ) -> Result<()> {
        let token_pool = &mut ctx.accounts.token_pool;
        token_pool.authority = ctx.accounts.authority.key();
        token_pool.custom_token_mint = ctx.accounts.custom_token_mint.key();
        token_pool.sol_vault = ctx.accounts.sol_vault.key();
        token_pool.authority_bump = authority_bump;
        token_pool.exit_premium = initial_premium;
        token_pool.total_volume = 0;
        
        msg!("Token pool initialized with {}% exit premium", initial_premium as f64 / 100.0);
        Ok(())
    }
}

// Структуры данных
#[account]
pub struct FlashPool {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub token_vault: Pubkey,
    pub fee_rate: u16, // в базисных пунктах (1/10000)
    pub bump: u8,
    pub total_liquidity: u64,
}

#[account]
pub struct LoanState {
    pub borrower: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub is_active: bool,
}

#[account]
pub struct TokenPool {
    pub authority: Pubkey,
    pub custom_token_mint: Pubkey,
    pub sol_vault: Pubkey,
    pub authority_bump: u8,
    pub exit_premium: u16,      // Премия при выводе в базисных пунктах
    pub total_volume: u64,      // Общий объем торгов
}

// Контексты для инструкций
#[derive(Accounts)]
#[instruction(pool_bump: u8)]
pub struct InitializeFlashPool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 2 + 1 + 8,
        seeds = [b"flash_pool", token_mint.key().as_ref()],
        bump = pool_bump,
    )]
    pub flash_pool: Account<'info, FlashPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        token::mint = token_mint,
        token::authority = flash_pool,
    )]
    pub token_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct FlashLoan<'info> {
    #[account(
        seeds = [b"flash_pool", flash_pool.token_mint.as_ref()],
        bump = flash_pool.bump,
    )]
    pub flash_pool: Account<'info, FlashPool>,
    
    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,
    
    pub borrower: Signer<'info>,
    
    #[account(mut)]
    pub borrower_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = borrower,
        space = 8 + 32 + 8 + 8 + 1,
    )]
    pub loan_state: Account<'info, LoanState>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct RepayFlashLoan<'info> {
    #[account(
        mut,
        close = borrower,
    )]
    pub loan_state: Account<'info, LoanState>,
    
    pub borrower: Signer<'info>,
    
    #[account(mut)]
    pub borrower_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateCustomToken<'info> {
    #[account(
        init,
        payer = mint_authority,
        mint::decimals = 9,
        mint::authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub mint_authority: Signer<'info>,
    
    #[account(
        init,
        payer = mint_authority,
        token::mint = mint,
        token::authority = mint_authority,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SwapSolForCustomToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_pool: Account<'info, TokenPool>,
    
    /// CHECK: This is safe because we only read from it
    #[account(mut)]
    pub sol_vault: AccountInfo<'info>,
    
    pub custom_token_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is the token authority PDA
    pub token_authority: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExitPosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_pool: Account<'info, TokenPool>,
    
    #[account(mut)]
    pub custom_token_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is safe because we only transfer from it
    #[account(mut)]
    pub sol_vault: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BuyMaxCustomTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_pool: Account<'info, TokenPool>,
    
    /// CHECK: This is safe because we only read from it
    #[account(mut)]
    pub sol_vault: AccountInfo<'info>,
    
    pub custom_token_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is the token authority PDA
    pub token_authority: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetExitPremium<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        has_one = authority,
    )]
    pub token_pool: Account<'info, TokenPool>,
}

#[derive(Accounts)]
#[instruction(authority_bump: u8)]
pub struct InitializeTokenPool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 1 + 2 + 8,
        seeds = [b"token_pool", custom_token_mint.key().as_ref()],
        bump = authority_bump,
    )]
    pub token_pool: Account<'info, TokenPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub custom_token_mint: Account<'info, Mint>,
    
    /// CHECK: This will be the SOL vault
    pub sol_vault: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Коды ошибок
#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient liquidity in the pool")]
    InsufficientLiquidity,
    #[msg("Loan is not active")]
    LoanNotActive,
    #[msg("Unauthorized borrower")]
    UnauthorizedBorrower,
    #[msg("Slippage tolerance exceeded")]
    SlippageTooHigh,
    #[msg("Exit premium rate is too high (max 50%)")]
    ExcessivePremium,
}