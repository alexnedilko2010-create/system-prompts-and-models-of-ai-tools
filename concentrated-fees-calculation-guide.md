# 📊 Расчет fees за один цикл - Детальный гайд

## 🧮 Базовая формула расчета

```python
def calculate_cycle_fees():
    """
    Fees = Volume × Fee_Rate × Your_Share × Time_Factor
    """
    
    # Входные данные
    pool_volume_per_second = 500_000  # $500k/sec
    pool_fee_rate = 0.0025            # 0.25%
    your_position = 1_000_000         # $1M
    concentration_multiplier = 100     # 100x эффект
    cycle_time = 0.4                  # 400ms
    
    # Расчеты
    volume_during_cycle = pool_volume_per_second * cycle_time  # $200k
    
    # Ваша эффективная позиция
    your_effective_position = your_position * concentration_multiplier  # $100M
    
    # Доля в пуле (в range)
    total_liquidity_in_range = 150_000_000  # $150M
    your_share = your_effective_position / total_liquidity_in_range  # 66.7%
    
    # Fees
    total_fees = volume_during_cycle * pool_fee_rate  # $500
    your_fees = total_fees * your_share  # $333
    
    return your_fees
```

## 📈 Метрики активности пула

### 1. Получение данных о volume:

```typescript
async function getPoolMetrics(poolAddress: string) {
    // Исторические данные (последний час)
    const hourlyVolume = await getHourlyVolume(poolAddress);
    const volumePerSecond = hourlyVolume / 3600;
    
    // Текущая активность (последние 10 секунд)
    const recentTrades = await getRecentTrades(poolAddress, 10);
    const recentVolumePerSecond = recentTrades.totalVolume / 10;
    
    // Взвешенное среднее (80% recent, 20% historical)
    const weightedVolume = (recentVolumePerSecond * 0.8) + 
                          (volumePerSecond * 0.2);
    
    return {
        avgVolumePerSecond: weightedVolume,
        peakVolumePerSecond: Math.max(...recentTrades.volumes),
        minVolumePerSecond: Math.min(...recentTrades.volumes)
    };
}
```

### 2. Анализ концентрации ликвидности:

```python
def analyze_liquidity_concentration(pool, price_range):
    """
    Определяем сколько ликвидности в нашем range
    """
    
    # Получаем все позиции в пуле
    positions = pool.get_all_positions()
    
    # Считаем ликвидность в нашем range
    liquidity_in_range = 0
    for position in positions:
        if position.tick_lower <= price_range.lower and \
           position.tick_upper >= price_range.upper:
            liquidity_in_range += position.liquidity
    
    # Наша доля
    our_liquidity = calculate_liquidity(1_000_000, price_range)
    our_share = our_liquidity / (liquidity_in_range + our_liquidity)
    
    return {
        'total_in_range': liquidity_in_range,
        'our_liquidity': our_liquidity,
        'our_share_percent': our_share * 100,
        'concentration_factor': pool.total_liquidity / liquidity_in_range
    }
```

## 🎯 Практические примеры расчета

### Пример 1: Стейблкоин пул (USDC/USDT)

```python
stable_pool_calculation = {
    'pool': 'USDC/USDT 0.05%',
    'metrics': {
        'avg_volume_per_sec': '$300,000',
        'volatility': '0.02%',
        'total_liquidity': '$50M'
    },
    
    'your_position': {
        'amount': '$500,000',
        'range': '±0.02%',  # 0.9998 - 1.0002
        'concentration': '250x'
    },
    
    'calculation': {
        'effective_position': '$125M',
        'liquidity_in_range': '$30M',
        'your_share': '80%',  # Доминируете!
        
        'per_cycle_calculation': {
            'cycle_time': '400ms',
            'volume_in_cycle': '$120,000',
            'total_fees': '$60',  # 0.05% fee
            'your_fees': '$48',   # 80% share
            'fl_cost': '$50',
            'net_profit': '-$2'   # Почти breakeven
        },
        
        'optimization': 'Need tighter range or higher volume'
    }
}
```

### Пример 2: Волатильный пул (SOL/USDC)

```python
volatile_pool_calculation = {
    'pool': 'SOL/USDC 0.25%',
    'metrics': {
        'avg_volume_per_sec': '$800,000',
        'volatility': '2%',
        'total_liquidity': '$100M'
    },
    
    'your_position': {
        'amount': '$1,000,000',
        'range': '±0.5%',  # Wider due to volatility
        'concentration': '40x'
    },
    
    'calculation': {
        'effective_position': '$40M',
        'liquidity_in_range': '$60M',
        'your_share': '40%',
        
        'per_cycle_calculation': {
            'cycle_time': '400ms',
            'volume_in_cycle': '$320,000',
            'total_fees': '$800',
            'your_fees': '$320',
            'fl_cost': '$100',
            'net_profit': '+$220'  # Profitable!
        }
    }
}
```

## 📊 Детальная формула с учетом всех факторов

```python
def calculate_exact_cycle_fees(
    # Pool parameters
    pool_address: str,
    fee_tier: float,
    
    # Position parameters  
    position_size: float,
    tick_lower: int,
    tick_upper: int,
    
    # Market conditions
    current_price: float,
    volatility: float,
    
    # FL parameters
    fl_amount: float,
    fl_fee_rate: float = 0.0001,  # 0.01%
    
    # Execution parameters
    cycle_time_ms: float = 400
):
    """
    Точный расчет прибыли за цикл
    """
    
    # 1. Получаем текущий volume
    volume_data = get_pool_volume_data(pool_address)
    volume_per_ms = volume_data['volume_per_second'] / 1000
    
    # 2. Рассчитываем концентрацию
    price_range_percent = (tick_upper - tick_lower) / current_price
    concentration = 1 / price_range_percent  # Simplified
    
    # 3. Определяем нашу долю в range
    total_liquidity_in_range = get_liquidity_in_range(
        pool_address, tick_lower, tick_upper
    )
    our_effective_liquidity = (position_size + fl_amount) * concentration
    our_share = our_effective_liquidity / (total_liquidity_in_range + our_effective_liquidity)
    
    # 4. Считаем fees
    volume_during_cycle = volume_per_ms * cycle_time_ms
    total_fees_generated = volume_during_cycle * fee_tier
    our_fees = total_fees_generated * our_share
    
    # 5. Считаем costs
    fl_cost = fl_amount * fl_fee_rate
    gas_cost = estimate_gas_cost()
    slippage = estimate_slippage(fl_amount, total_liquidity_in_range)
    
    # 6. Результат
    gross_profit = our_fees
    total_costs = fl_cost + gas_cost + slippage
    net_profit = gross_profit - total_costs
    
    return {
        'volume_during_cycle': volume_during_cycle,
        'our_share_percent': our_share * 100,
        'gross_fees': gross_profit,
        'costs': {
            'fl_fee': fl_cost,
            'gas': gas_cost,
            'slippage': slippage
        },
        'net_profit': net_profit,
        'profitable': net_profit > 0
    }
```

## 🔍 Мониторинг в реальном времени

```typescript
class RealTimeFeesMonitor {
    constructor(private pool: Pool) {
        this.startMonitoring();
    }
    
    async startMonitoring() {
        // Subscribe to swap events
        this.pool.on('swap', async (event) => {
            const feesPaid = event.amount * this.pool.feeRate;
            const ourShare = await this.calculateOurShare();
            const ourFees = feesPaid * ourShare;
            
            console.log(`Swap: $${event.amount}, Our fees: $${ourFees}`);
            
            // Update running average
            this.updateMetrics(ourFees);
        });
    }
    
    async calculateOptimalCycleTime() {
        const metrics = await this.getMetrics();
        
        // Оптимальное время = когда накопится достаточно fees
        const targetFees = 150; // $150 minimum
        const avgFeesPerMs = metrics.avgFeesPerSecond / 1000;
        const optimalTime = targetFees / avgFeesPerMs;
        
        return Math.min(optimalTime, 1000); // Max 1 second
    }
}
```

## 📈 Таблица быстрого расчета

### Quick Reference для разных пулов:

```
QUICK FEES ESTIMATION TABLE:
═══════════════════════════════════════════════════════════════

Pool Type    Volume/sec   Your Share   Fee Rate   Fees/400ms   Profitable?
─────────────────────────────────────────────────────────────────────────
Stable       $200k        80%          0.05%      $32          NO ❌
(tight)                                           (FL=$50)

Stable       $500k        70%          0.05%      $70          YES ✅
(active)                                          (FL=$50)

Major        $300k        50%          0.25%      $150         YES ✅
(normal)                                          (FL=$100)

Major        $800k        40%          0.25%      $320         YES ✅
(active)                                          (FL=$100)

Volatile     $200k        30%          0.50%      $120         MAYBE
(wide)                                            (FL=$100)

═══════════════════════════════════════════════════════════════
```

## 🎯 Оптимизация для максимальных fees

```python
optimization_strategies = {
    'range_optimization': {
        'stable_pairs': '±0.01-0.02%',
        'correlated': '±0.1-0.3%',
        'major_pairs': '±0.3-0.5%',
        'volatile': '±0.5-1%'
    },
    
    'timing_optimization': {
        'monitor_volume_spikes': True,
        'execute_during_peak': True,
        'avoid_low_volume_hours': True
    },
    
    'position_sizing': {
        'target_share': '50-70%',
        'max_share': '80%',  # Avoid 90%+ (slippage)
        'fl_multiplier': '1-2x base position'
    },
    
    'pool_selection': {
        'min_daily_volume': '$50M',
        'preferred_fee_tier': '0.25%',
        'avoid': 'New pools, low liquidity'
    }
}
```

## ✅ Простой чек-лист для расчета

```
BEFORE EACH CYCLE - CALCULATE:
═══════════════════════════════════════════

□ Current volume/second
□ Your share in range (%)
□ Expected volume in 400ms
□ Total fees generated
□ Your portion of fees
□ FL + gas + slippage costs
□ Net profit

IF profit > costs * 1.5 → EXECUTE ✓
IF profit < costs → SKIP ❌
═══════════════════════════════════════════
```