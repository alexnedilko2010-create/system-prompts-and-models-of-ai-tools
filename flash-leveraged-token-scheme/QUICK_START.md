# 🚀 Быстрый старт - Схема увеличения позиции

## 📋 Предварительные требования

1. **Node.js** (версия 16+)
2. **Rust** и **Cargo**
3. **Solana CLI** 
4. **Anchor Framework**

```bash
# Установка Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.16.0/install)"

# Установка Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Установка зависимостей Node.js
npm install
```

## ⚡ Быстрый запуск (5 минут)

### 1. Клонирование и установка
```bash
cd /workspace/flash-leveraged-token-scheme
npm install
```

### 2. Настройка Solana
```bash
# Создание кошелька (если нет)
solana-keygen new --outfile ~/.config/solana/id.json

# Настройка на devnet для тестирования
solana config set --url devnet

# Пополнение кошелька (devnet)
solana airdrop 2
```

### 3. Сборка и развертывание
```bash
# Сборка контрактов
anchor build

# Развертывание на devnet
anchor deploy

# Запуск скрипта развертывания
npm run deploy
```

### 4. Демонстрация схемы
```bash
# Запуск полной демонстрации
npm run demo
```

### 5. Тестирование
```bash
# Запуск тестов
npm test
```

## 🎯 Что произойдет при запуске демо

```
🚀 Инициализация схемы увеличения позиции...
💰 Базовый токен создан: [ADDRESS]
🎯 Кастомный токен создан: [ADDRESS]
🏦 Флеш-пул инициализирован с ликвидностью: 100000 USDC
🔄 Токен-пул для обмена создан
✅ Инициализация завершена

🎯 ВЫПОЛНЕНИЕ СХЕМЫ УВЕЛИЧЕНИЯ ПОЗИЦИИ
==================================================
💼 Начальный капитал: 3000 USDC
🏦 Беру флеш-займ: 10000 USDC
💰 Баланс после флеш-займа: 13000 USDC
📈 Увеличение позиции в 4.33x раз
🔄 Получаем под залог: 5 SOL
🎯 Получено кастомных токенов: 5000
💸 Возвращаю флеш-займ: 10050 USDC (включая комиссию 50 USDC)

📊 РЕЗУЛЬТАТ СХЕМЫ:
==============================
💼 Остаток базовых токенов: -2050 USDC
🎯 Кастомные токены: 5000
📈 Общая позиция увеличена с 3000 до 7950 эквивалента
🚀 Коэффициент увеличения: 2.65x

🏃‍♂️ ВЫВОД ПОЗИЦИИ ЧЕРЕЗ КАСТОМНЫЙ ТОКЕН:
✅ Позиция выведена через кастомный токен
⚠️  Остался долг, но позиция была в разы больше исходного капитала!

🎉 СХЕМА УСПЕШНО ВЫПОЛНЕНА!
```

## 📊 Структура проекта

```
flash-leveraged-token-scheme/
├── programs/
│   └── flash-leveraged-scheme/
│       └── src/
│           └── lib.rs                 # Основной контракт
├── scripts/
│   ├── leverage_scheme.ts             # Демо схемы
│   └── deploy.ts                      # Скрипт развертывания
├── tests/
│   └── flash_leveraged_scheme.ts      # Тесты
├── config/
│   └── deployment.json               # Конфигурация после развертывания
├── README.md                         # Основная документация
├── SCHEME_EXPLANATION.md             # Детальное объяснение схемы
├── RISK_ANALYSIS.md                  # Анализ рисков
└── QUICK_START.md                    # Этот файл
```

## 🛠️ Команды разработки

```bash
# Сборка
npm run build
anchor build

# Тестирование
npm test
anchor test

# Развертывание
npm run deploy
anchor deploy

# Демонстрация
npm run demo

# Линтинг
npm run lint

# Форматирование
npm run format
```

## 🔧 Настройка для разных сетей

### Devnet (рекомендуется для тестирования)
```bash
solana config set --url devnet
anchor build
anchor deploy
```

### Testnet
```bash
solana config set --url testnet
anchor build
anchor deploy
```

### Mainnet (⚠️ ТОЛЬКО ДЛЯ ПРОДАКШЕНА!)
```bash
solana config set --url mainnet-beta
# ВНИМАНИЕ: Убедитесь в безопасности кода!
anchor build
anchor deploy
```

## 🚨 ВАЖНЫЕ ПРЕДУПРЕЖДЕНИЯ

### ⛔ Перед использованием на mainnet:
1. **Полный аудит кода** профессиональными аудиторами
2. **Тестирование** на devnet/testnet с реальными суммами
3. **Страхование** позиций
4. **План управления рисками**
5. **Экстренный план выхода**

### ⚠️ Риски:
- **Полная потеря капитала** при неблагоприятном сценарии
- **Ликвидация позиций** при падении цен
- **Технические риски** смарт-контрактов
- **Регулятивные риски**

### 💡 Рекомендации:
- Начинайте с **минимальных сумм** ($10-100)
- Используйте только **10-20% от капитала**
- Постоянно **мониторьте позиции**
- Держите **резерв для экстренных ситуаций**

## 📞 Поддержка и помощь

### Частые проблемы:

**1. Ошибка "Insufficient funds"**
```bash
# Пополните кошелек
solana airdrop 2  # для devnet
```

**2. Ошибка компиляции Anchor**
```bash
# Обновите Anchor
avm install latest
avm use latest
```

**3. Проблемы с RPC**
```bash
# Используйте другой RPC endpoint
solana config set --url https://api.devnet.solana.com
```

### Логи и отладка:
```bash
# Включить подробные логи
export RUST_LOG=solana_runtime::system_instruction_processor=trace,solana_runtime::message_processor=debug,solana_bpf_loader=debug,solana_rbpf=debug

# Запуск с логами
anchor test --skip-local-validator
```

## 🎓 Обучающие материалы

1. **SCHEME_EXPLANATION.md** - Подробное объяснение схемы
2. **RISK_ANALYSIS.md** - Детальный анализ рисков
3. **Код контрактов** - Изучите `programs/flash-leveraged-scheme/src/lib.rs`
4. **Тесты** - Посмотрите `tests/flash_leveraged_scheme.ts`

## 🚀 Следующие шаги после запуска

1. **Изучите результаты** демонстрации
2. **Прочитайте анализ рисков** в RISK_ANALYSIS.md
3. **Поэкспериментируйте** с параметрами в тестовой среде
4. **Разработайте стратегию** управления рисками
5. **Начните с минимальных сумм** на devnet

---

**⚠️ Помните: данная схема предназначена только для образовательных целей и демонстрации возможностей DeFi. Реальное использование несет экстремально высокие риски!**