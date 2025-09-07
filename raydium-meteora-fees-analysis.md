# 🔍 Withdrawal Fees и Timelock на Raydium и Meteora

## 📊 Raydium - Детальный анализ

### Стандартные AMM пулы Raydium:

```javascript
const raydiumStandardPools = {
    withdrawal_fees: {
        base_pools: {
            fee: '0%',
            examples: ['SOL/USDC', 'RAY/USDC', 'USDC/USDT'],
            note: 'Можно добавлять и убирать LP мгновенно'
        },
        
        concentrated_liquidity: {
            fee: '0%',
            type: 'CLMM pools',
            note: 'Новые concentrated pools тоже без комиссий'
        }
    },
    
    timelock: {
        standard: 'НЕТ timelocks',
        instant_withdrawal: true,
        blocks_to_wait: 0,
        perfect_for: 'Fee Extraction!'
    },
    
    special_cases: {
        fusion_pools: {
            description: 'Экспериментальные пулы',
            withdrawal_fee: '0-0.25%',
            timelock: 'Иногда 1-5 блоков',
            warning: 'Проверяйте каждый пул!'
        },
        
        farms: {
            lp_staking: 'Отдельно от LP',
            unstaking_fee: '0%',
            harvest_lock: 'Нет'
        }
    }
};
```

### Raydium Ecosystem Farms:

```python
raydium_farms = {
    'dual_yield_farms': {
        'lp_tokens': 'Можно вывести мгновенно',
        'staked_lp': {
            'withdrawal': 'Instant',
            'fee': '0%',
            'but': 'Теряете незаявленные rewards'
        }
    },
    
    'acceleraytor': {
        'type': 'IDO platform',
        'ray_staking': {
            'lock': '7-30 дней для участия',
            'not_affecting': 'Обычные LP позиции'
        }
    },
    
    'verdict': 'LP позиции НЕ заблокированы, только staking'
}
```

## 🌊 Meteora - Детальный анализ

### Meteora Dynamic Liquidity Market Maker (DLMM):

```javascript
const meteoraDLMM = {
    standard_pools: {
        withdrawal_fees: {
            base_fee: '0%',
            examples: ['SOL/USDC', 'USDT/USDC'],
            note: 'БЕЗ комиссий на вывод'
        },
        
        timelock: {
            has_timelock: 'ИНОГДА!',
            details: {
                normal_pools: '0 blocks',
                volatile_pools: '0-10 blocks',
                new_pools: 'До 50 blocks (~20 сек)',
                reason: 'Защита от манипуляций'
            }
        }
    },
    
    dynamic_pools: {
        special_feature: 'Dynamic fees',
        withdrawal: {
            during_low_volatility: 'Instant',
            during_high_volatility: {
                cooldown: '10-50 blocks',
                time: '4-20 секунд',
                why: 'Предотвращение атак'
            }
        }
    }
};
```

### Meteora Special Programs:

```python
meteora_special = {
    'liquidity_mining': {
        'regular_lp': 'No restrictions',
        'boosted_positions': {
            'lock_required': True,
            'duration': '7-90 days',
            'bonus_apr': '+50-200%',
            'early_withdrawal': {
                'allowed': True,
                'penalty': '10-50% of rewards'
            }
        }
    },
    
    'dynamic_vaults': {
        'type': 'Auto-rebalancing',
        'withdrawal': {
            'fee': '0.1-0.3%',
            'reason': 'Rebalancing costs',
            'timelock': '1-5 blocks'
        }
    },
    
    'lending_pools': {
        'not_regular_lp': True,
        'different_product': True,
        'ignore_for': 'Fee Extraction'
    }
}
```

## 📋 Сравнительная таблица

```python
comparison_table = {
    'raydium_vs_meteora': {
        'feature': ['Withdrawal Fee', 'Timelock', 'Best For'],
        
        'raydium_standard': {
            'withdrawal_fee': '0%',
            'timelock': 'None',
            'best_for': 'Fee Extraction ✅✅✅'
        },
        
        'raydium_fusion': {
            'withdrawal_fee': '0-0.25%',
            'timelock': '0-5 blocks',
            'best_for': 'Check each pool ⚠️'
        },
        
        'meteora_standard': {
            'withdrawal_fee': '0%',
            'timelock': '0-10 blocks',
            'best_for': 'Good but check ✅✅'
        },
        
        'meteora_dynamic': {
            'withdrawal_fee': '0-0.3%',
            'timelock': '10-50 blocks',
            'best_for': 'Avoid for FL ⚠️'
        }
    }
}
```

## 🎯 Лучшие пулы для Fee Extraction

### Raydium - Топ выбор:

```javascript
const raydiumBestPools = {
    tier1_perfect: {
        pools: [
            'SOL/USDC',
            'RAY/USDC', 
            'USDC/USDT',
            'mSOL/SOL',
            'stSOL/SOL'
        ],
        why: 'Zero fees, instant withdrawal, high volume',
        rating: '10/10 для Fee Extraction'
    },
    
    tier2_good: {
        pools: [
            'ORCA/USDC',
            'SRM/USDC',
            'COPE/USDC'
        ],
        why: 'Zero fees, lower volume',
        rating: '8/10'
    },
    
    avoid: {
        pools: ['New experimental', 'Fusion test pools'],
        why: 'Могут иметь ограничения'
    }
};
```

### Meteora - Выборочно:

```python
meteora_recommendations = {
    'safe_choices': {
        'pools': [
            'USDC/USDT (stable)',
            'SOL/USDC (standard DLMM)'
        ],
        'check': 'Всегда проверяйте текущие параметры',
        'rating': '7/10 для Fee Extraction'
    },
    
    'risky_but_profitable': {
        'dynamic_fee_pools': {
            'pros': 'Fees до 2% в волатильность',
            'cons': 'Может быть timelock',
            'strategy': 'Только если можете ждать 20 сек'
        }
    },
    
    'avoid_for_fl': {
        'boosted_farms': 'Lock periods',
        'dynamic_vaults': 'Exit fees',
        'new_pools_24h': 'Unknown parameters'
    }
}
```

## ⚡ Как проверить конкретный пул

### Проверка через UI:

```javascript
const howToCheck = {
    raydium: {
        steps: [
            '1. Откройте app.raydium.io/liquidity',
            '2. Найдите пул',
            '3. Нажмите "Remove Liquidity"',
            '4. Посмотрите warnings (если есть)'
        ],
        
        red_flags: [
            '"Withdrawal fee" упоминание',
            '"Please wait X blocks"',
            '"Locked until"'
        ]
    },
    
    meteora: {
        steps: [
            '1. app.meteora.ag/pools',
            '2. Выберите пул',
            '3. Проверьте "Pool Info"',
            '4. Ищите "Dynamic Parameters"'
        ],
        
        look_for: {
            'Lock Period': 'Should be 0',
            'Exit Fee': 'Should be 0%',
            'Cooldown': 'Should be None'
        }
    }
};
```

### Проверка через код:

```typescript
// Проверка Raydium пула
async function checkRaydiumPool(poolAddress: PublicKey) {
    const poolInfo = await Liquidity.fetchInfo({
        connection,
        poolKeys: poolAddress
    });
    
    console.log({
        withdrawalFee: poolInfo.fees.withdrawalFee, // Should be 0
        lpLockedUntil: poolInfo.lpLockedUntil, // Should be null
        instantWithdraw: poolInfo.lpLockedUntil === null
    });
}

// Проверка Meteora пула
async function checkMeteoraPool(poolAddress: PublicKey) {
    const pool = await DLMM.create(connection, poolAddress);
    const params = await pool.getPoolParameters();
    
    console.log({
        lockBlocks: params.lockDuration, // Check if > 0
        withdrawFee: params.protocolFee, // Should be 0
        dynamicParams: params.isDynamic // If true, be careful
    });
}
```

## ✅ Практические рекомендации

### Для Fee Extraction используйте:

```python
recommended_setup = {
    'primary_choice': {
        'dex': 'Raydium',
        'pools': 'Standard AMM pools',
        'why': [
            'Guaranteed 0% withdrawal fee',
            'No timelocks ever',
            'Highest volume',
            'Battle-tested'
        ]
    },
    
    'secondary_choice': {
        'dex': 'Meteora',
        'pools': 'Stable pairs only',
        'caution': 'Always verify no lock',
        'why': 'Good for diversification'
    },
    
    'workflow': {
        'step1': 'Start with Raydium majors',
        'step2': 'Test each pool with $100',
        'step3': 'Verify instant withdrawal works',
        'step4': 'Scale up gradually'
    }
}
```

### Красные флаги - избегайте:

```javascript
const avoidThesePools = {
    raydium: [
        'Pools с "Fusion" в названии',
        'Экспериментальные пулы < 7 дней',
        'Любые с warning при выводе'
    ],
    
    meteora: [
        'Dynamic pools в высокой волатильности',
        'Boosted positions (locked)',
        'Auto-vaults с exit fees',
        'Новые пулы без истории'
    ],
    
    universal_red_flags: [
        'APR > 1000% (часто имеют locks)',
        '"Governance" в названии',
        'Low liquidity < $100k',
        'No volume history'
    ]
};
```

## 📊 Статистика по платформам

```python
platform_stats = {
    'raydium': {
        'total_pools': '~500 active',
        'without_fees': '95%+',
        'with_timelock': '<5%',
        'safe_for_fl': '90%+'
    },
    
    'meteora': {
        'total_pools': '~200 active',
        'without_fees': '80%',
        'with_timelock': '20%',
        'safe_for_fl': '60%'
    },
    
    'recommendation': 'Raydium safer for Fee Extraction'
}
```

## 🎯 Итоговые выводы

**Raydium:**
- ✅ 95%+ пулов БЕЗ withdrawal fees
- ✅ НЕТ timelocks в стандартных пулах
- ✅ Идеально для Fee Extraction
- ⚠️ Проверяйте Fusion пулы

**Meteora:**
- ✅ 80% пулов без fees
- ⚠️ Могут быть timelocks (10-50 блоков)
- ⚠️ Dynamic pools рискованны
- ✅ Stable pairs обычно безопасны

**Лучшая стратегия:**
1. Используйте Raydium для основного объема
2. Meteora только для проверенных stable pairs
3. ВСЕГДА тестируйте маленькой суммой сначала
4. Избегайте пулов младше 7 дней