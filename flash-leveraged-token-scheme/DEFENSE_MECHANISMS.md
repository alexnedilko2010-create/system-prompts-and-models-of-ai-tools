# 🛡️ Защиты от атак с подменой токенов

## 🎯 Анализ защитных механизмов против атак подменой токенов

### Атака которую защищаем:
```
🎭 Флеш-займ с подменой токенов:
1. Флеш-займ REAL_USDC → размещение как залог
2. Займ против залога
3. Создание FAKE_USDC с идентичным интерфейсом  
4. Подмена адреса: REAL_USDC → FAKE_USDC
5. "Вывод" залога: тратим FAKE_USDC, получаем REAL_USDC
6. Восстановление адреса, возврат флеш-займа
7. Прибыль: разница между выведенным залогом и займом
```

## 🛡️ УРОВЕНЬ 1: Архитектурные защиты

### 1.1 Immutable Token Addresses
```rust
// ✅ ПРАВИЛЬНО: Неизменяемые адреса токенов
#[account]
pub struct SecureLendingPool {
    pub authority: Pubkey,
    // Адреса токенов устанавливаются при инициализации и не могут быть изменены
    pub usdc_mint: Pubkey,
    pub weth_mint: Pubkey,
    pub wbtc_mint: Pubkey,
    // Остальные поля...
}

impl SecureLendingPool {
    pub fn initialize(
        ctx: Context<Initialize>,
        usdc_mint: Pubkey,
        weth_mint: Pubkey,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.lending_pool;
        pool.usdc_mint = usdc_mint;
        pool.weth_mint = weth_mint;
        // Адреса больше НИКОГДА не изменяются!
        Ok(())
    }
}

// ❌ УЯЗВИМО: Изменяемые адреса токенов
#[account]
pub struct VulnerableLendingPool {
    pub authority: Pubkey,
    pub current_token_mint: Pubkey, // Можно изменить!
    // ...
}
```

### 1.2 Константные адреса в коде
```rust
// ✅ ПРАВИЛЬНО: Хардкод известных токенов
pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const WETH_MINT: Pubkey = pubkey!("7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs");

#[derive(Accounts)]
pub struct SecureDeposit<'info> {
    // Проверяем что токен из whitelist
    #[account(
        constraint = token_mint.key() == USDC_MINT || 
                    token_mint.key() == WETH_MINT @ ErrorCode::InvalidToken
    )]
    pub token_mint: Account<'info, Mint>,
    // ...
}
```

### 1.3 Whitelist разрешенных токенов
```rust
#[account]
pub struct TokenWhitelist {
    pub authority: Pubkey,
    pub approved_tokens: Vec<Pubkey>, // Максимум 10 токенов
}

impl TokenWhitelist {
    pub fn is_token_approved(&self, token: &Pubkey) -> bool {
        self.approved_tokens.contains(token)
    }
    
    // Только authority может добавлять токены с timelock
    pub fn add_token_with_timelock(
        ctx: Context<AddToken>,
        token: Pubkey,
        execution_time: i64,
    ) -> Result<()> {
        require!(
            Clock::get()?.unix_timestamp >= execution_time,
            ErrorCode::TimelockNotExpired
        );
        
        let whitelist = &mut ctx.accounts.whitelist;
        require!(
            !whitelist.approved_tokens.contains(&token),
            ErrorCode::TokenAlreadyApproved
        );
        
        whitelist.approved_tokens.push(token);
        Ok(())
    }
}
```

## 🛡️ УРОВЕНЬ 2: Валидационные защиты

### 2.1 Многоуровневая проверка токенов
```rust
impl SecureLendingPool {
    pub fn deposit_with_validation(
        ctx: Context<SecureDeposit>,
        amount: u64,
    ) -> Result<()> {
        let token_mint = &ctx.accounts.token_mint;
        
        // Проверка 1: Токен в whitelist
        require!(
            ctx.accounts.whitelist.is_token_approved(&token_mint.key()),
            ErrorCode::TokenNotWhitelisted
        );
        
        // Проверка 2: Токен соответствует ожидаемому
        require!(
            token_mint.key() == ctx.accounts.lending_pool.usdc_mint,
            ErrorCode::TokenMismatch
        );
        
        // Проверка 3: Токен имеет правильные параметры
        require!(
            token_mint.decimals == 6,
            ErrorCode::InvalidDecimals
        );
        
        // Проверка 4: Supply токена в разумных пределах
        require!(
            token_mint.supply > 1_000_000 * 10u64.pow(6), // Минимум 1M tokens
            ErrorCode::SuspiciousSupply
        );
        
        // Проверка 5: Токен не создан недавно
        let token_account_info = token_mint.to_account_info();
        let creation_time = token_account_info.lamports(); // Proxy для времени создания
        require!(
            Clock::get()?.unix_timestamp - creation_time as i64 > 86400, // 24 часа
            ErrorCode::TokenTooNew
        );
        
        // Только после всех проверок выполняем операцию
        self.execute_deposit(ctx, amount)
    }
}
```

### 2.2 Проверка истории токена
```rust
#[account]
pub struct TokenHistory {
    pub token_mint: Pubkey,
    pub first_seen: i64,
    pub total_volume: u64,
    pub unique_holders: u32,
    pub is_suspicious: bool,
}

impl TokenHistory {
    pub fn validate_token_legitimacy(&self) -> Result<()> {
        // Проверяем что токен существует достаточно долго
        require!(
            Clock::get()?.unix_timestamp - self.first_seen > 7 * 86400, // 7 дней
            ErrorCode::TokenTooNew
        );
        
        // Проверяем минимальную активность
        require!(
            self.total_volume > 100_000 * 10u64.pow(6), // 100k объем
            ErrorCode::InsufficientActivity
        );
        
        // Проверяем распределение холдеров
        require!(
            self.unique_holders > 100,
            ErrorCode::TooFewHolders
        );
        
        // Проверяем что токен не помечен как подозрительный
        require!(
            !self.is_suspicious,
            ErrorCode::SuspiciousToken
        );
        
        Ok(())
    }
}
```

## 🛡️ УРОВЕНЬ 3: Runtime защиты

### 3.1 Reentrancy Guards
```rust
#[account]
pub struct ReentrancyGuard {
    pub is_locked: bool,
}

impl ReentrancyGuard {
    pub fn lock(&mut self) -> Result<()> {
        require!(!self.is_locked, ErrorCode::ReentrancyDetected);
        self.is_locked = true;
        Ok(())
    }
    
    pub fn unlock(&mut self) {
        self.is_locked = false;
    }
}

// Макрос для автоматической защиты от reentrancy
macro_rules! nonreentrant {
    ($guard:expr, $code:block) => {
        $guard.lock()?;
        let result = $code;
        $guard.unlock();
        result
    };
}

impl SecureLendingPool {
    pub fn secure_withdraw(
        ctx: Context<SecureWithdraw>,
        amount: u64,
    ) -> Result<()> {
        let guard = &mut ctx.accounts.reentrancy_guard;
        
        nonreentrant!(guard, {
            // Все проверки и операции внутри защищенного блока
            self.validate_withdrawal(ctx, amount)?;
            self.execute_withdrawal(ctx, amount)?;
            Ok(())
        })
    }
}
```

### 3.2 Anomaly Detection
```rust
#[account]
pub struct AnomalyDetector {
    pub recent_operations: Vec<Operation>,
    pub suspicious_patterns: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Operation {
    pub user: Pubkey,
    pub operation_type: OperationType,
    pub amount: u64,
    pub timestamp: i64,
    pub token_mint: Pubkey,
}

impl AnomalyDetector {
    pub fn check_for_anomalies(&mut self, operation: &Operation) -> Result<()> {
        // Проверка 1: Слишком большая сумма
        if operation.amount > 1_000_000 * 10u64.pow(6) { // 1M USDC
            self.suspicious_patterns += 1;
            msg!("⚠️ Large amount detected: {}", operation.amount);
        }
        
        // Проверка 2: Слишком частые операции
        let recent_ops = self.recent_operations.iter()
            .filter(|op| op.user == operation.user)
            .filter(|op| operation.timestamp - op.timestamp < 60) // 1 минута
            .count();
        
        if recent_ops > 5 {
            return Err(ErrorCode::TooFrequentOperations.into());
        }
        
        // Проверка 3: Подозрительная последовательность операций
        if self.detect_substitution_pattern(operation)? {
            return Err(ErrorCode::SubstitutionPatternDetected.into());
        }
        
        // Добавляем операцию в историю
        self.recent_operations.push(operation.clone());
        
        // Ограничиваем размер истории
        if self.recent_operations.len() > 100 {
            self.recent_operations.remove(0);
        }
        
        Ok(())
    }
    
    fn detect_substitution_pattern(&self, operation: &Operation) -> Result<bool> {
        // Ищем паттерн: deposit → borrow → withdraw в короткий промежуток времени
        let user_ops: Vec<_> = self.recent_operations.iter()
            .filter(|op| op.user == operation.user)
            .filter(|op| operation.timestamp - op.timestamp < 300) // 5 минут
            .collect();
        
        let has_deposit = user_ops.iter().any(|op| matches!(op.operation_type, OperationType::Deposit));
        let has_borrow = user_ops.iter().any(|op| matches!(op.operation_type, OperationType::Borrow));
        
        if has_deposit && has_borrow && matches!(operation.operation_type, OperationType::Withdraw) {
            msg!("🚨 Potential substitution pattern detected for user: {}", operation.user);
            return Ok(true);
        }
        
        Ok(false)
    }
}
```

## 🛡️ УРОВЕНЬ 4: Мониторинг и алерты

### 4.1 On-chain мониторинг
```rust
#[account]
pub struct SecurityMonitor {
    pub alert_threshold: u32,
    pub current_alerts: u32,
    pub emergency_mode: bool,
}

impl SecurityMonitor {
    pub fn check_security_status(&mut self) -> Result<()> {
        if self.current_alerts > self.alert_threshold {
            self.emergency_mode = true;
            msg!("🚨 EMERGENCY MODE ACTIVATED - Too many security alerts");
        }
        
        Ok(())
    }
    
    pub fn log_security_event(
        &mut self,
        event_type: SecurityEventType,
        severity: Severity,
        details: String,
    ) -> Result<()> {
        match severity {
            Severity::Critical => {
                self.current_alerts += 3;
                msg!("🚨 CRITICAL SECURITY EVENT: {} - {}", event_type, details);
            },
            Severity::High => {
                self.current_alerts += 2;
                msg!("⚠️ HIGH SECURITY EVENT: {} - {}", event_type, details);
            },
            Severity::Medium => {
                self.current_alerts += 1;
                msg!("⚡ MEDIUM SECURITY EVENT: {} - {}", event_type, details);
            },
            Severity::Low => {
                msg!("📝 LOW SECURITY EVENT: {} - {}", event_type, details);
            }
        }
        
        self.check_security_status()?;
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum SecurityEventType {
    SuspiciousTokenDetected,
    AnomalousTransaction,
    ReentrancyAttempt,
    LargeWithdrawal,
    RapidOperations,
    UnknownToken,
}
```

### 4.2 Circuit Breakers
```rust
#[account]
pub struct CircuitBreaker {
    pub is_paused: bool,
    pub pause_reason: String,
    pub pause_timestamp: i64,
    pub unpause_delay: i64, // Минимальная задержка перед разблокировкой
}

impl CircuitBreaker {
    pub fn emergency_pause(&mut self, reason: String) -> Result<()> {
        self.is_paused = true;
        self.pause_reason = reason;
        self.pause_timestamp = Clock::get()?.unix_timestamp;
        
        msg!("🛑 EMERGENCY PAUSE ACTIVATED: {}", self.pause_reason);
        Ok(())
    }
    
    pub fn request_unpause(&mut self) -> Result<()> {
        require!(self.is_paused, ErrorCode::NotPaused);
        
        let time_since_pause = Clock::get()?.unix_timestamp - self.pause_timestamp;
        require!(
            time_since_pause >= self.unpause_delay,
            ErrorCode::UnpauseDelayNotMet
        );
        
        self.is_paused = false;
        self.pause_reason.clear();
        
        msg!("✅ Protocol unpaused after {} seconds", time_since_pause);
        Ok(())
    }
    
    pub fn check_not_paused(&self) -> Result<()> {
        require!(!self.is_paused, ErrorCode::ProtocolPaused);
        Ok(())
    }
}

// Модификатор для проверки паузы
macro_rules! when_not_paused {
    ($circuit_breaker:expr) => {
        $circuit_breaker.check_not_paused()?;
    };
}
```

## 🛡️ УРОВЕНЬ 5: Governance защиты

### 5.1 Multi-sig для критических операций
```rust
#[account]
pub struct MultiSigConfig {
    pub signers: Vec<Pubkey>,
    pub required_signatures: u8,
    pub pending_transactions: Vec<PendingTransaction>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PendingTransaction {
    pub id: u64,
    pub transaction_data: Vec<u8>,
    pub signatures: Vec<Pubkey>,
    pub created_at: i64,
    pub expires_at: i64,
}

impl MultiSigConfig {
    pub fn propose_critical_change(
        &mut self,
        transaction_data: Vec<u8>,
        proposer: Pubkey,
    ) -> Result<u64> {
        require!(
            self.signers.contains(&proposer),
            ErrorCode::UnauthorizedProposer
        );
        
        let transaction_id = self.pending_transactions.len() as u64;
        let transaction = PendingTransaction {
            id: transaction_id,
            transaction_data,
            signatures: vec![proposer],
            created_at: Clock::get()?.unix_timestamp,
            expires_at: Clock::get()?.unix_timestamp + 7 * 86400, // 7 дней
        };
        
        self.pending_transactions.push(transaction);
        Ok(transaction_id)
    }
    
    pub fn sign_transaction(
        &mut self,
        transaction_id: u64,
        signer: Pubkey,
    ) -> Result<bool> {
        require!(
            self.signers.contains(&signer),
            ErrorCode::UnauthorizedSigner
        );
        
        let transaction = self.pending_transactions
            .get_mut(transaction_id as usize)
            .ok_or(ErrorCode::TransactionNotFound)?;
        
        require!(
            Clock::get()?.unix_timestamp < transaction.expires_at,
            ErrorCode::TransactionExpired
        );
        
        if !transaction.signatures.contains(&signer) {
            transaction.signatures.push(signer);
        }
        
        // Проверяем достаточно ли подписей
        Ok(transaction.signatures.len() >= self.required_signatures as usize)
    }
}
```

### 5.2 Timelock для изменений
```rust
#[account]
pub struct TimelockController {
    pub delay: i64, // Минимальная задержка в секундах
    pub pending_operations: Vec<TimelockOperation>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TimelockOperation {
    pub id: u64,
    pub target: Pubkey,
    pub data: Vec<u8>,
    pub scheduled_time: i64,
    pub executed: bool,
}

impl TimelockController {
    pub fn schedule_operation(
        &mut self,
        target: Pubkey,
        data: Vec<u8>,
    ) -> Result<u64> {
        let operation_id = self.pending_operations.len() as u64;
        let operation = TimelockOperation {
            id: operation_id,
            target,
            data,
            scheduled_time: Clock::get()?.unix_timestamp + self.delay,
            executed: false,
        };
        
        self.pending_operations.push(operation);
        
        msg!("⏰ Operation {} scheduled for execution at {}", 
             operation_id, operation.scheduled_time);
        
        Ok(operation_id)
    }
    
    pub fn execute_operation(
        &mut self,
        operation_id: u64,
    ) -> Result<()> {
        let operation = self.pending_operations
            .get_mut(operation_id as usize)
            .ok_or(ErrorCode::OperationNotFound)?;
        
        require!(!operation.executed, ErrorCode::AlreadyExecuted);
        require!(
            Clock::get()?.unix_timestamp >= operation.scheduled_time,
            ErrorCode::TimelockNotExpired
        );
        
        operation.executed = true;
        
        msg!("✅ Timelock operation {} executed", operation_id);
        Ok(())
    }
}
```

## 🛡️ КОМПЛЕКСНАЯ ЗАЩИТА: Пример интеграции всех уровней

```rust
#[program]
pub mod ultra_secure_lending {
    use super::*;
    
    pub fn secure_deposit(
        ctx: Context<UltraSecureDeposit>,
        amount: u64,
    ) -> Result<()> {
        // Уровень 1: Проверка паузы
        when_not_paused!(ctx.accounts.circuit_breaker);
        
        // Уровень 2: Reentrancy защита
        let guard = &mut ctx.accounts.reentrancy_guard;
        guard.lock()?;
        
        // Уровень 3: Валидация токена
        ctx.accounts.whitelist.validate_token(&ctx.accounts.token_mint.key())?;
        ctx.accounts.token_history.validate_token_legitimacy()?;
        
        // Уровень 4: Проверка аномалий
        let operation = Operation {
            user: ctx.accounts.user.key(),
            operation_type: OperationType::Deposit,
            amount,
            timestamp: Clock::get()?.unix_timestamp,
            token_mint: ctx.accounts.token_mint.key(),
        };
        ctx.accounts.anomaly_detector.check_for_anomalies(&operation)?;
        
        // Уровень 5: Логирование безопасности
        if amount > 100_000 * 10u64.pow(6) { // Большая сумма
            ctx.accounts.security_monitor.log_security_event(
                SecurityEventType::LargeWithdrawal,
                Severity::Medium,
                format!("Large deposit: {} by {}", amount, ctx.accounts.user.key())
            )?;
        }
        
        // Только после всех проверок выполняем операцию
        let result = ctx.accounts.lending_pool.execute_deposit(amount);
        
        // Разблокируем reentrancy guard
        guard.unlock();
        
        result
    }
}
```

## 📊 Эффективность защит

### Уровни защиты vs. Типы атак:

| Защита | Подмена токенов | Reentrancy | Flash Loan | Governance |
|--------|-----------------|------------|------------|------------|
| Immutable адреса | 🛡️🛡️🛡️ | 🛡️ | 🛡️🛡️ | 🛡️ |
| Whitelist токенов | 🛡️🛡️🛡️ | 🛡️ | 🛡️🛡️ | 🛡️🛡️ |
| Reentrancy guards | 🛡️ | 🛡️🛡️🛡️ | 🛡️🛡️ | 🛡️ |
| Anomaly detection | 🛡️🛡️ | 🛡️🛡️ | 🛡️🛡️🛡️ | 🛡️🛡️ |
| Circuit breakers | 🛡️🛡️ | 🛡️🛡️ | 🛡️🛡️ | 🛡️🛡️🛡️ |
| Multi-sig | 🛡️ | 🛡️ | 🛡️ | 🛡️🛡️🛡️ |
| Timelock | 🛡️🛡️ | 🛡️ | 🛡️ | 🛡️🛡️🛡️ |

**Легенда**: 🛡️ - Слабая защита, 🛡️🛡️ - Средняя защита, 🛡️🛡️🛡️ - Сильная защита

## 🎯 Рекомендации для разработчиков

### ✅ ОБЯЗАТЕЛЬНО:
1. **Используйте immutable адреса токенов**
2. **Внедрите whitelist разрешенных токенов**
3. **Добавьте reentrancy guards на все функции**
4. **Реализуйте систему мониторинга аномалий**
5. **Используйте circuit breakers для экстренных ситуаций**

### 🔍 РЕКОМЕНДУЕТСЯ:
1. Multi-sig для критических операций
2. Timelock для изменений протокола
3. Детальное логирование всех операций
4. Регулярные аудиты безопасности
5. Bug bounty программы

### ⚠️ НИКОГДА НЕ ДЕЛАЙТЕ:
1. Не делайте адреса токенов изменяемыми
2. Не полагайтесь только на один уровень защиты
3. Не игнорируйте предупреждения системы мониторинга
4. Не деплойте без тщательного аудита
5. Не забывайте про edge cases в тестах

## 🏁 Заключение

**Защита от атак подменой токенов требует комплексного подхода:**

1. **Архитектурный уровень**: Правильный дизайн с immutable адресами
2. **Валидационный уровень**: Множественные проверки токенов
3. **Runtime уровень**: Защита от reentrancy и аномалий
4. **Мониторинг уровень**: Отслеживание подозрительной активности
5. **Governance уровень**: Контроль изменений через multi-sig и timelock

**Правильно реализованные защиты делают атаки подменой токенов практически невозможными!**