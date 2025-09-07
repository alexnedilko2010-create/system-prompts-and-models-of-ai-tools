# 📋 ТЗ для программиста - Упрощенная версия

## 🎯 Что нужно сделать

```
ЗАДАЧА:
═══════════════════════════════════════════

Создать бота, который:
1. Берет Flash Loan (большой займ)
2. Кладет в concentrated позицию
3. Забирает fees (комиссии)
4. Возвращает займ
5. Оставляет прибыль себе

Всё за 0.4 секунды, 1000 раз в день!
Ожидаемая прибыль: $80-120k/день
═══════════════════════════════════════════
```

## 🏗️ Архитектура (простыми словами)

```
ОСНОВНЫЕ МОДУЛИ:
═══════════════════════════════════════════

1. PRICE WATCHER 👁️
   └─ Следит за ценой 24/7
   └─ Кричит если выходим из range

2. VOLUME ANALYZER 📊
   └─ Считает сколько $ торгуется
   └─ Говорит когда profitable

3. EXECUTOR 🚀
   └─ Строит транзакции
   └─ Отправляет их быстро
   └─ Использует Jito (анти-MEV)

4. POSITION MANAGER 🎯
   └─ Создает позиции
   └─ Делает rebalancing
   └─ Multi-range стратегии

5. RISK CONTROL 🛡️
   └─ Останавливает при убытках
   └─ Circuit breakers
   └─ Max loss = $10k/day
═══════════════════════════════════════════
```

## 💻 Технический стек

```yaml
Языки:
  основной: TypeScript
  критичные части: Rust (по желанию)
  
Блокчейн библиотеки:
  - @solana/web3.js
  - @orca-so/whirlpools-sdk
  - @meteora-ag/dlmm
  - jito-ts (для MEV защиты)

Инфраструктура:
  - RPC: Premium (Helius/Triton)
  - Cache: Redis
  - DB: PostgreSQL
  - Monitoring: Grafana

DevOps:
  - Docker
  - GitHub Actions
  - 3 реплики для надежности
```

## 📝 Основной функционал

### 1. Мониторинг цены

```typescript
// Следим за ценой в реальном времени
function watchPrice(pool: Pool) {
  // Обновление каждые 100ms
  // Alert если близко к границе range
  // Автоматический rebalance
}
```

### 2. Исполнение Flash Loan

```typescript
// Одна транзакция = 5 инструкций
function executeFlashLoanCycle() {
  const tx = new Transaction();
  
  tx.add(borrowFL($1_000_000));      // 1. Займ
  tx.add(addToPosition());            // 2. В позицию
  tx.add(collectFees());              // 3. Забрать fees
  tx.add(removeFromPosition());       // 4. Убрать
  tx.add(repayFL());                  // 5. Вернуть
  
  return sendViaJito(tx); // MEV защита
}
```

### 3. Расчет прибыльности

```typescript
// Считаем выгодно ли делать цикл
function isProfitable(pool: Pool): boolean {
  const volumePerSec = getVolume(pool);
  const ourShare = calculateShare();
  const expectedFees = volumePerSec * 0.4 * feeRate * ourShare;
  const costs = flFee + gas + slippage;
  
  return expectedFees > costs * 1.5; // 50% запас
}
```

## 🎯 Специфика для каждой DEX

```
ПОДДЕРЖКА 3 ПЛАТФОРМ:
═══════════════════════════════════════════

1. ORCA Whirlpools
   - NFT позиции
   - Несколько fee tiers
   - SDK готов

2. METEORA DLMM
   - Bin система (не NFT)
   - Самая быстрая
   - Дешевый gas

3. RAYDIUM CLMM
   - PDA позиции
   - Высокий volume
   - Средняя скорость
═══════════════════════════════════════════
```

## 🛡️ Безопасность и лимиты

```yaml
Circuit Breakers:
  max_daily_loss: $10,000
  max_failures_per_hour: 10
  max_slippage: 2%
  min_profit_ratio: 1.5x

Position Limits:
  max_position_size: $5M
  max_flash_loan: $10M
  max_concurrent_positions: 20

Emergency:
  - Автоматическая остановка
  - Вывод всей ликвидности
  - Уведомление админа
```

## 📊 Мониторинг

```
REAL-TIME МЕТРИКИ:
═══════════════════════════════════════════

Performance:
├─ Cycles/minute: 25
├─ Success rate: 92%
├─ Avg profit: $85
└─ Gas efficiency: $3.2

System Health:
├─ RPC latency: 45ms
├─ Memory: 2.1GB
├─ CPU: 35%
└─ Errors: 0.3%

Grafana Dashboard:
[████████░░] Daily P&L: $87,234
[█████████░] Success: 92%
[███████░░░] Positions: 14/20
═══════════════════════════════════════════
```

## 🚀 Этапы разработки

```
TIMELINE (6 недель):
═══════════════════════════════════════════

Неделя 1-2: Базовая инфраструктура
□ Подключение к RPC
□ Простой FL executor
□ Работа с 1 пулом

Неделя 3-4: Расширение функционала  
□ Multi-DEX поддержка
□ Position management
□ Rebalancing

Неделя 5: Risk & Monitoring
□ Circuit breakers
□ Grafana dashboards
□ Alert система

Неделя 6: Тестирование
□ Mainnet симуляция
□ Оптимизация
□ Документация
═══════════════════════════════════════════
```

## ✅ Что должно быть в итоге

```
DELIVERABLES:
═══════════════════════════════════════════

1. Рабочий бот
   - Success rate > 85%
   - Profit > $50k/day
   - Uptime > 99.9%

2. Документация
   - README с запуском
   - Конфиг примеры
   - API docs

3. Мониторинг
   - Grafana панели
   - Алерты
   - Логи

4. Тесты
   - Unit tests > 80%
   - Integration tests
   - Performance tests
═══════════════════════════════════════════
```

## 💡 Ключевые моменты

```
REMEMBER:
═══════════════════════════════════════════

• Всё в ОДНОЙ транзакции (atomic)
• Скорость критична (<400ms)
• MEV защита обязательна (Jito)
• Concentrated = 50-200x эффект
• Rebalance сразу при выходе из range
• Начать с 3 пулов, масштабировать до 12

Target: $2.4M прибыли в месяц!
═══════════════════════════════════════════
```

## 🎯 Quick Start для разработчика

```bash
# 1. Клонировать шаблон
git clone solana-fl-bot-template

# 2. Установить зависимости
npm install

# 3. Настроить конфиг
cp .env.example .env
# Добавить RPC_URL, PRIVATE_KEY

# 4. Запустить в тестовом режиме
npm run test:mainnet-fork

# 5. Начать с одного пула
POOLS=USDC/USDT npm run start
```