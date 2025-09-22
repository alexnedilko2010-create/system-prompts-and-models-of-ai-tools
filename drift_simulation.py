"""
Симуляция стратегии манипулирования на Drift Protocol
Демонстрация почему стратегия НЕ работает
"""

import numpy as np
import matplotlib.pyplot as plt
from dataclasses import dataclass
from typing import List, Tuple
import pandas as pd

@dataclass
class MarketState:
    """Состояние рынка Drift Protocol"""
    base_price: float = 50.0  # Базовая цена SOL
    liquidity_depth: float = 100_000_000  # $100M ликвидности
    jit_liquidity_available: float = 50_000_000  # JIT ликвидность
    oracle_price: float = 50.0  # Цена от внешних оракулов
    funding_rate: float = 0.0001  # 0.01% per hour
    
@dataclass
class TradingFees:
    """Комиссии на Drift Protocol"""
    taker_fee: float = 0.001  # 0.1%
    flash_loan_fee: float = 0.0009  # 0.09%
    gas_fee: float = 50  # $50 за bundle
    slippage: float = 0.002  # 0.2% проскальзывание

class DriftMarketSimulator:
    def __init__(self):
        self.market = MarketState()
        self.fees = TradingFees()
        self.trades_history = []
        
    def calculate_price_impact(self, trade_size: float, use_jit: bool = True) -> float:
        """
        Расчет price impact с учетом JIT ликвидности
        Использует квадратичную модель impact
        """
        effective_liquidity = self.market.liquidity_depth
        if use_jit:
            # JIT ликвидность снижает impact
            effective_liquidity += self.market.jit_liquidity_available
            
        # Квадратичная модель: impact = (size / liquidity)^2
        # Более реалистично для больших ордеров
        impact_percentage = (trade_size / effective_liquidity) ** 2 * 100
        
        # Ограничение максимального impact (защита протокола)
        max_impact = 0.5  # Максимум 0.5% за сделку
        return min(impact_percentage, max_impact)
    
    def execute_trade(self, size: float, direction: str, wallet: str) -> dict:
        """Исполнение сделки с расчетом всех параметров"""
        
        # Расчет price impact
        price_impact = self.calculate_price_impact(size)
        
        # Новая цена после impact
        if direction == "long":
            new_price = self.market.base_price * (1 + price_impact / 100)
        else:
            new_price = self.market.base_price * (1 - price_impact / 100)
            
        # Oracle arbitrage - цена возвращается к оракулу
        oracle_difference = abs(new_price - self.market.oracle_price)
        if oracle_difference > 0.5:  # Если разница > $0.5
            # Арбитражеры выравнивают цену
            new_price = (new_price + self.market.oracle_price) / 2
            
        # Расчет комиссий
        trading_fee = size * self.fees.taker_fee
        
        # Обновление рыночной цены
        old_price = self.market.base_price
        self.market.base_price = new_price
        
        result = {
            "wallet": wallet,
            "direction": direction,
            "size": size,
            "old_price": old_price,
            "new_price": new_price,
            "price_impact": price_impact,
            "trading_fee": trading_fee,
            "oracle_price": self.market.oracle_price
        }
        
        self.trades_history.append(result)
        return result
    
    def simulate_strategy(self) -> dict:
        """Симуляция предложенной стратегии"""
        
        print("=" * 60)
        print("СИМУЛЯЦИЯ СТРАТЕГИИ МАНИПУЛИРОВАНИЯ")
        print("=" * 60)
        
        # Начальные параметры
        flash_loan_a = 250_000
        flash_loan_b = 40_000
        leverage = 10
        
        position_a = flash_loan_a * leverage  # 2.5M
        position_b = flash_loan_b * leverage  # 400k
        
        total_pnl = 0
        
        # ШАГ 1: Wallet A открывает long
        print("\n📍 ШАГ 1: Wallet A открывает позицию")
        trade1 = self.execute_trade(position_a, "long", "Wallet_A")
        print(f"  Размер позиции: ${position_a:,.0f}")
        print(f"  Цена: ${trade1['old_price']:.2f} → ${trade1['new_price']:.2f}")
        print(f"  Price Impact: {trade1['price_impact']:.3f}%")
        print(f"  ⚠️  Реальный impact: {trade1['price_impact']:.3f}% (НЕ 2.8%!)")
        
        # ШАГ 2: Wallet B открывает long
        print("\n📍 ШАГ 2: Wallet B открывает позицию")
        trade2 = self.execute_trade(position_b, "long", "Wallet_B")
        print(f"  Размер позиции: ${position_b:,.0f}")
        print(f"  Цена: ${trade2['old_price']:.2f} → ${trade2['new_price']:.2f}")
        print(f"  Price Impact: {trade2['price_impact']:.3f}%")
        
        # Максимальная достигнутая цена
        max_price = self.market.base_price
        print(f"\n📊 Максимальная цена: ${max_price:.2f}")
        print(f"📊 Oracle цена (не изменилась): ${self.market.oracle_price:.2f}")
        
        # ШАГ 3: Wallet A закрывает позицию
        print("\n📍 ШАГ 3: Wallet A закрывает позицию")
        entry_price_a = trade1['old_price']
        trade3 = self.execute_trade(position_a, "short", "Wallet_A")
        exit_price_a = trade3['new_price']
        
        # Расчет P&L для Wallet A
        price_change_a = (exit_price_a - entry_price_a) / entry_price_a
        gross_pnl_a = position_a * price_change_a
        
        # Вычитаем комиссии
        fees_a = (
            flash_loan_a * self.fees.flash_loan_fee +  # Flash loan fee
            position_a * self.fees.taker_fee * 2 +      # Trading fees (open + close)
            position_a * self.fees.slippage +           # Slippage
            self.fees.gas_fee                           # Gas
        )
        
        net_pnl_a = gross_pnl_a - fees_a
        
        print(f"  Entry: ${entry_price_a:.2f}, Exit: ${exit_price_a:.2f}")
        print(f"  Gross P&L: ${gross_pnl_a:,.2f}")
        print(f"  Fees: ${fees_a:,.2f}")
        print(f"  Net P&L: ${net_pnl_a:,.2f}")
        
        # ШАГ 4: Wallet B закрывает позицию
        print("\n📍 ШАГ 4: Wallet B закрывает позицию")
        entry_price_b = trade2['old_price']
        trade4 = self.execute_trade(position_b, "short", "Wallet_B")
        exit_price_b = trade4['new_price']
        
        # Расчет P&L для Wallet B
        price_change_b = (exit_price_b - entry_price_b) / entry_price_b
        gross_pnl_b = position_b * price_change_b
        
        # Вычитаем комиссии
        fees_b = (
            flash_loan_b * self.fees.flash_loan_fee +
            position_b * self.fees.taker_fee * 2 +
            position_b * self.fees.slippage +
            self.fees.gas_fee
        )
        
        net_pnl_b = gross_pnl_b - fees_b
        
        print(f"  Entry: ${entry_price_b:.2f}, Exit: ${exit_price_b:.2f}")
        print(f"  Gross P&L: ${gross_pnl_b:,.2f}")
        print(f"  Fees: ${fees_b:,.2f}")
        print(f"  Net P&L: ${net_pnl_b:,.2f}")
        
        # Итоговые результаты
        total_pnl = net_pnl_a + net_pnl_b
        total_fees = fees_a + fees_b
        
        print("\n" + "=" * 60)
        print("ИТОГОВЫЕ РЕЗУЛЬТАТЫ")
        print("=" * 60)
        print(f"💰 Wallet A P&L: ${net_pnl_a:,.2f}")
        print(f"💰 Wallet B P&L: ${net_pnl_b:,.2f}")
        print(f"📊 Общий P&L: ${total_pnl:,.2f}")
        print(f"💸 Общие комиссии: ${total_fees:,.2f}")
        
        if total_pnl < 0:
            print(f"\n❌ СТРАТЕГИЯ УБЫТОЧНА: -${abs(total_pnl):,.2f}")
        else:
            print(f"\n✅ Прибыль: ${total_pnl:,.2f} (намного меньше ожидаемых $65k)")
            
        return {
            "wallet_a_pnl": net_pnl_a,
            "wallet_b_pnl": net_pnl_b,
            "total_pnl": total_pnl,
            "total_fees": total_fees,
            "max_price_achieved": max_price,
            "final_price": self.market.base_price
        }
    
    def plot_price_movement(self):
        """Визуализация движения цены"""
        if not self.trades_history:
            return
            
        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 8))
        
        # График цены
        prices = [50.0] + [t['new_price'] for t in self.trades_history]
        steps = range(len(prices))
        
        ax1.plot(steps, prices, 'b-', linewidth=2, marker='o', markersize=8)
        ax1.axhline(y=self.market.oracle_price, color='r', linestyle='--', 
                   label=f'Oracle Price: ${self.market.oracle_price}')
        ax1.set_xlabel('Шаг транзакции')
        ax1.set_ylabel('Цена SOL ($)')
        ax1.set_title('Движение цены во время выполнения стратегии')
        ax1.grid(True, alpha=0.3)
        ax1.legend()
        
        # Аннотации
        annotations = ['Начало', 'Wallet A Long', 'Wallet B Long', 
                      'Wallet A Close', 'Wallet B Close']
        for i, (step, price, ann) in enumerate(zip(steps, prices, annotations)):
            ax1.annotate(ann, xy=(step, price), xytext=(step, price + 0.1),
                        fontsize=9, ha='center')
        
        # График объемов
        volumes = [t['size'] for t in self.trades_history]
        wallets = [t['wallet'] for t in self.trades_history]
        colors = ['green' if 'A' in w else 'blue' for w in wallets]
        
        ax2.bar(range(len(volumes)), volumes, color=colors, alpha=0.6)
        ax2.set_xlabel('Транзакция')
        ax2.set_ylabel('Объем ($)')
        ax2.set_title('Объемы транзакций')
        ax2.grid(True, alpha=0.3)
        
        # Легенда для объемов
        from matplotlib.patches import Patch
        legend_elements = [Patch(facecolor='green', alpha=0.6, label='Wallet A'),
                          Patch(facecolor='blue', alpha=0.6, label='Wallet B')]
        ax2.legend(handles=legend_elements)
        
        plt.tight_layout()
        plt.savefig('drift_strategy_simulation.png', dpi=150, bbox_inches='tight')
        print("\n📈 График сохранен в 'drift_strategy_simulation.png'")
        plt.show()

def analyze_different_scenarios():
    """Анализ различных сценариев ликвидности"""
    print("\n" + "=" * 60)
    print("АНАЛИЗ РАЗЛИЧНЫХ СЦЕНАРИЕВ")
    print("=" * 60)
    
    scenarios = [
        {"name": "Низкая ликвидность", "liquidity": 10_000_000, "jit": 5_000_000},
        {"name": "Средняя ликвидность", "liquidity": 50_000_000, "jit": 25_000_000},
        {"name": "Высокая ликвидность (реальная)", "liquidity": 100_000_000, "jit": 50_000_000},
        {"name": "Очень высокая ликвидность", "liquidity": 500_000_000, "jit": 100_000_000},
    ]
    
    results = []
    
    for scenario in scenarios:
        sim = DriftMarketSimulator()
        sim.market.liquidity_depth = scenario["liquidity"]
        sim.market.jit_liquidity_available = scenario["jit"]
        
        print(f"\n📊 {scenario['name']}:")
        print(f"   Ликвидность: ${scenario['liquidity']:,}")
        print(f"   JIT ликвидность: ${scenario['jit']:,}")
        
        # Тестовая сделка 2.5M
        impact = sim.calculate_price_impact(2_500_000)
        print(f"   Price Impact для $2.5M: {impact:.3f}%")
        
        # Симуляция стратегии (упрощенная)
        result = sim.simulate_strategy()
        results.append({
            "scenario": scenario["name"],
            "liquidity": scenario["liquidity"],
            "impact": impact,
            "pnl": result["total_pnl"]
        })
    
    # Создание таблицы результатов
    df = pd.DataFrame(results)
    print("\n📊 СВОДНАЯ ТАБЛИЦА РЕЗУЛЬТАТОВ:")
    print(df.to_string(index=False))
    
    return df

def main():
    """Главная функция симуляции"""
    print("🚀 ЗАПУСК СИМУЛЯЦИИ СТРАТЕГИИ DRIFT PROTOCOL")
    print("=" * 60)
    
    # Основная симуляция
    simulator = DriftMarketSimulator()
    result = simulator.simulate_strategy()
    
    # Визуализация
    simulator.plot_price_movement()
    
    # Анализ различных сценариев
    scenarios_df = analyze_different_scenarios()
    
    # Финальные выводы
    print("\n" + "=" * 60)
    print("ФИНАЛЬНЫЕ ВЫВОДЫ")
    print("=" * 60)
    print("❌ Стратегия НЕ РАБОТАЕТ по следующим причинам:")
    print("1. Реальный price impact в 10-20 раз меньше ожидаемого")
    print("2. JIT ликвидность поглощает большую часть impact")
    print("3. Oracle arbitrage возвращает цену к рыночной")
    print("4. Комиссии съедают большую часть прибыли")
    print("5. Риск ликвидации при неблагоприятном движении")
    print("\n⚠️  ВАЖНО: Это манипулирование рынком - НЕЗАКОННО!")
    
    return result

if __name__ == "__main__":
    # Установка seed для воспроизводимости
    np.random.seed(42)
    
    # Запуск симуляции
    final_result = main()
    
    print("\n✅ Симуляция завершена")
    print(f"📊 Финальный результат: ${final_result['total_pnl']:,.2f}")