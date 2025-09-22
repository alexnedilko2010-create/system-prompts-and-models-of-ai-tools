"""
Корректный анализ стратегии для Drift Protocol Perpetual Futures
С учетом механики mark price vs index price и позиций с плечом
"""

from dataclasses import dataclass
from typing import Dict, Tuple
import math

@dataclass
class DriftPerpetualMechanics:
    """Реальная механика Drift Protocol Perpetual Futures"""
    
    # vAMM параметры (виртуальная ликвидность)
    vamm_base_asset_reserve: float = 1_000_000  # Виртуальные SOL в резерве
    vamm_quote_asset_reserve: float = 50_000_000  # Виртуальные USDC в резерве
    k_constant: float = 50_000_000_000_000  # x*y=k инвариант
    
    # Параметры протокола
    max_leverage: int = 10
    maintenance_margin: float = 0.05  # 5% maintenance margin
    initial_margin: float = 0.10  # 10% initial margin
    
    # Комиссии
    taker_fee: float = 0.0005  # 0.05%
    maker_fee: float = -0.0002  # -0.02% rebate
    
    # Funding rate параметры
    funding_period: int = 3600  # 1 час в секундах
    max_funding_rate: float = 0.01  # 1% в час максимум
    
    # Oracle параметры
    oracle_price: float = 50.0  # Index price от Pyth/Switchboard
    oracle_confidence: float = 0.05  # $0.05 confidence interval

class PerpetualStrategySimulator:
    def __init__(self):
        self.drift = DriftPerpetualMechanics()
        self.mark_price = self.drift.oracle_price  # Начальная mark price = index price
        
    def calculate_vamm_price_after_trade(self, quote_amount: float, is_long: bool) -> Dict:
        """
        Расчет mark price после сделки через vAMM механику
        Mark price определяется vAMM, а не оракулом!
        """
        
        # Текущие резервы vAMM
        base_reserve = self.drift.vamm_base_asset_reserve
        quote_reserve = self.drift.vamm_quote_asset_reserve
        
        # Текущая mark price (до сделки)
        current_mark_price = quote_reserve / base_reserve
        
        if is_long:
            # При покупке (long): добавляем quote, получаем base
            # Новое количество quote в резерве
            new_quote_reserve = quote_reserve + quote_amount
            
            # Из x*y=k находим новое количество base
            new_base_reserve = self.drift.k_constant / new_quote_reserve
            
            # Количество base assets получено трейдером
            base_received = base_reserve - new_base_reserve
            
            # Обновляем резервы
            self.drift.vamm_base_asset_reserve = new_base_reserve
            self.drift.vamm_quote_asset_reserve = new_quote_reserve
            
        else:
            # При продаже (short): добавляем base, получаем quote
            # Сначала находим сколько base нужно для quote_amount
            # Используем приближение через текущую цену
            base_to_sell = quote_amount / current_mark_price
            
            # Новое количество base в резерве
            new_base_reserve = base_reserve + base_to_sell
            
            # Из x*y=k находим новое количество quote
            new_quote_reserve = self.drift.k_constant / new_base_reserve
            
            # Количество quote получено трейдером
            quote_received = quote_reserve - new_quote_reserve
            
            # Обновляем резервы
            self.drift.vamm_base_asset_reserve = new_base_reserve
            self.drift.vamm_quote_asset_reserve = new_quote_reserve
            
            base_received = -base_to_sell  # Отрицательное для short
            
        # Новая mark price после сделки
        new_mark_price = self.drift.vamm_quote_asset_reserve / self.drift.vamm_base_asset_reserve
        
        # Price impact
        price_impact = (new_mark_price - current_mark_price) / current_mark_price * 100
        
        # Расчет funding rate (mark vs index)
        mark_index_divergence = (new_mark_price - self.drift.oracle_price) / self.drift.oracle_price
        funding_rate = min(max(mark_index_divergence * 0.1, -self.drift.max_funding_rate), 
                          self.drift.max_funding_rate)
        
        return {
            "old_mark_price": current_mark_price,
            "new_mark_price": new_mark_price,
            "price_impact": price_impact,
            "base_amount": abs(base_received),
            "average_fill_price": quote_amount / abs(base_received) if base_received != 0 else current_mark_price,
            "funding_rate": funding_rate,
            "mark_index_divergence": mark_index_divergence * 100
        }
    
    def simulate_jito_bundle_strategy(self):
        """Симуляция стратегии через Jito Bundle с реальной механикой perp"""
        
        print("=" * 70)
        print("АНАЛИЗ СТРАТЕГИИ НА DRIFT PERPETUAL FUTURES")
        print("=" * 70)
        
        # Параметры стратегии
        collateral_a = 250_000  # Flash loan как collateral
        collateral_b = 40_000
        leverage = 10
        
        # Notional позиции
        notional_a = collateral_a * leverage  # $2.5M notional
        notional_b = collateral_b * leverage   # $400k notional
        
        print(f"\n📊 НАЧАЛЬНЫЕ ПАРАМЕТРЫ:")
        print(f"  Index Price (Oracle): ${self.drift.oracle_price:.2f}")
        print(f"  Mark Price (vAMM): ${self.mark_price:.2f}")
        print(f"  vAMM Base Reserve: {self.drift.vamm_base_asset_reserve:,.0f} SOL")
        print(f"  vAMM Quote Reserve: ${self.drift.vamm_quote_asset_reserve:,.0f}")
        
        # ШАГ 1: Wallet A открывает long с leverage
        print(f"\n📍 ШАГ 1: Wallet A открывает LONG")
        print(f"  Collateral: ${collateral_a:,}")
        print(f"  Leverage: {leverage}x")
        print(f"  Notional: ${notional_a:,}")
        
        trade_a_open = self.calculate_vamm_price_after_trade(notional_a, is_long=True)
        entry_price_a = trade_a_open["average_fill_price"]
        position_size_a = trade_a_open["base_amount"]
        
        print(f"  Position size: {position_size_a:,.2f} SOL")
        print(f"  Average fill: ${entry_price_a:.3f}")
        print(f"  Mark price: ${trade_a_open['old_mark_price']:.2f} → ${trade_a_open['new_mark_price']:.2f}")
        print(f"  Price impact: {trade_a_open['price_impact']:.3f}%")
        print(f"  Mark-Index divergence: {trade_a_open['mark_index_divergence']:.3f}%")
        print(f"  Funding rate: {trade_a_open['funding_rate']*100:.4f}%/hour")
        
        # ШАГ 2: Wallet B открывает long
        print(f"\n📍 ШАГ 2: Wallet B открывает LONG")
        print(f"  Collateral: ${collateral_b:,}")
        print(f"  Notional: ${notional_b:,}")
        
        trade_b_open = self.calculate_vamm_price_after_trade(notional_b, is_long=True)
        entry_price_b = trade_b_open["average_fill_price"]
        position_size_b = trade_b_open["base_amount"]
        
        print(f"  Position size: {position_size_b:,.2f} SOL")
        print(f"  Average fill: ${entry_price_b:.3f}")
        print(f"  Mark price: ${trade_b_open['old_mark_price']:.2f} → ${trade_b_open['new_mark_price']:.2f}")
        print(f"  Price impact: {trade_b_open['price_impact']:.3f}%")
        
        max_mark_price = trade_b_open["new_mark_price"]
        
        print(f"\n📊 МАКСИМАЛЬНАЯ MARK PRICE: ${max_mark_price:.2f}")
        print(f"📊 Index Price (не изменилась): ${self.drift.oracle_price:.2f}")
        print(f"📊 Divergence: {((max_mark_price - self.drift.oracle_price) / self.drift.oracle_price * 100):.2f}%")
        
        # ШАГ 3: Wallet A закрывает позицию
        print(f"\n📍 ШАГ 3: Wallet A закрывает LONG (открывает SHORT)")
        
        # Закрытие = обратная сделка
        trade_a_close = self.calculate_vamm_price_after_trade(notional_a, is_long=False)
        exit_price_a = trade_a_close["average_fill_price"]
        
        print(f"  Average fill: ${exit_price_a:.3f}")
        print(f"  Mark price: ${trade_a_close['old_mark_price']:.2f} → ${trade_a_close['new_mark_price']:.2f}")
        
        # ШАГ 4: Wallet B закрывает позицию
        print(f"\n📍 ШАГ 4: Wallet B закрывает LONG")
        
        trade_b_close = self.calculate_vamm_price_after_trade(notional_b, is_long=False)
        exit_price_b = trade_b_close["average_fill_price"]
        
        print(f"  Average fill: ${exit_price_b:.3f}")
        print(f"  Mark price: ${trade_b_close['old_mark_price']:.2f} → ${trade_b_close['new_mark_price']:.2f}")
        
        # РАСЧЕТ P&L
        print(f"\n" + "=" * 70)
        print("РАСЧЕТ ПРИБЫЛИ/УБЫТКОВ")
        print("=" * 70)
        
        # P&L для Wallet A
        pnl_per_sol_a = exit_price_a - entry_price_a
        gross_pnl_a = pnl_per_sol_a * position_size_a
        
        # Комиссии
        fees_a = (
            notional_a * self.drift.taker_fee * 2 +  # Open + Close
            collateral_a * 0.0003 +  # Solend flash loan 0.03%
            100  # Jito tip
        )
        
        # Funding payments (если держали позицию хотя бы час)
        funding_payment_a = position_size_a * max_mark_price * trade_a_open["funding_rate"]
        
        net_pnl_a = gross_pnl_a - fees_a - funding_payment_a
        
        # P&L для Wallet B
        pnl_per_sol_b = exit_price_b - entry_price_b
        gross_pnl_b = pnl_per_sol_b * position_size_b
        
        fees_b = (
            notional_b * self.drift.taker_fee * 2 +
            collateral_b * 0.0003 +
            20  # Jito tip
        )
        
        funding_payment_b = position_size_b * max_mark_price * trade_b_open["funding_rate"]
        
        net_pnl_b = gross_pnl_b - fees_b - funding_payment_b
        
        # Проверка маржи и ликвидации
        print(f"\n⚠️  ПРОВЕРКА РИСКА ЛИКВИДАЦИИ:")
        
        # Unrealized PnL в худшей точке для Wallet A
        worst_pnl_a = (trade_b_close["new_mark_price"] - entry_price_a) * position_size_a
        margin_ratio_a = (collateral_a + worst_pnl_a) / notional_a
        
        print(f"  Wallet A margin ratio: {margin_ratio_a*100:.2f}%")
        if margin_ratio_a < self.drift.maintenance_margin:
            print(f"  ❌ ЛИКВИДАЦИЯ! Margin ratio < {self.drift.maintenance_margin*100}%")
            net_pnl_a = -collateral_a  # Потеря всего collateral
        
        print(f"\n💰 WALLET A:")
        print(f"  Entry: ${entry_price_a:.3f}, Exit: ${exit_price_a:.3f}")
        print(f"  Position: {position_size_a:.2f} SOL")
        print(f"  Gross P&L: ${gross_pnl_a:,.2f}")
        print(f"  Fees: ${fees_a:,.2f}")
        print(f"  Funding: ${funding_payment_a:,.2f}")
        print(f"  Net P&L: ${net_pnl_a:,.2f}")
        
        print(f"\n💰 WALLET B:")
        print(f"  Entry: ${entry_price_b:.3f}, Exit: ${exit_price_b:.3f}")
        print(f"  Position: {position_size_b:.2f} SOL")
        print(f"  Gross P&L: ${gross_pnl_b:,.2f}")
        print(f"  Fees: ${fees_b:,.2f}")
        print(f"  Funding: ${funding_payment_b:,.2f}")
        print(f"  Net P&L: ${net_pnl_b:,.2f}")
        
        total_pnl = net_pnl_a + net_pnl_b
        
        print(f"\n" + "=" * 70)
        print(f"📊 ИТОГОВЫЙ P&L: ${total_pnl:,.2f}")
        print("=" * 70)
        
        return {
            "total_pnl": total_pnl,
            "max_mark_price": max_mark_price,
            "max_divergence": (max_mark_price - self.drift.oracle_price) / self.drift.oracle_price * 100
        }

def explain_key_differences():
    """Объяснение ключевых различий perpetual vs spot"""
    
    print("\n" + "=" * 70)
    print("КЛЮЧЕВЫЕ ОСОБЕННОСТИ PERPETUAL FUTURES НА DRIFT")
    print("=" * 70)
    
    print("\n1️⃣ MARK PRICE vs INDEX PRICE:")
    print("   • Mark Price - определяется vAMM (может отличаться от oracle)")
    print("   • Index Price - от внешних оракулов (Pyth/Switchboard)")
    print("   • P&L рассчитывается по Mark Price!")
    print("   • Funding платежи = (Mark - Index) * Position")
    
    print("\n2️⃣ vAMM МЕХАНИКА:")
    print("   • Виртуальная ликвидность (не реальная)")
    print("   • Использует формулу x*y=k")
    print("   • Price impact зависит от размера резервов")
    print("   • Mark price МОЖЕТ сильно отклоняться от Index")
    
    print("\n3️⃣ FUNDING RATE:")
    print("   • Платится каждый час")
    print("   • Если Mark > Index: longs платят shorts")
    print("   • Может достигать 1% в час при сильном divergence")
    print("   • Съедает прибыль при удержании позиции")
    
    print("\n4️⃣ ЛИКВИДАЦИЯ:")
    print("   • При margin ratio < 5% = ликвидация")
    print("   • Рассчитывается по Mark Price (не Index!)")
    print("   • При резком движении mark price = массовые ликвидации")
    
    print("\n5️⃣ ПОЧЕМУ СТРАТЕГИЯ МОЖЕТ РАБОТАТЬ (теоретически):")
    print("   • vAMM позволяет двигать mark price")
    print("   • Нет прямого ограничения от oracle")
    print("   • P&L считается по mark price")
    print("   • В атомарной транзакции можно зафиксировать прибыль")

def analyze_real_scenario():
    """Анализ реалистичного сценария"""
    
    print("\n" + "=" * 70)
    print("РЕАЛИСТИЧНЫЙ СЦЕНАРИЙ")
    print("=" * 70)
    
    # Более реалистичные параметры vAMM для SOL-PERP
    realistic_sim = PerpetualStrategySimulator()
    
    # Увеличиваем резервы для более реалистичной симуляции
    realistic_sim.drift.vamm_base_asset_reserve = 2_000_000  # 2M SOL
    realistic_sim.drift.vamm_quote_asset_reserve = 100_000_000  # $100M
    realistic_sim.drift.k_constant = 200_000_000_000_000
    
    print("\n📊 РЕАЛИСТИЧНЫЕ ПАРАМЕТРЫ vAMM:")
    print(f"  Base Reserve: 2,000,000 SOL")
    print(f"  Quote Reserve: $100,000,000")
    print(f"  Implied Mark Price: $50.00")
    
    result = realistic_sim.simulate_jito_bundle_strategy()
    
    print("\n⚠️  КРИТИЧЕСКИЕ ПРОБЛЕМЫ:")
    
    print("\n1. НЕДОСТАТОЧНЫЙ PRICE IMPACT")
    print(f"   • Достигнутый divergence: {result['max_divergence']:.2f}%")
    print(f"   • Нужно для прибыли: 2-3%+")
    
    print("\n2. FUNDING RATE СЪЕДАЕТ ПРИБЫЛЬ")
    print(f"   • При divergence {result['max_divergence']:.2f}% = высокий funding")
    print(f"   • Longs платят shorts каждый час")
    
    print("\n3. РИСК КАСКАДНЫХ ЛИКВИДАЦИЙ")
    print(f"   • Другие longs могут быть ликвидированы")
    print(f"   • Это создаст давление на продажу")
    print(f"   • Mark price может резко упасть")
    
    print("\n4. КОНКУРЕНЦИЯ АРБИТРАЖЕРОВ")
    print(f"   • При divergence > 1% включаются арбитражные боты")
    print(f"   • Они откроют shorts и вернут mark к index")
    
    return result

def main():
    """Главная функция"""
    
    print("\n🎯 КОРРЕКТНЫЙ АНАЛИЗ СТРАТЕГИИ НА DRIFT PERPETUALS\n")
    
    # Базовая симуляция с малыми резервами (лучший сценарий)
    print("СЦЕНАРИЙ 1: МАЛАЯ ЛИКВИДНОСТЬ vAMM (лучший случай)")
    print("-" * 70)
    optimistic_sim = PerpetualStrategySimulator()
    optimistic_result = optimistic_sim.simulate_jito_bundle_strategy()
    
    # Объяснение механики
    explain_key_differences()
    
    # Реалистичный сценарий
    print("\n\nСЦЕНАРИЙ 2: РЕАЛИСТИЧНАЯ ЛИКВИДНОСТЬ")
    print("-" * 70)
    realistic_result = analyze_real_scenario()
    
    # Финальные выводы
    print("\n" + "=" * 70)
    print("ФИНАЛЬНЫЙ ВЕРДИКТ")
    print("=" * 70)
    
    print("\n✅ ВЫ ПРАВЫ В СЛЕДУЮЩЕМ:")
    print("• Используются perpetual futures с плечом, а не спот")
    print("• Oracle не ограничивает mark price напрямую")
    print("• Mark price определяется vAMM и может отклоняться от index")
    
    print("\n⚠️  НО ЕСТЬ ПРОБЛЕМЫ:")
    
    print("\n1. РЕАЛЬНЫЕ РЕЗЕРВЫ vAMM БОЛЬШЕ")
    print("   • Drift имеет достаточную виртуальную ликвидность")
    print("   • Price impact будет меньше ожидаемого")
    
    print("\n2. FUNDING RATE МЕХАНИЗМ")
    print("   • При большом divergence = огромный funding")
    print("   • Съест значительную часть прибыли")
    
    print("\n3. АРБИТРАЖЕРЫ")
    print("   • При divergence > 0.5-1% включатся арбитражные боты")
    print("   • Они откроют counter-позиции и вернут баланс")
    
    print("\n4. РИСК ЛИКВИДАЦИИ")
    print("   • При leverage 10x нужен всего 10% движение против вас")
    print("   • В volatile условиях = высокий риск")
    
    print(f"\n📊 РЕЗУЛЬТАТЫ СИМУЛЯЦИИ:")
    print(f"Оптимистичный сценарий: ${optimistic_result['total_pnl']:,.2f}")
    print(f"Реалистичный сценарий: ${realistic_result['total_pnl']:,.2f}")
    
    if optimistic_result['total_pnl'] > 0:
        print(f"\n⚠️  Даже если стратегия приносит прибыль:")
        print("• Это все еще манипулирование рынком")
        print("• Юридические риски остаются")
        print("• Drift может заблокировать аккаунты")
        print("• Репутационные потери")

if __name__ == "__main__":
    main()