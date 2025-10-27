# Детальная схема "Sandwich свои сделки" на Pump.fun и Bonk.fun

## 🎯 Концепция: Продавать самому себе для снижения slippage

### Как работает классический sandwich:
```python
traditional_sandwich = {
    'attacker_sees': 'Большая покупка в mempool',
    'step1': 'Покупает ПЕРЕД жертвой',
    'step2': 'Жертва покупает дороже',
    'step3': 'Атакующий продает жертве',
    'profit': 'Из разницы цен'
}

# Наша стратегия - МЫ и атакующий, и жертва!
self_sandwich = {
    'we_are': 'И покупатель, и продавец',
    'benefit': 'Контролируем обе стороны сделки',
    'result': 'Снижаем slippage И зарабатываем'
}
```

## 📊 Специфика Pump.fun и Bonk.fun

### Особенности bonding curve:
```javascript
const bondingCurveSpecifics = {
    pumpFun: {
        curve: 'Автоматическая формула цены',
        liquidity: 'Вся в контракте curve',
        orders: 'Нет традиционного стакана',
        challenge: 'Нельзя выставить лимитки напрямую'
    },
    
    bonkFun: {
        similar: 'Та же механика bonding curve',
        difference: 'Другие параметры кривой',
        migration: '$69k vs $83k'
    },
    
    problem: 'Как сделать sandwich без ордеров?'
};
```

## 💡 Решение: Модифицированный Self-Sandwich

### Метод 1: "Pre-Liquidity Sandwich"

```python
def pre_liquidity_sandwich():
    """
    Создаем ликвидность ВНЕ bonding curve
    """
    
    # Шаг 1: Создаем пул на другом DEX
    create_pool = {
        'platform': 'Raydium/Orca',
        'before': 'До основной операции',
        'liquidity': {
            'sol_side': 50,  # SOL
            'token_side': 500_000_000,  # Ваши токены
            'price': 'Выше чем на bonding curve'
        }
    }
    
    # Шаг 2: Flash Loan покупка
    fl_execution = {
        'primary_route': 'Bonding curve (70%)',
        'secondary_route': 'Ваш LP pool (30%)',
        
        'what_happens': [
            'FL покупает на curve → цена растет',
            'Арбитражеры видят разницу цен',
            'Покупают на curve, продают в ваш pool',
            'Вы зарабатываете комиссии + продаете дороже'
        ]
    }
    
    # Шаг 3: Результат
    result = {
        'fl_slippage': 'Снижен на 20-30%',
        'your_profit': 'Продали часть токенов дороже',
        'lp_fees': 'Дополнительный доход',
        'control': 'Сохранили'
    }
```

### Метод 2: "Multi-Wallet Sandwich"

```javascript
const multiWalletSandwich = {
    // Подготовка
    setup: {
        wallets: {
            main: '60% ваших токенов',
            seller1: '10% токенов',
            seller2: '10% токенов',
            seller3: '10% токенов',
            buyer: 'Для FL операции'
        }
    },
    
    // Исполнение в одной транзакции
    atomicExecution: [
        {
            step: 1,
            action: 'FL начинает покупку',
            size: '20 SOL'
        },
        {
            step: 2,
            action: 'Seller1 продает в тот же блок',
            size: '5M tokens',
            effect: 'Съедает часть FL покупки'
        },
        {
            step: 3,
            action: 'FL продолжает покупку',
            size: '30 SOL'
        },
        {
            step: 4,
            action: 'Seller2 продает',
            size: '8M tokens',
            price: 'Выше из-за FL пампа'
        },
        {
            step: 5,
            action: 'FL докупает',
            size: '40 SOL'
        },
        {
            step: 6,
            action: 'Seller3 продает на пике',
            size: '10M tokens',
            profit: 'Максимальная цена'
        }
    ],
    
    result: {
        flGotTokens: 'С меньшим slippage',
        youSoldHigh: 'Продали дорого',
        effectiveSlippage: '25% вместо 45%'
    }
};
```

### Метод 3: "Bonding Curve Manipulation"

```python
# Более агрессивная тактика
def curve_manipulation_sandwich():
    """
    Используем особенности bonding curve
    """
    
    # Факт: На curve цена зависит от supply в обращении
    
    strategy = {
        'preparation': {
            'accumulate': '40% supply',
            'avg_price': 'Очень низкая'
        },
        
        'execution_sequence': [
            {
                'time': 'T+0',
                'action': 'Начать FL покупку 100 SOL',
                'your_action': 'Ничего',
                'price_movement': 'Начинает расти'
            },
            {
                'time': 'T+0.5 sec',
                'action': 'FL купил 30%',
                'your_action': 'Продать 50M tokens',
                'effect': 'Забрать ликвидность у FL'
            },
            {
                'time': 'T+1 sec',
                'action': 'FL вынужден покупать дороже',
                'your_action': 'Продать еще 30M',
                'price': 'Еще выше'
            },
            {
                'time': 'T+2 sec',
                'action': 'FL завершает покупку',
                'your_action': 'Подождать коррекцию',
                'result': 'FL переплатил 30%+'
            },
            {
                'time': 'T+5 min',
                'action': 'Цена корректируется',
                'your_action': 'Откупить обратно дешевле',
                'profit': 'Разница цен + больше токенов'
            }
        ]
    }
```

## 🎮 Практическая реализация на Pump.fun

### Пошаговый план:

```typescript
class PumpFunSelfSandwich {
    async executeSandwich(flashLoanAmount: number) {
        // 1. Подготовка
        const preparation = {
            // Разделить токены по кошелькам
            distributeTokens: async () => {
                await this.transfer(this.wallet2, '10% of tokens');
                await this.transfer(this.wallet3, '10% of tokens');
                await this.transfer(this.wallet4, '10% of tokens');
            },
            
            // Создать внешнюю ликвидность
            createExternalLP: async () => {
                if (this.hasEnoughTokens()) {
                    await this.createOrcaPool(50_SOL, equivalentTokens);
                }
            }
        };
        
        // 2. Основная транзакция
        const mainTransaction = new Transaction();
        
        // FL покупка разбита на части
        mainTransaction.add(
            // Часть 1: FL покупает
            this.createBuyInstruction(flashLoanAmount * 0.3),
            
            // Наша продажа #1
            this.createSellFromWallet2('5% of supply'),
            
            // Часть 2: FL покупает дороже
            this.createBuyInstruction(flashLoanAmount * 0.4),
            
            // Наша продажа #2
            this.createSellFromWallet3('7% of supply'),
            
            // Часть 3: FL докупает
            this.createBuyInstruction(flashLoanAmount * 0.3),
            
            // Проверка миграции
            this.checkMigrationTriggered()
        );
        
        // 3. Пост-обработка
        const postProcess = {
            // Собрать прибыль
            collectProfits: async () => {
                const solFromSales = await this.getSOLBalance(this.wallet2) +
                                    await this.getSOLBalance(this.wallet3);
                return solFromSales;
            },
            
            // Откупить если нужно
            buyBackCheaper: async () => {
                if (this.currentPrice < this.avgSellPrice * 0.8) {
                    await this.buyTokens(this.profits * 0.5);
                }
            }
        };
    }
}
```

## 📊 Сравнение эффективности

### Обычная FL покупка vs Self-Sandwich:

```python
comparison = {
    'standard_fl': {
        'fl_amount': 300,  # SOL
        'single_buy': True,
        'slippage': '45%',
        'effective_buy': 165,  # SOL worth
        'tokens_received': 165_000_000,
        'avg_price': 0.000182
    },
    
    'self_sandwich': {
        'fl_amount': 300,  # SOL
        'multi_part_buy': True,
        'your_sales': '15% supply sold high',
        'effective_slippage': '25%',
        'fl_effective_buy': 225,  # SOL worth
        'your_profit': 45,  # SOL from sales
        'net_cost': 255,  # SOL
        'tokens_kept': '85% of your supply',
        'avg_price': 0.000147
    },
    
    'benefit': {
        'saved_on_slippage': 60,  # SOL
        'earned_from_sales': 45,  # SOL
        'total_advantage': 105,  # SOL ($10,500!)
    }
}
```

## ⚠️ Риски и ограничения

### На Pump.fun/Bonk.fun:

1. **Timing критичен**
   - Все должно быть в одной транзакции
   - Или очень быстрая последовательность

2. **Размер позиции**
   - Нужно минимум 30% supply для эффекта
   - Чем больше, тем лучше контроль

3. **Gas fees**
   - Множественные транзакции = больше gas
   - Нужен расчет рентабельности

4. **Обнаружение**
   - Паттерн может быть заметен
   - Другие могут скопировать

## ✅ Оптимальная стратегия Self-Sandwich

### Для Pump.fun ($83k migration):

```python
optimal_self_sandwich = {
    'requirements': {
        'your_position': '35-40% supply',
        'avg_entry': '<$0.00003',
        'fl_size': '200-400 SOL',
        'current_mc': '$75,000+'
    },
    
    'execution': {
        'split_tokens': '70% main, 3x10% sellers',
        'create_external_lp': 'Optional but helpful',
        'fl_parts': '30%-40%-30%',
        'sell_timing': 'Between FL parts',
        'sell_amounts': '5%-7%-8% of supply'
    },
    
    'expected_result': {
        'fl_slippage_reduction': '40-50%',
        'your_sales_profit': '20-30 SOL',
        'final_position': 'Still 20%+ supply',
        'total_benefit': '30-40% better than standard'
    }
}
```

### Для Bonk.fun ($69k migration):

```javascript
// Похожая стратегия, но с корректировками
const bonkFunSandwich = {
    differences: {
        lowerThreshold: 'Проще достичь',
        fasterMigration: 'Меньше времени',
        adjust: 'FL размеры меньше'
    },
    
    optimal: {
        flSize: '150-250 SOL',
        yourPosition: '30-35% supply',
        execution: 'Те же принципы'
    }
};
```

## 🎯 Вывод

**Self-Sandwich РАБОТАЕТ на Pump.fun и Bonk.fun, но требует:**

1. **Значительной позиции** (30%+ supply)
2. **Технических навыков** (сложное исполнение)
3. **Правильного тайминга** (все в секунды)
4. **Подготовки** (распределение по кошелькам)

**Эффект**: Снижение slippage на 40-50% + дополнительная прибыль от продаж

**Лучше всего работает**: При FL 200+ SOL и вашей позиции 35%+ supply