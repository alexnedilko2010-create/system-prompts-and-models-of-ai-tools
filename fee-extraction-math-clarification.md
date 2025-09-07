# 📊 Fee Extraction - Уточнение математики прибыльности

## 🔍 Два пути к прибыльности

### Путь 1: Большая доля в обычном AMM (без Concentrated)

```python
traditional_amm_profitability = {
    'minimum_share_for_profit': 20,  # 20% минимум
    'optimal_share': 30-50,  # 30-50% оптимально
    
    'math_example_20_percent': {
        'your_share': 20,  # 20% пула
        'your_volume': 1_000_000,
        'fees_paid': 2_500,  # 0.25%
        'fees_received': 500,  # 20% от $2,500
        'loss_on_wash': -2_000,
        
        'needed_external': {
            'minimum': 4_000_000,  # 4x от вашего объема!
            'external_fees': 10_000,
            'your_capture': 2_000,  # 20% от $10k
            'break_even': True
        }
    },
    
    'math_example_50_percent': {
        'your_share': 50,  # 50% пула
        'your_volume': 1_000_000,
        'fees_paid': 2_500,
        'fees_received': 1_250,  # 50% обратно
        'loss_on_wash': -1_250,
        
        'needed_external': {
            'minimum': 1_000_000,  # 1:1 с вашим объемом
            'external_fees': 2_500,
            'your_capture': 1_250,
            'break_even': True
        }
    }
}
```

### Путь 2: Concentrated Liquidity (меньше капитала, больше эффективность)

```python
concentrated_liquidity_profitability = {
    'can_work_with_small_capital': True,
    'effective_share_in_range': 90-99,  # В узком диапазоне
    
    'comparison': {
        'traditional_50_percent': {
            'capital_needed': 500_000,  # Для 50% в $1M пуле
            'external_volume_needed': '100% от вашего',
            'complexity': 'Простая'
        },
        
        'concentrated_95_percent': {
            'capital_needed': 50_000,  # 10x меньше!
            'external_volume_needed': '5% от вашего',
            'complexity': 'Требует навыка'
        }
    }
}
```

## 📈 Сравнительная таблица стратегий

```python
strategy_comparison_table = {
    'headers': ['Стратегия', 'Мин. доля', 'Капитал для $1M пула', 'Внешний объем', 'Сложность'],
    
    'rows': [
        ['Traditional 20%', '20%', '$200k', '400% вашего', '⭐⭐'],
        ['Traditional 30%', '30%', '$300k', '230% вашего', '⭐⭐'],
        ['Traditional 50%', '50%', '$500k', '100% вашего', '⭐⭐'],
        ['Concentrated 90%', '90% в range', '$50k', '11% вашего', '⭐⭐⭐⭐'],
        ['Concentrated 95%', '95% в range', '$50k', '5% вашего', '⭐⭐⭐⭐'],
        ['Concentrated 99%', '99% в range', '$100k', '1% вашего', '⭐⭐⭐⭐⭐']
    ]
}
```

## 💰 Формула расчета безубыточности

```python
def calculate_breakeven_external_volume(your_share_percent, pool_fee_percent=0.25):
    """
    Рассчитывает необходимый внешний объем для безубыточности
    """
    # Формула: External Volume = Your Volume × (1 - Your Share) / Your Share
    
    multiplier = (100 - your_share_percent) / your_share_percent
    
    return {
        'your_share': f'{your_share_percent}%',
        'external_multiplier': f'{multiplier:.1f}x',
        'example': {
            'if_you_trade': '$1,000,000',
            'need_external': f'${int(1_000_000 * multiplier):,}',
            'feasibility': 'Easy' if multiplier < 0.5 else 'Medium' if multiplier < 2 else 'Hard'
        }
    }

# Примеры
print(calculate_breakeven_external_volume(20))   # 4x external needed
print(calculate_breakeven_external_volume(33))   # 2x external needed  
print(calculate_breakeven_external_volume(50))   # 1x external needed
print(calculate_breakeven_external_volume(80))   # 0.25x external needed
print(calculate_breakeven_external_volume(95))   # 0.05x external needed
```

## 🎯 Когда какую стратегию выбрать?

### Traditional AMM (20-50% доля):

```python
traditional_amm_use_cases = {
    'pros': [
        'Проще в управлении',
        'Не нужно следить за диапазоном',
        'Стабильный доход',
        'Меньше рисков'
    ],
    
    'cons': [
        'Нужен большой капитал',
        'Нужен huge внешний объем',
        'Меньше ROI'
    ],
    
    'best_for': [
        'Большие капиталы ($100k+)',
        'Пулы с гарантированным объемом',
        'Долгосрочные позиции',
        'Risk-averse трейдеры'
    ],
    
    'example_pools': [
        'SOL/USDC (основные пары)',
        'ETH/USDC (высокий объем)',
        'Стейблкоины (постоянный флоу)'
    ]
}
```

### Concentrated Liquidity (90-99% в диапазоне):

```python
concentrated_use_cases = {
    'pros': [
        'Меньше нужен капитал (10x)',
        'Минимальный внешний объем',
        'Огромный ROI потенциал',
        'Быстрая окупаемость'
    ],
    
    'cons': [
        'Сложнее в управлении',
        'Нужен rebalancing',
        'Риск выхода из диапазона',
        'Требует мониторинга'
    ],
    
    'best_for': [
        'Маленькие капиталы ($10k+)',
        'Активные трейдеры',
        'High risk/reward',
        'Автоматизированные стратегии'
    ]
}
```

## 📊 Реальные примеры обеих стратегий

### Пример 1: Traditional 30% доля

```python
traditional_example = {
    'pool': 'SOL/USDC на Raydium',
    'pool_size': 3_000_000,
    'your_capital': 1_000_000,  # Нужен $1M!
    'your_share': 33.3,
    
    'daily_operation': {
        'your_volume': 5_000_000,
        'your_fees_paid': 12_500,
        'your_fees_back': 4_167,  # 33.3%
        'loss': -8_333,
        
        'organic_volume': 12_000_000,  # 2.4x
        'organic_fees': 30_000,
        'your_capture': 10_000,  # 33.3%
        
        'fl_cost': -500,
        'net_profit': 1_167  # $1,167 прибыли в день
    },
    
    'roi': '0.12% в день на $1M капитала'
}
```

### Пример 2: Concentrated 95% в диапазоне

```python
concentrated_example = {
    'pool': 'SOL/USDC на Orca',
    'your_capital': 50_000,  # Всего $50k!
    'range': '±0.5%',
    'effective_share': 95,
    
    'daily_operation': {
        'your_volume': 5_000_000,
        'your_fees_paid': 12_500,
        'your_fees_back': 11_875,  # 95%!
        'loss': -625,
        
        'organic_volume': 300_000,  # Всего 6%!
        'organic_fees': 750,
        'your_capture': 712,  # 95%
        
        'fl_cost': -500,
        'net_profit': -413  # Почти безубыток
    },
    
    'with_more_organic': {
        'organic_volume': 500_000,  # 10%
        'additional_profit': 475,
        'total_profit': 62,  # Уже профит!
    },
    
    'roi': 'Потенциал 1-5% в день на $50k'
}
```

## ✅ Финальные выводы

```python
final_conclusions = {
    'both_strategies_work': True,
    
    'traditional_amm': {
        'minimum_share': '20% (лучше 30%+)',
        'capital_required': 'High ($200k+ для больших пулов)',
        'external_volume': 'Нужно 100-400% от вашего',
        'best_use': 'Стабильный доход, большой капитал'
    },
    
    'concentrated_liquidity': {
        'minimum_share': '90% в диапазоне',
        'capital_required': 'Low ($10-50k достаточно)',
        'external_volume': 'Всего 5-10% от вашего',
        'best_use': 'Максимум ROI, активное управление'
    },
    
    'recommendation': '''
    Начинающим с капиталом < $100k:
    → Concentrated Liquidity
    
    Большим игрокам с $500k+:
    → Traditional 30-50% может быть проще
    
    Для максимального профита:
    → Concentrated + JIT всегда выигрывает
    '''
}
```

## 🎯 Правильная формула выбора

```
Если капитал < $100k → Concentrated обязательно
Если капитал > $500k → Traditional 30%+ возможен
Если хотите макс ROI → Concentrated всегда лучше
Если хотите простоту → Traditional приемлем при большой доле
```