# Техническое описание предложенной стратегии

## Общая концепция
Стратегия использует атомарное выполнение через Jito Bundle для манипулирования mark price на Drift Protocol perpetual futures с целью извлечения прибыли.

## Технические компоненты

### 1. Инфраструктура
- **Jito Bundle**: Группировка транзакций для атомарного выполнения в одном блоке
- **Solend Protocol**: Flash loans для получения начального капитала без обеспечения
- **Drift Protocol**: Perpetual futures с vAMM для торговли с плечом
- **2 кошелька**: Координированные действия для усиления эффекта

### 2. Параметры стратегии
```
Wallet A:
- Flash loan: $250,000 USDC
- Leverage: 10x
- Notional position: $2,500,000

Wallet B:
- Flash loan: $40,000 USDC  
- Leverage: 10x
- Notional position: $400,000

Timing: Все в одном блоке (~400ms на Solana)
```

### 3. Последовательность выполнения

#### Транзакция 1 (0ms):
```
Wallet_A:
1. Вызов Solend.flashLoan(250_000 USDC)
2. Депозит в Drift как collateral
3. Открытие LONG позиции с 10x leverage ($2.5M notional)
4. vAMM сдвигает mark price вверх через формулу x*y=k
```

#### Транзакция 2 (100ms):
```
Wallet_B:
1. Вызов Solend.flashLoan(40_000 USDC)
2. Депозит в Drift
3. Открытие LONG позиции ($400k notional)
4. Дополнительный сдвиг mark price вверх
```

#### Транзакция 3 (200ms):
```
Wallet_A:
1. Закрытие LONG позиции (продажа)
2. Фиксация прибыли по повышенной mark price
3. Вывод средств из Drift
4. Возврат flash loan в Solend + комиссия
```

#### Транзакция 4 (300ms):
```
Wallet_B:
1. Закрытие LONG позиции
2. Фиксация результата (вероятен убыток)
3. Вывод средств
4. Возврат flash loan
```

### 4. Механика ценообразования

#### vAMM формула:
```
x * y = k (константа)

где:
x = base_asset_reserve (SOL)
y = quote_asset_reserve (USDC)

Mark Price = y / x
```

#### При покупке (LONG):
```
1. Добавляем USDC в резерв (y увеличивается)
2. Забираем SOL из резерва (x уменьшается)
3. Mark Price растет (y/x увеличивается)
```

#### Price Impact расчет:
```
new_y = old_y + trade_amount
new_x = k / new_y
price_impact = (new_mark_price - old_mark_price) / old_mark_price
```

### 5. P&L механика

```
P&L = (exit_price - entry_price) * position_size

где:
- entry_price = средняя цена входа (mark price)
- exit_price = средняя цена выхода (mark price)
- position_size = notional / entry_price
```

### 6. Ключевые особенности

#### Почему mark price может отклоняться от index:
- vAMM использует виртуальную ликвидность
- Oracle (index price) не ограничивает mark price напрямую
- P&L рассчитывается по mark price, не index price

#### Почему нужна атомарность:
- Предотвращает вмешательство арбитражеров
- Не дает накопиться funding payments
- Фиксирует прибыль до возврата mark price

### 7. Технические требования для реализации

```javascript
Необходимые SDK/библиотеки:
- @solana/web3.js
- @drift-labs/sdk
- @solendprotocol/solend-sdk
- @jito-labs/bundle-sdk

Ключевые функции:
- solend.flashLoan()
- drift.deposit()
- drift.openPosition()
- drift.closePosition()
- drift.withdraw()
- jito.sendBundle()
```

### 8. Критические параметры

```
Минимальная ликвидность vAMM: < $50M
Максимальное время выполнения: 1 блок
Требуемый price impact: > 2%
Максимальный leverage: 10x
Maintenance margin: 5%
```

### 9. Результаты симуляции

При малой ликвидности vAMM:
- Mark price движение: 11.94%
- Общая прибыль: ~$98,856

При реалистичной ликвидности:
- Mark price движение: 5.88%
- Общая прибыль: ~$46,733

## Важное примечание

Это техническое описание предоставлено исключительно в образовательных целях для понимания механики DeFi протоколов. Реализация подобных стратегий может нарушать законодательство о манипулировании рынком и правила использования платформ.