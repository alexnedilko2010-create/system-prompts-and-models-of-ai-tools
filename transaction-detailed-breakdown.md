# 🔧 Детальная структура транзакции Flash Loan + Concentrated Liquidity

## 🏗️ Анатомия транзакции

### Общая структура

```typescript
interface FlashLoanCycleTransaction {
  // Метаданные транзакции
  metadata: {
    recentBlockhash: string;
    feePayer: PublicKey;
    signatures: Signature[];
  };
  
  // Инструкции (должны идти в строгом порядке)
  instructions: [
    ComputeBudgetInstruction,      // 0. Установка лимитов
    PriorityFeeInstruction,        // 1. Приоритет
    FlashLoanBorrowInstruction,    // 2. Займ FL
    AddLiquidityInstruction,       // 3. Добавить в LP
    CollectFeesInstruction,        // 4. Собрать fees
    RemoveLiquidityInstruction,    // 5. Убрать LP
    FlashLoanRepayInstruction      // 6. Вернуть FL
  ];
  
  // Address lookup tables для оптимизации
  addressLookupTables: AddressLookupTable[];
}
```

## 📝 Детальный разбор каждой инструкции

### 0. Compute Budget (Обязательно первой!)

```typescript
// Инструкция 0: Установка вычислительных лимитов
const computeBudgetIx = ComputeBudgetProgram.setComputeUnitLimit({
  units: 1_400_000  // Нужно много для сложных операций
});

// Почему важно:
// - Concentrated liquidity требует сложных вычислений
// - Default 200k units недостаточно
// - Лучше заплатить больше, чем получить ошибку
```

### 1. Priority Fee (Для скорости включения)

```typescript
// Инструкция 1: Динамическая priority fee
const priorityFeeIx = ComputeBudgetProgram.setComputeUnitPrice({
  microLamports: calculateDynamicPriorityFee({
    baseRate: 1000,
    congestionMultiplier: getCurrentCongestion(),
    profitabilityBoost: expectedProfit / 100
  })
});

// Пример расчета:
// Base: 1000 microLamports
// При high congestion: × 5 = 5000
// При profit $500: + 5 = 5005
// Total priority: ~0.007 SOL
```

### 2. Flash Loan Borrow

```typescript
// Инструкция 2: Займ Flash Loan
const flashLoanBorrowIx = await buildFlashLoanBorrowInstruction({
  // Программа FL провайдера (Solend/Port)
  programId: SOLEND_PROGRAM_ID,
  
  // Аккаунты
  accounts: {
    // Источник ликвидности
    liquidityPool: new PublicKey("Pool..."),
    liquidityPoolAuthority: findPDA(["authority"], SOLEND_PROGRAM_ID),
    
    // Получатель займа (наш кошелек)
    borrower: wallet.publicKey,
    borrowerTokenAccount: getAssociatedTokenAddress(USDC_MINT, wallet.publicKey),
    
    // Токен и оракул
    tokenMint: USDC_MINT,
    tokenProgram: TOKEN_PROGRAM_ID,
    priceOracle: PYTH_USDC_ORACLE,
    
    // Системные
    clock: SYSVAR_CLOCK_PUBKEY,
    rent: SYSVAR_RENT_PUBKEY
  },
  
  // Данные инструкции
  data: {
    instruction: FlashLoanInstruction.Borrow,
    amount: new BN(1_000_000_000_000), // 1M USDC (6 decimals)
    expectedFee: new BN(100_000_000)    // 100 USDC fee
  }
});

// После этой инструкции:
// - На нашем счету +1M USDC
// - Программа ожидает возврат в этой же TX
```

### 3. Add Concentrated Liquidity

```typescript
// Инструкция 3: Добавление в concentrated позицию
const addLiquidityIx = await buildAddConcentratedLiquidityInstruction({
  // Для Orca Whirlpools
  programId: ORCA_WHIRLPOOL_PROGRAM_ID,
  
  accounts: {
    // Пул и позиция
    whirlpool: poolAddress,
    position: positionAddress,
    positionTokenAccount: positionNFTAccount, // NFT для Orca
    
    // Наши токен аккаунты
    tokenOwnerAccountA: userUSDCAccount,  // Наш USDC (с FL)
    tokenOwnerAccountB: userUSDTAccount,  // Наш USDT
    
    // Vault'ы пула
    tokenVaultA: poolVaultUSDC,
    tokenVaultB: poolVaultUSDT,
    
    // Tick arrays для concentrated range
    tickArrayLower: getTickArray(tickLower),
    tickArrayUpper: getTickArray(tickUpper),
    tickArray0: getCurrentTickArray(),
    
    // Авторизация
    positionAuthority: wallet.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID
  },
  
  data: {
    // Расчет оптимального соотношения токенов
    liquidityAmount: calculateLiquidityAmount({
      currentPrice: pool.sqrtPrice,
      lowerPrice: tickToPrice(tickLower),
      upperPrice: tickToPrice(tickUpper),
      amount0: 1_000_000_000_000, // 1M USDC
      amount1: 999_500_000_000    // ~999.5k USDT (зависит от range)
    }),
    
    // Максимальные amounts (slippage protection)
    amount0Max: new BN(1_000_000_000_000),
    amount1Max: new BN(1_000_000_000_000),
    
    // Tick range для concentrated позиции
    tickLowerIndex: -5,  // Цена 0.9995
    tickUpperIndex: 5    // Цена 1.0005
  }
});

// Специфика concentrated:
// - Liquidity сконцентрирована в узком range
// - Эффективность капитала 100-200x
// - Требует точного расчета amounts
```

### 4. Collect Fees (Критично для прибыли!)

```typescript
// Инструкция 4: Сбор накопленных fees
const collectFeesIx = await buildCollectFeesInstruction({
  programId: ORCA_WHIRLPOOL_PROGRAM_ID,
  
  accounts: {
    whirlpool: poolAddress,
    position: positionAddress,
    positionTokenAccount: positionNFTAccount,
    
    // Куда отправить fees
    tokenOwnerAccountA: userUSDCAccount,
    tokenOwnerAccountB: userUSDTAccount,
    
    tokenVaultA: poolVaultUSDC,
    tokenVaultB: poolVaultUSDT,
    
    positionAuthority: wallet.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID
  },
  
  data: {
    // Обычно собираем все доступные fees
    amount0Requested: u64::MAX,
    amount1Requested: u64::MAX
  }
});

// Важно:
// - Fees накапливаются даже за 400ms
// - В concentrated позиции fees больше
// - Это наша прибыль!
```

### 5. Remove Liquidity (Точно FL сумму)

```typescript
// Инструкция 5: Удаление ликвидности
const removeLiquidityIx = await buildRemoveLiquidityInstruction({
  programId: ORCA_WHIRLPOOL_PROGRAM_ID,
  
  accounts: {
    // Те же аккаунты что и при добавлении
    whirlpool: poolAddress,
    position: positionAddress,
    // ... остальные аккаунты
  },
  
  data: {
    // КРИТИЧНО: убираем ровно столько, сколько заняли
    liquidityAmount: flashLoanLiquidityAmount,
    
    // Минимальные amounts (slippage protection)
    amount0Min: calculateMinAmount(1_000_000_000_000, 0.02), // 2% slippage
    amount1Min: calculateMinAmount(999_500_000_000, 0.02)
  }
});

// Расчет точной суммы:
// - Должны вернуть ровно FL amount
// - Учесть возможный slippage
// - Оставить базовую позицию (если есть)
```

### 6. Flash Loan Repay

```typescript
// Инструкция 6: Возврат Flash Loan
const flashLoanRepayIx = await buildFlashLoanRepayInstruction({
  programId: SOLEND_PROGRAM_ID,
  
  accounts: {
    liquidityPool: poolAddress,
    borrower: wallet.publicKey,
    borrowerTokenAccount: userUSDCAccount,
    // ... остальные аккаунты
  },
  
  data: {
    instruction: FlashLoanInstruction.Repay,
    repayAmount: new BN(1_000_100_000_000), // Principal + Fee
    
    // Проверочные данные
    borrowAmount: new BN(1_000_000_000_000),
    feeAmount: new BN(100_000_000)
  }
});

// Проверки в смарт-контракте:
// - Вернули principal + fee
// - В той же транзакции
// - Правильный borrower
```

## 🔄 Полная транзакция в коде

```typescript
async function buildCompleteFlashLoanCycle(
  pool: ConcentratedPool,
  position: Position,
  flashLoanAmount: number
): Promise<Transaction> {
  const transaction = new Transaction();
  
  // 0. Compute budget (ВСЕГДА первой)
  transaction.add(
    ComputeBudgetProgram.setComputeUnitLimit({
      units: 1_400_000
    })
  );
  
  // 1. Priority fee
  transaction.add(
    ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: await calculateOptimalPriorityFee()
    })
  );
  
  // 2. Borrow flash loan
  const borrowIx = await flashLoanProvider.createBorrowInstruction({
    amount: flashLoanAmount,
    borrower: wallet.publicKey
  });
  transaction.add(borrowIx);
  
  // 3. Add to concentrated position
  const addIx = await dexAdapter.createAddLiquidityInstruction({
    pool,
    position,
    amounts: calculateOptimalAmounts(flashLoanAmount, pool.currentPrice),
    tickRange: position.tickRange
  });
  transaction.add(addIx);
  
  // 4. Collect any fees
  const collectIx = await dexAdapter.createCollectFeesInstruction({
    position,
    recipient: wallet.publicKey
  });
  transaction.add(collectIx);
  
  // 5. Remove liquidity (FL portion)
  const removeIx = await dexAdapter.createRemoveLiquidityInstruction({
    position,
    liquidityAmount: calculateFlashLoanLiquidity(flashLoanAmount)
  });
  transaction.add(removeIx);
  
  // 6. Repay flash loan
  const repayIx = await flashLoanProvider.createRepayInstruction({
    amount: flashLoanAmount,
    fee: flashLoanAmount * 0.0001, // 0.01% fee
    borrower: wallet.publicKey
  });
  transaction.add(repayIx);
  
  // Оптимизация размера транзакции
  transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  transaction.feePayer = wallet.publicKey;
  
  // Address lookup tables для экономии места
  const lookupTables = await getAddressLookupTables([
    pool.address,
    position.address,
    USDC_MINT,
    USDT_MINT
  ]);
  transaction.addressLookupTables = lookupTables;
  
  return transaction;
}
```

## ⚠️ Критические проверки

### Pre-flight проверки перед отправкой:

```typescript
async function validateTransaction(tx: Transaction): Promise<boolean> {
  // 1. Размер транзакции
  const txSize = tx.serialize().length;
  if (txSize > 1232) {
    throw new Error(`TX too large: ${txSize} bytes`);
  }
  
  // 2. Правильный порядок инструкций
  const expectedOrder = [
    'ComputeBudget',
    'PriorityFee',
    'FlashLoanBorrow',
    'AddLiquidity',
    'CollectFees',
    'RemoveLiquidity',
    'FlashLoanRepay'
  ];
  
  // 3. Проверка profitability
  const simulation = await connection.simulateTransaction(tx);
  const estimatedFees = parseFeesFromLogs(simulation.logs);
  const costs = calculateTotalCosts(tx);
  
  if (estimatedFees < costs * 1.5) {
    throw new Error('Not profitable enough');
  }
  
  // 4. Проверка позиции в range
  const currentPrice = await pool.getCurrentPrice();
  if (!isPositionInRange(position, currentPrice)) {
    throw new Error('Position out of range');
  }
  
  return true;
}
```

## 🚀 Отправка через Jito

```typescript
async function sendViaJito(
  transaction: Transaction,
  expectedProfit: number
): Promise<string> {
  // Подписываем транзакцию
  transaction.sign(wallet);
  
  // Создаем bundle для Jito
  const bundle = {
    transactions: [transaction],
    
    // Tip для Jito (процент от профита)
    tip: Math.max(
      10_000,  // Минимум 0.00001 SOL
      Math.floor(expectedProfit * 0.01)  // 1% от профита
    )
  };
  
  // Отправляем через Jito RPC
  const response = await jitoClient.sendBundle(bundle);
  
  // Ждем подтверждения
  const confirmation = await jitoClient.confirmBundle(
    response.bundleId,
    {
      commitment: 'confirmed',
      maxRetries: 3
    }
  );
  
  return confirmation.signature;
}
```

## 📊 Мониторинг результатов

```typescript
interface TransactionResult {
  signature: string;
  
  // Финансовые результаты
  flashLoanAmount: number;
  feesCollected: number;
  totalCosts: {
    flashLoanFee: number;
    gas: number;
    slippage: number;
    jitoTip: number;
  };
  netProfit: number;
  
  // Метрики производительности
  executionTime: number;
  blockNumber: number;
  
  // Данные позиции
  position: {
    address: string;
    tickRange: [number, number];
    liquidity: number;
    inRange: boolean;
  };
}

// Парсинг результатов из логов
async function parseTransactionResult(
  signature: string
): Promise<TransactionResult> {
  const tx = await connection.getTransaction(signature, {
    maxSupportedTransactionVersion: 0
  });
  
  // Парсим логи для получения точных данных
  const logs = tx.meta.logMessages;
  const feesCollected = parseFeesFromLogs(logs);
  const gasUsed = tx.meta.fee;
  
  return {
    signature,
    feesCollected,
    // ... остальные поля
  };
}
```

## ✅ Чек-лист для транзакции

```
PRE-EXECUTION:
□ Позиция в range
□ Volume достаточный
□ FL доступен
□ Gas в кошельке

TRANSACTION BUILD:
□ Compute budget первой
□ Priority fee оптимальная
□ Все accounts правильные
□ Amounts точно рассчитаны
□ Slippage limits установлены

POST-EXECUTION:
□ Profit > 0
□ Позиция здорова
□ Логи сохранены
□ Метрики обновлены
```