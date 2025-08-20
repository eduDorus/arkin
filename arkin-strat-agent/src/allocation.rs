use arkin_core::prelude::*;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use std::sync::Arc;
use time::UtcDateTime;
use tracing::warn;
use uuid::Uuid;

#[derive(Clone)]
pub struct Allocation {
    pub price: Price,
    pub quantity: Quantity,
    pub current_weight: Weight,
}

impl Allocation {
    pub fn new(price: Price, quantity: Quantity, current_weight: Weight) -> Self {
        Self {
            price,
            quantity,
            current_weight,
        }
    }
}

pub struct AllocationEngine {
    pub allocation: DashMap<Arc<Instrument>, Allocation>,
    pub current_price: DashMap<Arc<Instrument>, Arc<Tick>>,
    pub capital_per_inst: Decimal,
    pub strategy: Arc<Strategy>,
}

impl AllocationEngine {
    pub fn new(capital_per_inst: Decimal, strategy: Arc<Strategy>) -> Self {
        Self {
            allocation: DashMap::new(),
            current_price: DashMap::new(),
            capital_per_inst,
            strategy,
        }
    }

    pub fn update_price(&self, tick: Arc<Tick>) {
        self.current_price.insert(tick.instrument.clone(), tick);
    }

    pub fn update(&self, time: UtcDateTime, instrument: &Arc<Instrument>, weight: Weight) -> Option<ExecutionOrder> {
        let current_tick = self.current_price.get(instrument)?.clone();
        let current_price = current_tick.mid_price();
        if current_price.is_zero() {
            warn!(target: "strat::agent", "No current price for instrument {}", instrument);
            return None;
        }

        let mut entry = self
            .allocation
            .entry(instrument.clone())
            .or_insert_with(|| Allocation::new(Price::ZERO, Quantity::ZERO, Weight::ZERO));

        if entry.current_weight == weight {
            return None;
        }

        let target_exposure = weight * self.capital_per_inst;
        let desired_qty = target_exposure / current_price;

        let direction = if desired_qty.is_zero() {
            Decimal::ZERO
        } else if desired_qty.is_sign_positive() {
            Decimal::ONE
        } else {
            Decimal::NEGATIVE_ONE
        };

        let abs_scaled = (desired_qty.abs() / instrument.lot_size).floor();
        let target_qty = (abs_scaled * instrument.lot_size * direction).round_dp(instrument.quantity_precision);

        let delta_qty = target_qty - entry.quantity;
        if delta_qty.is_zero() {
            entry.current_weight = weight;
            return None;
        }

        let target_capital = (target_qty * current_price).abs();
        if target_capital > self.capital_per_inst {
            warn!(
                "Target quantity exceeds available capital: {} > {}",
                target_capital, self.capital_per_inst
            );
            return None; // Safety check, though floor prevents this
        }

        let side = if delta_qty > Decimal::ZERO {
            MarketSide::Buy
        } else {
            MarketSide::Sell
        };
        let order = ExecutionOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(self.strategy.clone()))
            .instrument(instrument.clone())
            .exec_strategy_type(ExecutionStrategyType::Taker)
            .side(side)
            .set_price(current_price)
            .set_quantity(delta_qty.abs())
            .status(ExecutionOrderStatus::New)
            .created(time)
            .updated(time)
            .build();

        // Assume immediate fill; update position
        let old_value = entry.quantity * entry.price;
        let trade_value = delta_qty * current_price;
        let new_qty = entry.quantity + delta_qty;
        let new_price = if new_qty.is_zero() {
            Price::ZERO
        } else {
            (old_value + trade_value) / new_qty
        };
        entry.quantity = new_qty;
        entry.price = new_price;
        entry.current_weight = weight;

        Some(order)
    }
}

#[cfg(test)]
mod tests {
    use tracing::info;

    use super::*;

    #[tokio::test]
    #[test_log::test]
    async fn test_allocation_engine_update() {
        let strategy = test_strategy_1();
        let engine = AllocationEngine::new(dec!(10_000), strategy);

        let instrument = test_inst_binance_btc_usdt_perp();

        engine.update_price(
            Tick::builder()
                .event_time(UtcDateTime::now())
                .instrument(instrument.clone())
                .tick_id(0)
                .bid_price(dec!(2990))
                .bid_quantity(dec!(1))
                .ask_price(dec!(3000))
                .ask_quantity(dec!(1))
                .build()
                .into(),
        );
        let order = engine.update(UtcDateTime::now(), &instrument, dec!(0));
        assert!(order.is_none());

        let order = engine.update(UtcDateTime::now(), &instrument, dec!(1));
        assert!(order.is_some());
        let order = order.unwrap();
        info!("order: {}", order);
        assert_eq!(order.instrument, instrument);
        assert_eq!(order.exec_strategy_type, ExecutionStrategyType::Taker);
        assert_eq!(order.side, MarketSide::Buy);
        assert_eq!(order.price, dec!(2995));
        assert_eq!(order.quantity, dec!(3.338));

        let order = engine.update(UtcDateTime::now(), &instrument, dec!(-1));
        assert!(order.is_some());
        let order = order.unwrap();
        info!("order: {}", order);
        assert_eq!(order.instrument, instrument);
        assert_eq!(order.exec_strategy_type, ExecutionStrategyType::Taker);
        assert_eq!(order.side, MarketSide::Sell);
        assert_eq!(order.price, dec!(2995));
        assert_eq!(order.quantity, dec!(6.676));

        engine.update_price(
            Tick::builder()
                .event_time(UtcDateTime::now())
                .instrument(instrument.clone())
                .tick_id(0)
                .bid_price(dec!(4000))
                .bid_quantity(dec!(1))
                .ask_price(dec!(4100))
                .ask_quantity(dec!(1))
                .build()
                .into(),
        );

        let order = engine.update(UtcDateTime::now(), &instrument, dec!(0));
        assert!(order.is_some());
        let order = order.unwrap();
        info!("order: {}", order);
        assert_eq!(order.instrument, instrument);
        assert_eq!(order.exec_strategy_type, ExecutionStrategyType::Taker);
        assert_eq!(order.side, MarketSide::Buy);
        assert_eq!(order.price, dec!(4050));
        assert_eq!(order.quantity, dec!(3.338));
    }
}
