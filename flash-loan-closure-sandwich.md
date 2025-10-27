# 🔒 Закрытие Flash Loan при Self-Sandwich стратегии

## ⚡ Ключевой вопрос: Откуда SOL для возврата FL?

### 📊 Механика закрытия Flash Loan

```python
flash_loan_lifecycle = {
    'borrow': 300,  # SOL
    'must_return': 301.5,  # SOL + 0.5% fee
    
    'question': 'Если мы покупаем токены, откуда SOL для возврата?',
    'answer': 'Из ПРОДАЖИ токенов после миграции!'
}
```

## 🎯 Полный цикл Self-Sandwich с FL

### Шаг 1: Структура операции

```javascript
const atomicTransaction = {
    // 1. Берем Flash Loan
    flashLoanBorrow: {
        amount: 300,  // SOL
        fee: 1.5,     // SOL (0.5%)
        mustReturn: 301.5  // SOL
    },
    
    // 2. Покупаем + Self-Sandwich
    buyWithSandwich: {
        flBuys: 300,  // SOL тратим
        yourSales: {
            wallet2: 'Продает 50M токенов за 50 SOL',
            wallet3: 'Продает 60M токенов за 60 SOL',
            wallet4: 'Продает 40M токенов за 48 SOL'
        },
        totalYourSOL: 158  // SOL от ваших продаж
    },
    
    // 3. Миграция произошла!
    
    // 4. Продаем часть FL токенов
    flSellAfterMigration: {
        sellAmount: '40% купленных токенов',
        expectedSOL: 180,  // SOL от продажи
        
        // ВАЖНО: 158 + 180 = 338 SOL
        // Хватает вернуть 301.5 SOL!
    },
    
    // 5. Возвращаем Flash Loan
    returnFlashLoan: 301.5  // SOL
};
```

## 💡 Почему Self-Sandwich НЕ ломает FL?

### Вариант A: FL покупает ваши токены

```python
def sandwich_scenario_a():
    """FL покупает токены, которые вы продаете"""
    
    # FL потратил 300 SOL на:
    fl_bought = {
        'from_bonding_curve': '60% (180 SOL)',
        'from_your_wallets': '40% (120 SOL)',  # Купил ВАШИ токены!
        'total_tokens': '400M tokens'
    }
    
    # Вы получили:
    you_received = {
        'from_fl_purchases': 120,  # SOL
        'from_other_buyers': 38,   # SOL (FOMO покупатели)
        'total': 158  # SOL
    }
    
    # После миграции:
    post_migration = {
        'fl_sells': '160M tokens за 180 SOL',
        'fl_keeps': '240M tokens',
        'fl_total_sol': 180,  # От продажи
        
        'can_return_loan': True  # 180 SOL хватит!
    }
    
    return "FL закроется успешно!"
```

### Вариант B: Комбинированная ликвидность

```javascript
const scenarioB = {
    // FL покупает из разных источников
    sources: {
        bondingCurve: '200 SOL worth',
        yourSales: '100 SOL worth',
        
        // Вы продали токены и получили SOL
        yourSOL: 100
    },
    
    // После миграции FL может:
    options: [
        {
            strategy: 'Продать 35%',
            getSOL: 175,
            returnLoan: 'Не хватает 126.5 SOL'
        },
        {
            strategy: 'Продать 45%',
            getSOL: 225,
            returnLoan: 'Не хватает 76.5 SOL'
        },
        {
            strategy: 'Продать 60%',
            getSOL: 302,
            returnLoan: 'Успех! Остаток 0.5 SOL'
        }
    ]
};
```

## 🔄 Реальный пример с точными расчетами

### Исходные данные:
```
MC до операции: $75,000
FL размер: 300 SOL
Ваша позиция: 400M токенов (40% supply)
Цель: Миграция на $83,000
```

### Детальное исполнение:

```python
execution_flow = {
    # T+0: Начало
    'initial_state': {
        'token_price': 0.000075,
        'your_tokens': 400_000_000,
        'fl_sol': 300
    },
    
    # T+1: FL покупка с sandwich
    'during_execution': {
        'fl_buys_round1': {
            'spends': 100,  # SOL
            'gets': '120M tokens',
            'price_impact': '→ $0.000082'
        },
        
        'your_sell1': {
            'from_wallet2': '50M tokens',
            'get_sol': 41,  # SOL по цене 0.000082
            'to_whom': 'FL + арбитражеры'
        },
        
        'fl_buys_round2': {
            'spends': 100,  # SOL
            'gets': '100M tokens',
            'price_impact': '→ $0.000091'
        },
        
        'your_sell2': {
            'from_wallet3': '60M tokens',
            'get_sol': 54.6,  # SOL по цене 0.000091
            'to_whom': 'FL + новые покупатели'
        },
        
        'fl_buys_round3': {
            'spends': 100,  # SOL
            'gets': '80M tokens',
            'price_impact': '→ $0.000100 = MIGRATION!'
        }
    },
    
    # T+2: После миграции
    'post_migration': {
        'fl_has': '300M tokens',
        'fl_needs': 301.5,  # SOL для возврата
        
        'fl_sells': {
            'amount': '180M tokens (60%)',
            'at_price': 0.000095,  # Небольшой дамп
            'gets_sol': 171  # SOL
        },
        
        'your_sandwich_profit': {
            'sold_tokens': '110M',
            'received_sol': 95.6,
            'avg_price': 0.000087
        },
        
        'problem': 'FL получил только 171 SOL, нужно 301.5!'
    }
}
```

## ❌ ПРОБЛЕМА: FL может не закрыться!

### Почему FL может провалиться:

```python
fl_failure_analysis = {
    'fl_spent': 300,  # SOL на покупку
    'fl_must_return': 301.5,  # SOL долг
    
    'fl_tokens': '300M',
    'fl_can_sell': '60-70% max',  # Больше = обвал цены
    'fl_gets_from_sale': '170-200 SOL',
    
    'shortage': '101.5 - 131.5 SOL',
    
    'result': 'FL НЕ МОЖЕТ ЗАКРЫТЬСЯ!'
}
```

## ✅ РЕШЕНИЕ: Модифицированная стратегия

### Вариант 1: FL + Ваши SOL

```javascript
const hybridSolution = {
    setup: {
        yourTokens: '400M (40% supply)',
        yourSOL: 150,  // Дополнительные SOL
        flashLoan: 200  // Меньший FL!
    },
    
    execution: {
        // Комбинируем FL + ваши SOL
        totalBuyPower: 350,  // SOL (200 FL + 150 yours)
        
        // Self-sandwich
        yourSales: '100M tokens за 90 SOL',
        
        // FL должен вернуть
        flMustReturn: 201,  // SOL
        
        // FL продает после миграции
        flSells: '40% позиции за 120 SOL',
        yourSOLFromSales: 90,  // SOL
        
        // 120 + 90 = 210 SOL > 201 нужно!
        result: 'FL ЗАКРОЕТСЯ УСПЕШНО!'
    }
};
```

### Вариант 2: Двухэтапная стратегия

```python
def two_stage_strategy():
    """Сначала прокачиваем, потом FL"""
    
    stage1 = {
        'your_action': 'Покупаете сами 100 SOL',
        'mc_movement': '$75k → $78k',
        'your_new_position': '450M tokens'
    }
    
    stage2 = {
        'flash_loan': 150,  # SOL (меньше!)
        'your_sandwich': 'Продаете 80M за 70 SOL',
        'migration': 'Triggered at $83k',
        
        'fl_sells': '30% за 90 SOL',
        'your_sol': 70,  # От sandwich
        'total': 160,  # SOL
        'fl_needs': 150.75,  # SOL
        
        'success': True
    }
```

## 📊 Оптимальная формула для Self-Sandwich

```python
optimal_formula = {
    'rule': 'FL размер должен быть таким, чтобы:',
    
    'calculation': '''
    FL_size * 0.4 (продажа после миграции) +
    Your_sandwich_sales_SOL >= 
    FL_size * 1.005 (долг + комиссия)
    ''',
    
    'example': {
        'fl_size': 200,
        'fl_return_need': 201,
        'fl_can_sell_for': 80,  # SOL (40% токенов)
        'your_must_sell_for': 121,  # SOL minimum
        'your_tokens_to_sell': '130M @ 0.00093'
    },
    
    'safety_margin': 'Добавьте 20% запас!'
}
```

## 🎯 Ключевые выводы

### ✅ Self-Sandwich РАБОТАЕТ если:

1. **FL не слишком большой** (150-250 SOL оптимально)
2. **Вы продаете достаточно** (получаете 40%+ от FL размера)
3. **Есть запасной план** (ваши SOL или партнер)
4. **Расчет с запасом** (20% буфер минимум)

### ❌ Self-Sandwich ПРОВАЛИТСЯ если:

1. **FL слишком большой** (300+ SOL)
2. **Мало продаете в sandwich** (<30% от FL)
3. **Нет ликвидности после миграции**
4. **Слишком жадная стратегия**

## 💡 Лучшая практика

```python
best_practice = {
    'fl_size': 'Не больше 70% от нужного для миграции',
    'your_position': 'Минимум 35% supply',
    'sandwich_sales': 'Продавать 25-30% вашей позиции',
    'safety': 'Иметь 50-100 SOL резерв',
    'execution': 'Все в одной атомарной транзакции'
}
```

**ВЫВОД**: Self-Sandwich может сломать FL если неправильно рассчитан! Всегда моделируйте с запасом.