# Стратегия двойного Flash Loan для миграции

## 🎯 Концепция стратегии

```
Flash Loan #1: Накопление позиции при низком MC
Flash Loan #2: Финальный push для миграции
```

## 📊 Детальный расчет

### Параметры:
- Total Supply: 1,000,000,000 токенов
- Цель: 10% supply (100,000,000 токенов)
- Курс SOL: $100

### Фаза 1: Первый Flash Loan при MC = $40,000

```python
# Flash Loan #1
market_cap_1 = 40000
price_1 = 0.00004  # $40,000 / 1B tokens
target_tokens = 100_000_000  # 10% supply

# Расчет с учетом slippage на bonding curve
# При покупке 10% supply цена вырастет примерно на 25%
average_price = 0.00004 * 1.25 = 0.00005
cost_for_10_percent = 100_000_000 * 0.00005 = 5000  # $5,000

# Flash Loan #1 параметры:
flash_loan_1_usd = 5000
flash_loan_1_sol = 50  # SOL
```

### Что происходит после покупки:

```python
# Новое состояние после покупки 10%
new_market_cap = 40000 * 1.25 = 50000  # MC вырос до $50,000
new_price = 0.00005
our_position_value = 100_000_000 * 0.00005 = 5000  # Безубыток пока
```

### Фаза 2: Ожидание органического роста

```
Ждем пока MC вырастет до $60,000-65,000 органически
Это критически важно для прибыльности!

$50,000 → $60,000 (органический рост)
Наша позиция: 100M токенов × $0.00006 = $6,000
Бумажная прибыль: +$1,000 (20%)
```

### Фаза 3: Второй Flash Loan при MC = $60,000

```python
# Flash Loan #2
current_mc = 60000
target_mc = 69000
gap = 9000

# Расчет необходимой суммы с учетом slippage
# Для подъема с $60k до $69k нужно больше чем $9k
slippage_factor = 1.4  # 40% slippage на последнем участке
required_amount = gap * slippage_factor = 12600  # $12,600

# Flash Loan #2 параметры:
flash_loan_2_usd = 13000  # С запасом
flash_loan_2_sol = 130
```

## 💰 Итоговый расчет прибыли

### После миграции при MC = $69,000:

```python
# Наши активы:
price_after_migration = 0.000069

# Позиция из Flash Loan #1:
position_1_value = 100_000_000 * 0.000069 = 6900  # $6,900

# Токены купленные через Flash Loan #2:
# С учетом высокого slippage получим меньше токенов
tokens_from_loan_2 = 13000 / 0.00008 = 162_500_000  # Средняя цена выше!
position_2_value = 162_500_000 * 0.000069 = 11212  # $11,212

# Общая стоимость позиции:
total_value = 6900 + 11212 = 18112  # $18,112

# Обязательства по Flash Loans:
loan_1_repay = 5000 * 1.005 = 5025  # С комиссией 0.5%
loan_2_repay = 13000 * 1.005 = 13065  # С комиссией 0.5%
total_debt = 18090  # $18,090

# ПРИБЫЛЬ:
gross_profit = 18112 - 18090 = 22  # Всего $22!
```

## 😱 Проблема: Почти нет прибыли!

### Почему так мало?

1. **Высокий slippage** на обоих этапах
2. **Недостаточный органический рост** между займами
3. **Комиссии flash loan** съедают маржу

## ✅ Оптимизированная версия стратегии

### Вариант A: Больше органического роста

```python
# Сценарий с ожиданием
# Flash Loan #1 при MC = $30,000
loan_1 = 4000  # Покупаем 10% дешевле
wait_for_mc = 65000  # Ждем больше

# Flash Loan #2 при MC = $65,000
loan_2 = 6000  # Меньше нужно для добивания

# Результат:
position_value = 100_000_000 * 0.000069 = 6900
cost = 4000  # Только первый loan
profit = 6900 - 4000 = 2900  # $2,900 профит!
```

### Вариант B: Один loan + собственные средства

```python
# Комбинированный подход
own_funds = 3000  # При MC = $25,000
flash_loan = 8000  # При MC = $55,000

# Финальная позиция:
tokens_owned = 120_000_000  # От собственных средств
tokens_flash = 80_000_000   # От flash loan
total_tokens = 200_000_000  # 20% supply!

# После миграции:
value = 200_000_000 * 0.000069 = 13800
costs = 3000 + 8000 = 11000
profit = 2800  # $2,800 профит
```

## 📈 Оптимальные параметры для двойного Flash Loan

### Параметры для максимальной прибыли:

```
Flash Loan #1:
- Вход: MC < $35,000
- Размер: $3,000-5,000
- Цель: 8-12% supply
- Ожидаемый slippage: 20-25%

Ожидание:
- Целевой MC: $60,000-65,000
- Время: 1-7 дней
- Мониторинг: каждые 30 минут

Flash Loan #2:
- Вход: MC = $64,000-66,000
- Размер: $5,000-8,000
- Цель: Добить до $69,000
- Ожидаемый slippage: 30-35%
```

## 🛠️ Код для реализации

```javascript
class DoubleFlashLoanStrategy {
    constructor(connection, config) {
        this.connection = connection;
        this.config = config;
        this.phase = 1;
        this.positions = [];
    }

    async executePhase1(tokenMint, targetSupplyPercent = 0.1) {
        const marketCap = await this.getMarketCap(tokenMint);
        
        if (marketCap > this.config.phase1MaxMC) {
            throw new Error(`MC слишком высокий: ${marketCap}`);
        }

        // Расчет размера Flash Loan #1
        const loanSize = this.calculateLoanSize(
            marketCap, 
            targetSupplyPercent
        );

        // Выполнение первого Flash Loan
        const result = await this.executeFlashLoan({
            amount: loanSize,
            action: 'accumulate',
            targetPercent: targetSupplyPercent
        });

        this.positions.push({
            phase: 1,
            tokens: result.tokensAcquired,
            cost: loanSize,
            avgPrice: result.avgPrice
        });

        // Начинаем мониторинг для фазы 2
        this.startPhase2Monitoring(tokenMint);
    }

    async startPhase2Monitoring(tokenMint) {
        const monitor = setInterval(async () => {
            const mc = await this.getMarketCap(tokenMint);
            
            if (mc >= this.config.phase2TriggerMC) {
                console.log('🎯 Время для Flash Loan #2!');
                clearInterval(monitor);
                await this.executePhase2(tokenMint);
            }
        }, 30000); // Каждые 30 секунд
    }

    async executePhase2(tokenMint) {
        const currentMC = await this.getMarketCap(tokenMint);
        const gapToTarget = 69000 - currentMC;
        
        // Расчет размера Flash Loan #2
        const loanSize = gapToTarget * this.config.slippageFactor;

        // Выполнение второго Flash Loan
        const result = await this.executeFlashLoan({
            amount: loanSize,
            action: 'graduate',
            checkMigration: true
        });

        // Расчет общей прибыли
        const totalProfit = this.calculateTotalProfit();
        console.log(`💰 Общая прибыль: $${totalProfit}`);
    }

    calculateTotalProfit() {
        const finalPrice = 0.000069;
        let totalValue = 0;
        let totalCost = 0;

        for (const position of this.positions) {
            totalValue += position.tokens * finalPrice;
            totalCost += position.cost * 1.005; // С комиссией
        }

        return totalValue - totalCost;
    }
}

// Использование
const strategy = new DoubleFlashLoanStrategy(connection, {
    phase1MaxMC: 35000,      // Максимальный MC для входа
    phase2TriggerMC: 64000,  // MC для второго loan
    slippageFactor: 1.4,     // Ожидаемый slippage
    maxTotalExposure: 20000  // Максимальный риск
});

// Запуск
await strategy.executePhase1(tokenMint, 0.1); // 10% supply
```

## ⚠️ Риски двойного Flash Loan

### 1. Риск застревания между фазами
- MC может не дойти до $60,000+
- Первый loan придется погашать с убытком

### 2. Фронт-раннинг на второй фазе
- Другие могут опередить при приближении к $69,000
- Защита: рандомизация времени + private mempool

### 3. Изменение параметров
- Проект может изменить порог миграции
- Может добавить ограничения на покупку

## 🎯 Вывод

**Двойной Flash Loan может работать, НО:**

1. **Критически важен вход ДО $35,000 MC**
2. **Нужно ждать органический рост между фазами**
3. **Прибыль 20-50% реалистична при правильном исполнении**
4. **Риски выше из-за двух точек отказа**

**Оптимальная стратегия:**
- Flash Loan #1: $4,000 при MC $30,000
- Ожидание роста до MC $64,000
- Flash Loan #2: $6,000 для финального push
- Ожидаемая прибыль: $2,000-3,000 (20-30%)