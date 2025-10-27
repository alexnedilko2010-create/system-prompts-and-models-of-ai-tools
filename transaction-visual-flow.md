# 🔄 Транзакция Flash Loan - Визуальный Flow

## 🎯 Общая схема транзакции

```
ОДНА АТОМАРНАЯ ТРАНЗАКЦИЯ (400ms):
═══════════════════════════════════════════

     START
       │
    ┌──▼──┐
    │ 0,1 │ Setup (Compute + Priority)
    └──┬──┘
       │
    ┌──▼──┐
    │  2  │ Borrow $1M Flash Loan
    └──┬──┘ Balance: +$1M
       │
    ┌──▼──┐
    │  3  │ Add to Concentrated LP
    └──┬──┘ Position: Active
       │
    ┌──▼──┐
    │  4  │ Collect Fees ($300)
    └──┬──┘ Profit secured!
       │
    ┌──▼──┐
    │  5  │ Remove FL portion
    └──┬──┘ Get back $1M
       │
    ┌──▼──┐
    │  6  │ Repay FL + $100 fee
    └──┬──┘ Debt cleared
       │
     END
   Profit: $200 ✅
═══════════════════════════════════════════
```

## 📝 Детали каждого шага

### Step 0-1: Подготовка транзакции

```
SETUP INSTRUCTIONS:
═══════════════════════════════════════════

Instruction 0: Compute Budget
┌─────────────────────────────┐
│ setComputeUnitLimit:        │
│ • Units: 1,400,000         │
│ • Why: Complex math needs   │
│ • Cost: ~0.001 SOL         │
└─────────────────────────────┘

Instruction 1: Priority Fee
┌─────────────────────────────┐
│ setComputeUnitPrice:        │
│ • Base: 1000 microLamports  │
│ • If congested: ×5          │
│ • Dynamic calculation       │
└─────────────────────────────┘
═══════════════════════════════════════════
```

### Step 2: Flash Loan Borrow

```
BORROW FLASH LOAN:
═══════════════════════════════════════════

From: Solend Pool
┌─────────────────────────────┐
│ 🏦 LIQUIDITY POOL           │
│ Available: $50M USDC        │
└──────────┬──────────────────┘
           │ Borrow $1M
           ▼
┌─────────────────────────────┐
│ 👛 YOUR WALLET              │
│ Before: $100 USDC           │
│ After: $1,000,100 USDC      │
└─────────────────────────────┘

Contract remembers: "Must repay in same TX!"
═══════════════════════════════════════════
```

### Step 3: Add Concentrated Liquidity

```
ADD TO CONCENTRATED POSITION:
═══════════════════════════════════════════

Your $1M in range ±0.05%:

Price Range:
0.9995 ←────[████████]────→ 1.0005
              ↑ Your LP

Effect:
• Normal LP: $1M = $1M
• Concentrated: $1M = $200M effect! 🚀
• Your share in range: 80%

Token Distribution:
• USDC: 500,000
• USDT: 499,750
(Exact ratio depends on current price)
═══════════════════════════════════════════
```

### Step 4: Collect Fees

```
COLLECT ACCUMULATED FEES:
═══════════════════════════════════════════

During 400ms cycle:
┌─────────────────────────────┐
│ 🔄 TRADING VOLUME           │
│ • Passed through: $500k     │
│ • Your share: 80%           │
│ • Volume for you: $400k     │
└─────────────────────────────┘
              │
              ▼
┌─────────────────────────────┐
│ 💰 FEES COLLECTED           │
│ • Rate: 0.25%               │
│ • From $400k = $1,000       │
│ • But realistically: $300   │
└─────────────────────────────┘

This is your PROFIT!
═══════════════════════════════════════════
```

### Step 5: Remove Liquidity

```
REMOVE FL PORTION:
═══════════════════════════════════════════

From Position:
┌─────────────────────────────┐
│ Your Concentrated LP:       │
│ • Total: $1M + fees         │
│ • Remove: $1M (FL part)     │
│ • Keep: $0 + fees           │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│ 👛 WALLET RECEIVES:         │
│ • USDC: ~500,100            │
│ • USDT: ~500,000            │
│ • Total: ~$1,000,100        │
└─────────────────────────────┘

Must be EXACT FL amount!
═══════════════════════════════════════════
```

### Step 6: Repay Flash Loan

```
REPAY FLASH LOAN:
═══════════════════════════════════════════

Your Wallet:
┌─────────────────────────────┐
│ Before repay:               │
│ • USDC: $1,000,400          │
│ • (FL + fees collected)     │
└──────────┬──────────────────┘
           │ Repay $1M + $100
           ▼
┌─────────────────────────────┐
│ After repay:                │
│ • USDC: $300                │
│ • Net profit: +$200 ✅      │
└─────────────────────────────┘

Solend: "Debt repaid, TX success!"
═══════════════════════════════════════════
```

## 🔧 Техническая структура

```typescript
// Полная транзакция
const transaction = new Transaction();

// Обязательный порядок инструкций:
transaction.add(computeBudget);     // 0
transaction.add(priorityFee);       // 1
transaction.add(borrowFL);          // 2
transaction.add(addLiquidity);      // 3
transaction.add(collectFees);       // 4
transaction.add(removeLiquidity);   // 5
transaction.add(repayFL);           // 6

// Отправка
const signature = await sendViaJito(transaction);
```

## ⚡ Критические моменты

```
CRITICAL POINTS:
═══════════════════════════════════════════

1. TIMING ⏱️
   • Total < 400ms
   • No delays between steps
   • All atomic

2. AMOUNTS 💰
   • Borrow: exactly $1M
   • Repay: exactly $1M + fee
   • Slippage: max 2%

3. RANGE 🎯
   • Must be in range
   • Tighter = more fees
   • But higher rebalance risk

4. GAS ⛽
   • Need ~0.01 SOL
   • Priority fee critical
   • Jito tip for MEV protection
═══════════════════════════════════════════
```

## 📊 Accounts структура (для каждой инструкции)

```
ACCOUNTS MAP:
═══════════════════════════════════════════

Flash Loan:
├─ Pool Account
├─ Pool Authority (PDA)
├─ Borrower (you)
├─ Token Accounts
└─ Oracle

Concentrated LP:
├─ Whirlpool/Pool
├─ Position (NFT/Account)
├─ Token Vaults (A & B)
├─ Tick Arrays (3)
├─ User Token Accounts
└─ Authority

Common:
├─ Token Program
├─ System Program
├─ Rent Sysvar
└─ Clock Sysvar
═══════════════════════════════════════════
```

## ✅ Проверки перед отправкой

```
PRE-FLIGHT CHECKLIST:
═══════════════════════════════════════════

□ Price in range?
□ Volume > $400k/sec?
□ FL available?
□ Gas balance OK?
□ Accounts correct?
□ Amounts calculated?
□ Slippage set?
□ Priority fee optimal?

IF ALL ✓ → SEND IT! 🚀
═══════════════════════════════════════════
```

## 🎯 Результат успешной транзакции

```
SUCCESS METRICS:
═══════════════════════════════════════════

Execution time: 387ms ✓
Gas used: 0.008 SOL
Jito tip: 0.002 SOL

Financial:
• FL borrowed: $1,000,000
• FL fee: -$100
• Fees collected: +$312
• Gas + tip: -$12
• NET PROFIT: +$200 ✅

Do this 1000× per day = $200,000!
═══════════════════════════════════════════
```