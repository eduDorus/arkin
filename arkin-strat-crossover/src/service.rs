use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use tracing::{debug, info, instrument, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use uuid::Uuid;

#[derive(TypedBuilder)]
#[allow(unused)]
pub struct CrossoverStrategy {
    #[builder(default = String::from("strat-crossover"))]
    identifier: String,
    time: Arc<dyn SystemTime>,
    publisher: Arc<dyn Publisher>,
    strategy: Arc<Strategy>,
    #[builder(default = DashMap::new())]
    current_weights: DashMap<Arc<Instrument>, Decimal>,
    allocation_limit_per_instrument: Decimal,
    fast_ma: FeatureId,
    slow_ma: FeatureId,
}

impl CrossoverStrategy {
    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn insight_tick(&self, tick: &InsightsUpdate) {
        debug!(target: "strat-crossover", "received insight tick");

        // Calculate the crossover signals for each instrument in the tick
        let mut new_weights = HashMap::new();
        for i in &tick.instruments {
            let fast_ma = tick.insights.iter().find(|x| {
                if let Some(inst) = x.instrument.as_ref() {
                    inst == i && x.feature_id == self.fast_ma
                } else {
                    false
                }
            });

            let slow_ma = tick.insights.iter().find(|x| {
                if let Some(inst) = x.instrument.as_ref() {
                    inst == i && x.feature_id == self.slow_ma
                } else {
                    false
                }
            });

            let new_weight = match (fast_ma, slow_ma) {
                (Some(f), Some(s)) => match f.value > s.value {
                    true => Decimal::ONE,
                    false => Decimal::NEGATIVE_ONE,
                },
                _ => Decimal::ZERO,
            };

            new_weights.insert(i, new_weight);
        }

        // Check the difference between the current and previous signals
        let mut allocations = HashMap::new();
        for (instrument, new_weight) in new_weights {
            if let Some(current_weight) = self.current_weights.get(instrument) {
                if *current_weight != new_weight {
                    info!(target: "strat-crossover", "weight change detected for {} from {} to {}", instrument, *current_weight, new_weight);
                    // Calculate the allocation change
                    let allocation_change = new_weight - *current_weight;
                    allocations.insert(instrument.clone(), allocation_change);
                }
            } else {
                info!(target: "strat-crossover", "new weight for {} is {}", instrument, new_weight);
                if !new_weight.is_zero() {
                    allocations.insert(instrument.clone(), new_weight);
                }
            }
            self.current_weights.insert(instrument.clone(), new_weight);
        }

        // Create execution orders based on the new weights
        let mut execution_orders = Vec::new();
        for (instrument, weight) in allocations {
            if weight.is_zero() {
                continue; // Skip zero weights
            }
            let order = ExecutionOrder::builder()
                .id(Uuid::new_v4())
                .strategy(Some(self.strategy.to_owned()))
                .instrument(instrument.to_owned())
                .exec_strategy_type(ExecutionStrategyType::Taker)
                .side(if weight.is_sign_positive() {
                    MarketSide::Buy
                } else {
                    MarketSide::Sell
                })
                .set_price(dec!(0))
                .set_quantity(weight.abs())
                .status(ExecutionOrderStatus::New)
                .created_at(self.time.now().await)
                .updated_at(self.time.now().await)
                .build();
            execution_orders.push(order);
        }

        for order in execution_orders.iter() {
            self.publisher
                .publish(Event::NewTakerExecutionOrder(order.to_owned().into()))
                .await;
            info!(target: "strat-crossover", "send {} execution order for {} to execution strategy {} for a quantity of {}", order.side, order.instrument, order.exec_strategy_type, order.quantity);
        }
    }
}

#[async_trait]
impl Runnable for CrossoverStrategy {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::InsightsUpdate(vo) => self.insight_tick(vo).await,
            e => warn!(target: "strat-crossover", "received unused event {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkin_core::test_utils::{MockPublisher, MockTime};

    #[tokio::test]
    #[test_log::test]
    async fn test_crossover_strategy() {
        let publisher = MockPublisher::new();
        let time = MockTime::new();
        let strategy = Strategy::builder()
            .id(Uuid::from_str("1fce35ce-1583-4334-a410-bc0f71c7469b").expect("Invalid UUID"))
            .name("crossover_strategy".into())
            .description(Some("This strategy is only for testing".into()))
            .build();
        let strategy_name = Arc::new(strategy);
        let service = CrossoverStrategy::builder()
            .identifier("crossover_strategy".into())
            .publisher(publisher.to_owned())
            .time(time.to_owned())
            .strategy(strategy_name)
            .allocation_limit_per_instrument(dec!(10000))
            .fast_ma(FeatureId::new("vwap_price_ema_10".into()))
            .slow_ma(FeatureId::new("vwap_price_ema_60".into()))
            .build();

        // First update with insights
        let insight_btc_fast = Insight::builder()
            .instrument(Some(test_inst_binance_btc_usdt_perp()))
            .feature_id(service.fast_ma.clone())
            .value(50000.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insight_btc_slow = Insight::builder()
            .instrument(Some(test_inst_binance_btc_usdt_perp()))
            .feature_id(service.slow_ma.clone())
            .value(49000.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insight_eth_fast = Insight::builder()
            .instrument(Some(test_inst_binance_eth_usdt_perp()))
            .feature_id(service.fast_ma.clone())
            .value(3000.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insight_eth_slow = Insight::builder()
            .instrument(Some(test_inst_binance_eth_usdt_perp()))
            .feature_id(service.slow_ma.clone())
            .value(3100.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insights_update = InsightsUpdate::builder()
            .instruments(vec![test_inst_binance_btc_usdt_perp(), test_inst_binance_eth_usdt_perp()])
            .insights(vec![
                insight_btc_fast.into(),
                insight_btc_slow.into(),
                insight_eth_fast.into(),
                insight_eth_slow.into(),
            ])
            .event_time(time.now().await)
            .build();

        service.handle_event(Event::InsightsUpdate(insights_update.into())).await;

        // Second update with insights, should trigger no weight change
        let insight_btc_fast = Insight::builder()
            .instrument(Some(test_inst_binance_btc_usdt_perp()))
            .feature_id(service.fast_ma.clone())
            .value(51000.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insight_btc_slow = Insight::builder()
            .instrument(Some(test_inst_binance_btc_usdt_perp()))
            .feature_id(service.slow_ma.clone())
            .value(50000.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insight_eth_fast = Insight::builder()
            .instrument(Some(test_inst_binance_eth_usdt_perp()))
            .feature_id(service.fast_ma.clone())
            .value(3100.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insight_eth_slow = Insight::builder()
            .instrument(Some(test_inst_binance_eth_usdt_perp()))
            .feature_id(service.slow_ma.clone())
            .value(3200.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insights_update = InsightsUpdate::builder()
            .instruments(vec![test_inst_binance_btc_usdt_perp(), test_inst_binance_eth_usdt_perp()])
            .insights(vec![
                insight_btc_fast.into(),
                insight_btc_slow.into(),
                insight_eth_fast.into(),
                insight_eth_slow.into(),
            ])
            .event_time(time.now().await)
            .build();

        service.handle_event(Event::InsightsUpdate(insights_update.into())).await;

        // Second update with insights, should trigger complete filp
        let insight_btc_fast = Insight::builder()
            .instrument(Some(test_inst_binance_btc_usdt_perp()))
            .feature_id(service.fast_ma.clone())
            .value(49000.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insight_btc_slow = Insight::builder()
            .instrument(Some(test_inst_binance_btc_usdt_perp()))
            .feature_id(service.slow_ma.clone())
            .value(50000.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insight_eth_fast = Insight::builder()
            .instrument(Some(test_inst_binance_eth_usdt_perp()))
            .feature_id(service.fast_ma.clone())
            .value(3100.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insight_eth_slow = Insight::builder()
            .instrument(Some(test_inst_binance_eth_usdt_perp()))
            .feature_id(service.slow_ma.clone())
            .value(3000.0)
            .insight_type(InsightType::Continuous)
            .event_time(time.now().await)
            .build();

        let insights_update = InsightsUpdate::builder()
            .instruments(vec![test_inst_binance_btc_usdt_perp(), test_inst_binance_eth_usdt_perp()])
            .insights(vec![
                insight_btc_fast.into(),
                insight_btc_slow.into(),
                insight_eth_fast.into(),
                insight_eth_slow.into(),
            ])
            .event_time(time.now().await)
            .build();

        service.handle_event(Event::InsightsUpdate(insights_update.into())).await;
    }
}
