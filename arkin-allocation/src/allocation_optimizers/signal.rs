#![allow(dead_code)]
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use arkin_portfolio::prelude::*;
use uuid::Uuid;

use crate::{AllocationOptim, AllocationOptimError, AllocationService};

#[derive(Debug, TypedBuilder)]
pub struct SignalAllocationOptim {
    pubsub: Arc<PubSub>,
    persistence: Arc<PersistenceService>,
    portfolio: Arc<dyn Accounting>,
    #[builder(default = DashMap::new())]
    optimal_allocation: DashMap<Arc<Instrument>, Arc<Signal>>,
    leverage: Decimal,
    min_trade_value: Decimal,
    allocation_feature_id: FeatureId,
    reference_currency: Arc<Asset>,
}

pub struct OptimalPosition {
    pub instrument: Arc<Instrument>,
    pub price: Price,
    pub quantity: Quantity,
}

pub struct DiffPosition {
    pub instrument: Arc<Instrument>,
    pub price: Price,
    pub quantity: Quantity,
    pub diff: Quantity,
}

#[async_trait]
impl AllocationOptim for SignalAllocationOptim {
    async fn optimize(&self, signal: Arc<Signal>) -> Result<Vec<Arc<ExecutionOrder>>, AllocationOptimError> {
        // Save down new allocation

        self.optimal_allocation.insert(signal.instrument.clone(), signal.clone());

        // Check if we have any signals
        if self.optimal_allocation.is_empty() {
            debug!("No allocations found for optimization");
            return Ok(Vec::new());
        }

        // Calculate money allocated to each signal
        let capital = self.portfolio.available_balance(&self.reference_currency).await;
        if capital.is_zero() {
            warn!("No capital available for allocation");
            return Ok(Vec::new());
        }
        let leveraged_capital = capital * self.leverage;
        info!(
            "Available capital for allocation: {} with {} times leverage becomes {}",
            capital, self.leverage, leveraged_capital
        );

        // Get current positions
        let current_positions = self.portfolio.get_positions().await;
        for (_, position) in current_positions.iter() {
            info!("Current position {}", position);
        }

        // Calculate current weights
        let mut current_weights = HashMap::new();
        for (instrument, position) in current_positions.iter() {
            let weight = position.market_value() / leveraged_capital;
            current_weights.insert(instrument.clone(), weight);
        }
        for (instrument, weight) in current_weights.iter() {
            info!("Current weight for {} is {}", instrument, weight);
        }

        // Get our current optimal allocation
        let optimal_weights = self
            .optimal_allocation
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect::<HashMap<_, _>>();
        for (instrument, signal) in optimal_weights.iter() {
            info!("Optimal weight for {} is {}", instrument, signal.weight);
        }

        // Calculate the difference between current and optimal allocation weights
        let mut allocation_change = HashMap::new();
        for (optimal_instrument, optimal_weight) in optimal_weights.iter() {
            if let Some(current_weight) = current_weights.get(optimal_instrument) {
                let diff = optimal_weight.weight - current_weight;
                allocation_change.insert(optimal_instrument.clone(), diff);
            } else {
                allocation_change.insert(optimal_instrument.clone(), optimal_weight.weight);
            }
        }
        for (instrument, weight) in allocation_change.iter() {
            info!("Change weight for {} with {}", instrument, weight);
        }

        // Create execution orders
        let mut execution_orders: Vec<Arc<ExecutionOrder>> = Vec::with_capacity(allocation_change.len());
        for (instrument, diff) in allocation_change.into_iter() {
            // Get current price
            let tick = if let Some(tick) = self.persistence.tick_store.get_last_tick(&instrument).await {
                tick
            } else {
                warn!("No price found for {}", instrument);
                continue;
            };

            // Determine order side
            let order_side = if diff.is_sign_positive() {
                MarketSide::Buy
            } else {
                MarketSide::Sell
            };

            // We need to round the price to the tick_size and then scale it to the price_precision
            // Calculate the scaling factor: 1 / tick_size
            let price = match MarketSide::Buy {
                MarketSide::Buy => tick.ask_price(),
                MarketSide::Sell => tick.bid_price(),
            };
            let scaling_factor = Decimal::ONE / instrument.tick_size;
            let scaled_price = price * scaling_factor;
            let rounded_scaled_price = scaled_price.round();
            let rounded_price = rounded_scaled_price * instrument.tick_size;
            let final_price = rounded_price.round_dp(instrument.price_precision);

            // Calculate the quantity to trade
            let trade_amount = leveraged_capital * diff.abs();
            let quantity = trade_amount / final_price;
            let scaling_factor = Decimal::ONE / instrument.lot_size;
            let scaled_quantity = quantity * scaling_factor;
            let rounded_scaled_quantity = scaled_quantity.round();
            let round_quantity = rounded_scaled_quantity * instrument.lot_size;
            let final_quantity = round_quantity.round_dp(instrument.quantity_precision);

            // Skip if quantity is below minimum trade size
            let value = final_price * final_quantity.abs();
            if value < self.min_trade_value {
                info!(
                    "Skipping trade for {} as value of {} is below minimum trade size of {}",
                    instrument, value, self.min_trade_value
                );
                continue;
            }

            let order = ExecutionOrder::builder()
                .id(Uuid::new_v4())
                .strategy(Some(test_strategy()))
                .instrument(instrument.clone())
                .order_type(ExecutionOrderType::Maker)
                .side(order_side)
                .quantity(final_quantity)
                .price(final_price)
                .created_at(tick.event_time)
                .updated_at(tick.event_time)
                .build();
            execution_orders.push(order.into());
        }
        for order in execution_orders.iter() {
            info!("Execution order: {}", order);
        }

        for order in execution_orders.iter() {
            self.pubsub.publish(order.clone()).await;
        }

        Ok(execution_orders)
    }
}

#[async_trait]
impl RunnableService for SignalAllocationOptim {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting LimitedAllocation...");

        let mut rx = self.pubsub.subscribe();

        loop {
            select! {
                Ok(event) = rx.recv() => {
                    match event {
                        Event::Signal(signal) => {
                            debug!("LimitedAllocationOptim received signal: {}", signal.event_time);
                            self.optimize(signal).await?;
                        }
                        _ => {}
                    }
                }
                _ = shutdown.cancelled() => {
                    info!("LimitedAllocationOptim shutdown...");
                    break;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AllocationService for SignalAllocationOptim {}
