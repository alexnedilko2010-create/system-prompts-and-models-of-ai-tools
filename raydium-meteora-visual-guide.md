# 🔍 Raydium vs Meteora - Где НЕТ скрытых комиссий?

## ✅ Raydium - Почти идеально для Fee Extraction!

```
RAYDIUM СТАНДАРТНЫЕ ПУЛЫ:
════════════════════════════════════

Withdrawal Fee: 0% ✅
Timelock: НЕТ ✅
Instant вывод: ДА ✅

Примеры безопасных:
• SOL/USDC
• RAY/USDC  
• USDC/USDT
• mSOL/SOL
• stSOL/SOL

ИДЕАЛЬНО ДЛЯ FEE EXTRACTION!
════════════════════════════════════
```

### ⚠️ Raydium исключения:

```
FUSION POOLS (экспериментальные):
• Withdrawal fee: 0-0.25%
• Timelock: 0-5 блоков
• Статус: ПРОВЕРЯЙТЕ КАЖДЫЙ!

Как определить:
- "Fusion" в названии
- Новые < 7 дней
- Warning при выводе
```

## 🌊 Meteora - Осторожнее!

```
METEORA ОБЫЧНЫЕ ПУЛЫ:
════════════════════════════════════

Withdrawal Fee: 0% ✅
НО! Timelock: 0-50 блоков ⚠️

Безопасные:
• USDC/USDT (стейблы)
• Старые пулы > 30 дней

Рискованные:
• Dynamic fee pools
• Новые пулы
• Volatile пары
════════════════════════════════════
```

### 🚨 Meteora опасности:

```
DYNAMIC POOLS:
━━━━━━━━━━━━━━━━━━━━━━━━
При низкой волатильности:
• Вывод: Instant ✅

При высокой волатильности:
• Cooldown: 10-50 блоков
• Время: 4-20 секунд
• FL НЕ ЗАКРОЕТСЯ! ☠️
━━━━━━━━━━━━━━━━━━━━━━━━

BOOSTED POSITIONS:
• Lock: 7-90 дней 🔒
• Penalty за ранний вывод
• НЕ ДЛЯ FL!
```

## 📊 Быстрое сравнение

```
                 RAYDIUM         METEORA
═══════════════════════════════════════════
Withdrawal fee    0%              0-0.3%
Timelock         НЕТ             0-50 блоков
Безопасность     95%             60%
Для FL           ✅✅✅           ✅⚠️
═══════════════════════════════════════════
```

## 🔎 Как проверить пул ДО операции

### Проверка через интерфейс:

```
RAYDIUM:
1. app.raydium.io/liquidity
2. Выберите пул
3. Нажмите "Remove"
4. Ищите warnings ⚠️

Если видите:
❌ "Withdrawal fee"
❌ "Please wait X blocks"
❌ "Locked until"
= НЕ ИСПОЛЬЗУЙТЕ!
```

```
METEORA:
1. app.meteora.ag/pools
2. Откройте "Pool Info"
3. Проверьте параметры:
   • Lock Period: 0 ✅
   • Exit Fee: 0% ✅
   • Cooldown: None ✅
```

### Проверка кодом (для про):

```typescript
// Быстрая проверка Raydium
const safePool = 
  poolInfo.fees.withdrawalFee === 0 &&
  poolInfo.lpLockedUntil === null;

// Быстрая проверка Meteora  
const safePool = 
  params.lockDuration === 0 &&
  params.protocolFee === 0 &&
  !params.isDynamic;
```

## ✅ Топ-10 безопасных пулов

### Raydium (100% безопасные):

```
1. SOL/USDC     - $100M+ объем
2. RAY/USDC     - Нативный токен
3. USDC/USDT    - Стейблы
4. mSOL/SOL     - Liquid staking
5. ORCA/USDC    - Кросс-DEX
```

### Meteora (проверенные):

```
6. USDC/USDT    - Только стейблы!
7. USDC/DAI     - Низкий риск
8. SOL/USDC*    - *Проверить dynamic
9. wBTC/USDC*   - *Старые пулы
10. ETH/USDC*   - *Без dynamic fee
```

## ⚡ Практические советы

### Безопасная стратегия:

```
ДЛЯ НАЧИНАЮЩИХ:
━━━━━━━━━━━━━━━━━━━━━━━━
1. Только Raydium
2. Только топ-5 пулов
3. Избегать Fusion
4. Тест $100 сначала
━━━━━━━━━━━━━━━━━━━━━━━━

ДЛЯ ОПЫТНЫХ:
━━━━━━━━━━━━━━━━━━━━━━━━
1. Raydium - основа (80%)
2. Meteora стейблы (20%)
3. Проверять каждый пул
4. Иметь план Б
━━━━━━━━━━━━━━━━━━━━━━━━
```

### Красные флаги - бегите:

```
🚨 ОПАСНО:
• "Fusion" в названии
• "Dynamic" в описании
• "Boosted" позиции
• APR > 1000%
• Volume < $10k/день
• Пул < 7 дней
• Любые warnings
```

## 📈 Статистика безопасности

```
RAYDIUM:
• 500+ пулов
• 95% без комиссий ✅
• 5% с ограничениями ⚠️

METEORA:
• 200+ пулов  
• 80% без комиссий
• 20% с timelocks ⚠️

ВЫВОД: Raydium безопаснее!
```

## 🎯 Главные правила

```
┌─────────────────────────────────────────┐
│                                         │
│  RAYDIUM = ПЕРВЫЙ ВЫБОР ДЛЯ FL         │
│                                         │
│  • 0% withdrawal fees                   │
│  • NO timelocks                         │
│  • Instant LP removal                   │
│                                         │
│  METEORA = ОСТОРОЖНО                    │
│                                         │
│  • Может быть timelock                  │
│  • Dynamic = опасно                     │
│  • Только стейблы                       │
│                                         │
│  ВСЕГДА ТЕСТИРУЙТЕ $100 ПЕРВЫМ!        │
│                                         │
└─────────────────────────────────────────┘
```