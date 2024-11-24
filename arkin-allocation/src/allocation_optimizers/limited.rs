#![allow(dead_code)]
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use dashmap::DashMap;
use derive_builder::Builder;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument, warn};

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use arkin_portfolio::prelude::*;

use crate::{AllocationOptim, AllocationOptimError};

#[derive(Debug, Builder)]
pub struct LimitedAllocationOptim {
    pubsub: Arc<PubSub>,
    persistence: Arc<dyn Persistor>,
    portfolio: Arc<dyn Portfolio>,
    #[builder(default = "DashMap::new()")]
    signals: DashMap<(StrategyId, Arc<Instrument>), Signal>,
    #[builder(default = "Decimal::from_f32(0.8).unwrap()")]
    max_allocation: Decimal,
    #[builder(default = "Decimal::from_f32(0.2).unwrap()")]
    max_allocation_per_signal: Decimal,
}

#[async_trait]
impl AllocationOptim for LimitedAllocationOptim {
    #[instrument(skip_all)]
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), AllocationOptimError> {
        info!("Starting LimitedAllocation...");
        let mut signals = self.pubsub.subscribe::<Signal>();
        loop {
            select! {
                Ok(signal) = signals.recv() => {
                    self.new_signal(signal).await?;
                }
                _ = _shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }

    #[instrument(skip_all)]
    async fn cleanup(&self) -> Result<(), AllocationOptimError> {
        info!("Cleaning up LimitedAllocation...");
        info!("LimitedAllocation cleaned up");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn list_signals(&self) -> Result<Vec<Signal>, AllocationOptimError> {
        Ok(self.signals.iter().map(|entry| entry.value().clone()).collect())
    }

    #[instrument(skip_all)]
    async fn new_signal(&self, signal: Signal) -> Result<(), AllocationOptimError> {
        let key = (signal.strateg_id.clone(), signal.instrument.clone());
        self.signals.insert(key, signal);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn new_signals(&self, signals: Vec<Signal>) -> Result<(), AllocationOptimError> {
        for signal in signals {
            self.new_signal(signal).await?;
        }
        Ok(())
    }

    #[instrument(skip_all)]
    async fn optimize(&self, event_time: OffsetDateTime) -> Result<Vec<ExecutionOrder>, AllocationOptimError> {
        // Check if we have any signals
        if self.signals.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate money allocated to each signal
        let max_allocation = self.portfolio.capital().await * self.max_allocation;
        let max_allocation_per_signal = max_allocation * self.max_allocation_per_signal;

        // Calculate optimal position for each signal
        let mut optimal_positions = HashMap::new();
        for entry in self.signals.iter() {
            let signal = entry.value();
            let signal_allocation = max_allocation_per_signal * signal.weight;
            let res = self.persistence.read_latest_tick(event_time, &signal.instrument).await?;
            match res {
                Some(tick) => {
                    let quantity = signal_allocation / tick.mid_price();
                    optimal_positions.insert(signal.instrument.clone(), quantity);
                }
                None => {
                    warn!("No tick found for instrument: {}", signal.instrument);
                }
            }
        }

        // Calculate difference between current and expected positions
        let current_positions = self.portfolio.positions().await;
        let mut position_diff = HashMap::new();
        for (instrument, expected_quantity) in optimal_positions.iter() {
            let order_quantity = if let Some(position) = current_positions.get(instrument) {
                (expected_quantity - position.quantity).round_dp(instrument.quantity_precision)
            } else {
                expected_quantity.round_dp(instrument.quantity_precision)
            };
            position_diff.insert(instrument, order_quantity);
        }

        // Create execution orders
        let mut execution_orders = Vec::with_capacity(position_diff.len());
        for (instrument, quantity) in position_diff.into_iter() {
            // Skip if quantity is zero
            if quantity == Decimal::zero() {
                continue;
            }

            // Determine order side
            let order_side = if quantity > Decimal::zero() {
                MarketSide::Buy
            } else {
                MarketSide::Sell
            };

            // Create execution order
            // let order = ExecutionOrderBuilder::default()
            //     .event_time(event_time)
            //     .instrument(instrument.clone())
            //     .execution_type(ExecutionOrderStrategy::Market {
            //         side: order_side,
            //         quantity: quantity.abs(),
            //         split: false,
            //         vwap: false,
            //     })
            //     .side(order_side)
            //     .quantity(quantity.abs())
            //     .build()
            //     .expect("Failed to build ExecutionOrder");

            // execution_orders.push(order);
        }

        Ok(execution_orders)
    }
}
