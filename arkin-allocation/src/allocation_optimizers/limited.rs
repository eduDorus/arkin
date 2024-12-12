#![allow(dead_code)]
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use time::OffsetDateTime;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use arkin_portfolio::prelude::*;
use uuid::Uuid;

use crate::{AllocationOptim, AllocationOptimError};

#[derive(Debug, TypedBuilder)]
pub struct LimitedAllocationOptim {
    pubsub: Arc<PubSub>,
    persistence: Arc<PersistenceService>,
    portfolio: Arc<dyn Accounting>,
    #[builder(default = DashMap::new())]
    optimal_allocation: DashMap<Arc<Instrument>, Arc<AllocationWeight>>,
    #[builder(default = dec!(0.8))]
    max_allocation: Decimal,
    #[builder(default = dec!(1.0))]
    max_allocation_per_signal: Decimal,
    #[builder(default = dec!(100))]
    min_trade_value: Decimal,
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
impl AllocationOptim for LimitedAllocationOptim {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), AllocationOptimError> {
        info!("Starting LimitedAllocation...");
        let mut signal_tick = self.pubsub.subscribe::<AllocationTick>();
        loop {
            select! {
                Ok(tick) = signal_tick.recv() => {
                    info!("LimitedAllocationOptim received signal tick: {}", tick.event_time);
                    self.new_weights(tick.clone()).await?;
                    self.optimize(tick.event_time).await?;
                }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn new_weights(&self, tick: Arc<AllocationTick>) -> Result<(), AllocationOptimError> {
        for allocation in tick.allocations.iter() {
            self.optimal_allocation
                .insert(allocation.instrument.clone(), allocation.clone());
        }
        Ok(())
    }

    async fn optimize(&self, event_time: OffsetDateTime) -> Result<Vec<Arc<ExecutionOrder>>, AllocationOptimError> {
        // Check if we have any signals
        if self.optimal_allocation.is_empty() {
            warn!("No allocations found for optimization");
            return Ok(Vec::new());
        }

        // Calculate money allocated to each signal
        let capital = self.portfolio.capital(&self.reference_currency).await;

        // Get current positions
        let current_positions = self.portfolio.list_open_positions().await;
        for (_, position) in current_positions.iter() {
            info!("Current position {}", position);
        }

        // Calculate current weights
        let mut current_weights = HashMap::new();
        for (instrument, position) in current_positions.iter() {
            let weight = position.market_value() / capital;
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
        for (instrument, weight) in optimal_weights.iter() {
            info!("Optimal weight for {} is {}", instrument, weight.weight);
        }

        // Calculate the difference between current and optimal allocation weights
        let mut allocation_change = HashMap::new();
        for (current_instrument, weight) in current_weights.iter() {
            if let Some(optimal_weight) = optimal_weights.get(current_instrument) {
                let diff = optimal_weight.weight - weight;
                allocation_change.insert(current_instrument.clone(), diff);
            } else {
                allocation_change.insert(current_instrument.clone(), -weight);
            }
        }
        for (instrument, weight) in allocation_change.iter() {
            info!("Change to optimal allocation for {} would be {}", instrument, weight);
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

            // Skip if quantity is zero
            let value = diff.abs() * tick.mid_price();
            if value < self.min_trade_value {
                info!(
                    "Skipping trade for {} as value of {} is below minimum trade size of {}",
                    instrument, value, self.min_trade_value
                );
                continue;
            }

            // Determine order side
            let order_side = if diff.is_sign_positive() {
                MarketSide::Buy
            } else {
                MarketSide::Sell
            };

            // Create execution order
            let order = ExecutionOrder::builder()
                .id(Uuid::new_v4())
                .portfolio(test_portfolio())
                .instrument(instrument.clone())
                .order_type(ExecutionOrderType::Maker)
                .side(order_side)
                .quantity(diff.abs())
                .created_at(event_time)
                .updated_at(event_time)
                .build();
            execution_orders.push(order.into());
        }
        for order in execution_orders.iter() {
            info!("Execution order: {}", order);
        }

        for order in execution_orders.iter() {
            self.pubsub.publish::<ExecutionOrder>(order.clone());
        }

        Ok(execution_orders)
    }
}
