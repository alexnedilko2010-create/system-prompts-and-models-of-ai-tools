use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

/**
 * Примеры УЯЗВИМЫХ флеш-займ контрактов для образовательных целей
 * 
 * ВНИМАНИЕ: Данные контракты содержат НАМЕРЕННЫЕ уязвимости!
 * Используются ТОЛЬКО для демонстрации проблем безопасности!
 * НИКОГДА не используйте подобный код в продакшене!
 */

/// УЯЗВИМОСТЬ 1: Неправильная проверка баланса
#[derive(Accounts)]
pub struct VulnerableFlashLoan1<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 1,
        seeds = [b"vulnerable_flash_1", token_mint.key().as_ref()],
        bump,
    )]
    pub vulnerable_pool: Account<'info, VulnerableFlashPool1>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        token::mint = token_mint,
        token::authority = vulnerable_pool,
    )]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ExploitVulnerable1<'info> {
    #[account(mut)]
    pub attacker: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"vulnerable_flash_1", vulnerable_pool.token_mint.as_ref()],
        bump = vulnerable_pool.bump,
    )]
    pub vulnerable_pool: Account<'info, VulnerableFlashPool1>,
    
    #[account(mut)]
    pub attacker_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    
    /// Безопасный адрес куда переводим токены
    #[account(mut)]
    pub safe_address_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

/// УЯЗВИМОСТЬ 2: Отсутствие reentrancy guard
#[derive(Accounts)]
pub struct VulnerableFlashLoan2<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 8 + 1 + 1,
        seeds = [b"vulnerable_flash_2", token_mint.key().as_ref()],
        bump,
    )]
    pub vulnerable_pool: Account<'info, VulnerableFlashPool2>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        token::mint = token_mint,
        token::authority = vulnerable_pool,
    )]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ExploitReentrancy<'info> {
    #[account(mut)]
    pub attacker: Signer<'info>,
    
    #[account(mut)]
    pub vulnerable_pool: Account<'info, VulnerableFlashPool2>,
    
    #[account(mut)]
    pub attacker_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

// Структуры данных для уязвимых пулов
#[account]
pub struct VulnerableFlashPool1 {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub vault: Pubkey,
    pub total_liquidity: u64,
    pub fee_rate: u64,
    pub bump: u8,
}

#[account]
pub struct VulnerableFlashPool2 {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub vault: Pubkey,
    pub total_liquidity: u64,
    pub is_locked: bool,    // Должен быть reentrancy guard, но реализован неправильно
    pub bump: u8,
}

impl<'info> VulnerableFlashPool1 {
    /// 🚨 УЯЗВИМЫЙ флеш-займ с неправильной проверкой баланса
    pub fn vulnerable_flash_loan_1(
        ctx: Context<ExploitVulnerable1>,
        amount: u64,
    ) -> Result<()> {
        msg!("🚨 VULNERABLE FLASH LOAN 1 - Incorrect balance check");
        
        let vault = &ctx.accounts.vault;
        let balance_before = vault.amount;
        
        // Переводим токены атакующему
        let pool_seeds = &[
            b"vulnerable_flash_1",
            ctx.accounts.vulnerable_pool.token_mint.as_ref(),
            &[ctx.accounts.vulnerable_pool.bump],
        ];
        let signer = &[&pool_seeds[..]];
        
        let transfer_to_attacker = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.attacker_token_account.to_account_info(),
                authority: ctx.accounts.vulnerable_pool.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_to_attacker, amount)?;
        
        // Имитируем callback (в реальности это был бы CPI)
        msg!("Calling attacker callback...");
        
        // Атакующий переводит токены на безопасный адрес
        let transfer_to_safe = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.attacker_token_account.to_account_info(),
                to: ctx.accounts.safe_address_account.to_account_info(),
                authority: ctx.accounts.attacker.to_account_info(),
            },
        );
        token::transfer(transfer_to_safe, amount - 1)?; // Оставляем 1 token
        
        // 🚨 УЯЗВИМОСТЬ: Неправильная проверка баланса
        ctx.accounts.vault.reload()?;
        let balance_after = ctx.accounts.vault.amount;
        
        // БАГ: Проверяем > вместо >= (и не учитываем fee)
        require!(
            balance_after > balance_before,
            VulnerableFlashLoanError::NotRepaid
        );
        
        // Возвращаем только 1 token вместо amount + fee
        let return_minimal = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.attacker_token_account.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.attacker.to_account_info(),
            },
        );
        token::transfer(return_minimal, 1)?;
        
        msg!("🎉 VULNERABILITY 1 EXPLOITED! {} tokens stolen", amount - 1);
        
        Ok(())
    }
}

impl<'info> VulnerableFlashPool2 {
    /// 🚨 УЯЗВИМЫЙ флеш-займ без reentrancy protection
    pub fn vulnerable_flash_loan_2(
        ctx: Context<ExploitReentrancy>,
        amount: u64,
    ) -> Result<()> {
        msg!("🚨 VULNERABLE FLASH LOAN 2 - No reentrancy protection");
        
        let pool = &mut ctx.accounts.vulnerable_pool;
        
        // СЛАБАЯ защита от reentrancy (можно обойти)
        require!(!pool.is_locked, VulnerableFlashLoanError::Locked);
        pool.is_locked = true;
        
        let vault = &ctx.accounts.vault;
        let balance_before = vault.amount;
        
        // Переводим токены атакующему
        let pool_seeds = &[
            b"vulnerable_flash_2",
            pool.token_mint.as_ref(),
            &[pool.bump],
        ];
        let signer = &[&pool_seeds[..]];
        
        let transfer_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.attacker_token_account.to_account_info(),
                authority: ctx.accounts.vulnerable_pool.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_cpi, amount)?;
        
        // Имитируем callback где происходит reentrancy атака
        msg!("Calling callback - reentrancy attack possible here");
        
        // 🚨 БАГ: Снимаем блокировку ДО проверки возврата
        pool.is_locked = false;
        
        // Теперь атакующий может вызвать reentrancy
        // В реальности здесь был бы рекурсивный вызов
        
        // Проверка возврата (но уже после снятия блокировки)
        ctx.accounts.vault.reload()?;
        let balance_after = ctx.accounts.vault.amount;
        
        require!(
            balance_after >= balance_before + amount / 1000, // 0.1% fee
            VulnerableFlashLoanError::NotRepaid
        );
        
        msg!("Flash loan 2 completed");
        Ok(())
    }
}

/// ДЕМОНСТРАЦИЯ: Как НЕ должен выглядеть флеш-займ
pub fn demonstrate_vulnerable_patterns(
    ctx: Context<ExploitVulnerable1>,
) -> Result<()> {
    msg!("🚨 DEMONSTRATING VULNERABLE FLASH LOAN PATTERNS");
    
    // Паттерн 1: Проверка баланса в неправильном месте
    msg!("Vulnerability 1: Balance check before state updates");
    
    // Паттерн 2: Неучтение fee в проверке
    msg!("Vulnerability 2: Fee not included in balance check");
    
    // Паттерн 3: Использование > вместо >=
    msg!("Vulnerability 3: Using > instead of >= in balance check");
    
    // Паттерн 4: Отсутствие reentrancy protection
    msg!("Vulnerability 4: Missing reentrancy guards");
    
    // Паттерн 5: Неправильный порядок операций
    msg!("Vulnerability 5: Incorrect order of operations");
    
    msg!("⚠️ These patterns should NEVER be used in production!");
    
    Ok(())
}

/// ДЕМОНСТРАЦИЯ: Правильная реализация флеш-займа
pub fn demonstrate_secure_flash_loan(
    ctx: Context<ExploitVulnerable1>,
    amount: u64,
) -> Result<()> {
    msg!("✅ DEMONSTRATING SECURE FLASH LOAN IMPLEMENTATION");
    
    // Правильный паттерн:
    msg!("1. Check balance BEFORE transfer");
    msg!("2. Transfer tokens to borrower");
    msg!("3. Execute borrower callback");
    msg!("4. Check balance AFTER callback");
    msg!("5. Ensure balance >= initial + fee");
    msg!("6. Use reentrancy guards");
    msg!("7. Proper error handling");
    
    msg!("✅ This is how flash loans should be implemented!");
    
    Ok(())
}

#[error_code]
pub enum VulnerableFlashLoanError {
    #[msg("Flash loan not repaid")]
    NotRepaid,
    #[msg("Pool is locked (weak reentrancy protection)")]
    Locked,
    #[msg("Insufficient balance for flash loan")]
    InsufficientBalance,
    #[msg("Invalid fee calculation")]
    InvalidFee,
}