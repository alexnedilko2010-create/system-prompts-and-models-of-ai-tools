/**
 * Скрипт для проверки РЕАЛЬНЫХ комиссий Jupiter Perpetuals
 * 
 * Цель: Определить точную структуру комиссий:
 * 1. Есть ли фиксированная комиссия 0.06%?
 * 2. Есть ли динамическая price impact fee?
 * 3. Какая формула используется?
 */

import { Connection, PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

// Jupiter Perps Program ID (mainnet)
const JUPITER_PERPS_PROGRAM_ID = new PublicKey('PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu');

// Solana RPC endpoint
const RPC_URL = 'https://api.mainnet-beta.solana.com';

interface FeeCheckResult {
  positionSize: number;
  expectedFeeFixed: number;      // Если 0.06% фиксированная
  expectedFeeWithImpact?: number; // Если есть impact fee
  actualFee?: number;             // Реальная комиссия из quote
}

/**
 * Проверяет комиссии для разных размеров позиций
 */
async function checkJupiterFees() {
  console.log('🔍 Проверка структуры комиссий Jupiter Perpetuals\n');
  
  const connection = new Connection(RPC_URL, 'confirmed');
  
  // Размеры позиций для тестирования
  const positionSizes = [
    100_000,      // $100k
    1_000_000,    // $1M
    5_000_000,    // $5M
    10_000_000,   // $10M
    25_000_000,   // $25M
  ];
  
  const results: FeeCheckResult[] = [];
  
  for (const size of positionSizes) {
    console.log(`\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━`);
    console.log(`📊 Размер позиции: $${size.toLocaleString()}`);
    console.log(`━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━`);
    
    // Вариант 1: Фиксированная комиссия 0.06%
    const fixedFee = size * 0.0006; // 0.06%
    console.log(`Вариант 1 (фикс 0.06%): $${fixedFee.toLocaleString()}`);
    
    // Вариант 2: С price impact fee (моя старая формула)
    const impactFeeCoeff = 0.0006 + (size / 1_000_000_000);
    const feeWithImpact = size * impactFeeCoeff;
    console.log(`Вариант 2 (с impact): $${feeWithImpact.toLocaleString()}`);
    
    // Вариант 3: Фикс 0.06% + маленький impact (начинается с 0.01%)
    const smallImpactFee = size * 0.0006 + size * 0.0001; // 0.06% + 0.01%
    console.log(`Вариант 3 (0.06% + 0.01%): $${smallImpactFee.toLocaleString()}`);
    
    results.push({
      positionSize: size,
      expectedFeeFixed: fixedFee,
      expectedFeeWithImpact: feeWithImpact,
    });
    
    // TODO: Здесь нужно запросить реальный quote от Jupiter
    // Но для этого нужна полная интеграция с Jupiter SDK
    console.log(`⚠️  Реальная комиссия: НУЖЕН ЗАПРОС К JUPITER API`);
  }
  
  console.log('\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('📋 СВОДНАЯ ТАБЛИЦА');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
  
  console.table(results.map(r => ({
    'Position': `$${(r.positionSize / 1_000_000).toFixed(1)}M`,
    'Fixed 0.06%': `$${r.expectedFeeFixed.toLocaleString()}`,
    'With Impact': `$${r.expectedFeeWithImpact!.toLocaleString()}`,
    'Difference': `$${(r.expectedFeeWithImpact! - r.expectedFeeFixed).toLocaleString()}`
  })));
  
  console.log('\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
  console.log('💡 ВЫВОД:');
  console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
  
  console.log('Для позиции $25M разница составляет:');
  const diff = results[results.length - 1].expectedFeeWithImpact! - results[results.length - 1].expectedFeeFixed;
  console.log(`$${diff.toLocaleString()}`);
  console.log('\nЕсли такая разница есть - схема УБЫТОЧНА! ❌');
  console.log('Если такой разницы нет - схема ПРИБЫЛЬНА! ✅\n');
  
  console.log('🔧 Для точной проверки нужно:');
  console.log('1. Интегрировать Jupiter Perps SDK');
  console.log('2. Запросить реальный quote для позиции');
  console.log('3. Посмотреть фактическую комиссию\n');
  
  console.log('Или просто открыть позицию на app.jup.ag и посмотреть! 👀\n');
}

// Альтернативный подход: Анализ реальных транзакций
async function analyzeRealTransactions() {
  console.log('\n🔎 АЛЬТЕРНАТИВА: Анализ реальных транзакций\n');
  console.log('Можно найти крупные трейды на Jupiter Perps и посмотреть:');
  console.log('1. Размер позиции (notional)');
  console.log('2. Уплаченную комиссию');
  console.log('3. Вычислить процент\n');
  console.log('Примеры транзакций можно найти на:');
  console.log('- solscan.io');
  console.log('- solana.fm');
  console.log('- Фильтр по Jupiter Perps program ID\n');
}

// Запуск проверки
checkJupiterFees()
  .then(() => analyzeRealTransactions())
  .catch(console.error);
