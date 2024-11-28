#![allow(dead_code)]
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use dashmap::DashMap;
use derive_builder::Builder;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use time::OffsetDateTime;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

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
    #[builder(default = "Decimal::from_f32(1.0).unwrap()")]
    max_allocation_per_signal: Decimal,
    #[builder(default = "dec!(100)")]
    min_trade_value: Decimal,
    #[builder(default = "AssetId::from(\"USDT\".to_string())")]
    reference_currency: AssetId,
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
        let mut signal_tick = self.pubsub.subscribe::<SignalTick>();
        loop {
            select! {
                Ok(tick) = signal_tick.recv() => {
                    info!("LimitedAllocationOptim received signal tick: {}", tick.event_time);
                    self.new_signals(tick.signals).await?;
                    self.optimize(tick.event_time).await?;
                }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn list_signals(&self) -> Result<Vec<Signal>, AllocationOptimError> {
        Ok(self.signals.iter().map(|entry| entry.value().clone()).collect())
    }

    async fn new_signal(&self, signal: Signal) -> Result<(), AllocationOptimError> {
        info!("Received new signal: {}", signal);
        let key = (signal.strateg_id.clone(), signal.instrument.clone());
        self.signals.insert(key, signal);
        Ok(())
    }

    async fn new_signals(&self, signals: Vec<Signal>) -> Result<(), AllocationOptimError> {
        for signal in signals {
            self.new_signal(signal).await?;
        }
        Ok(())
    }

    async fn optimize(&self, event_time: OffsetDateTime) -> Result<Vec<ExecutionOrder>, AllocationOptimError> {
        // Check if we have any signals
        if self.signals.is_empty() {
            warn!("No signals found for optimization");
            return Ok(Vec::new());
        }

        // Calculate money allocated to each signal
        let capital = self.portfolio.capital(&self.reference_currency).await;
        let max_allocation = capital * self.max_allocation;
        let amount_of_signals = Decimal::from_usize(self.signals.len()).expect("Failed to convert usize to Decimal");
        let max_allocation_per_signal = (max_allocation / amount_of_signals) * self.max_allocation_per_signal;
        info!("Capital: {}", capital);
        info!("Max allocation: {}", max_allocation);
        info!("Max allocation per signal: {}", max_allocation_per_signal);

        // Get current positions
        let current_positions = self.portfolio.list_open_positions().await;
        for (_, position) in current_positions.iter() {
            info!("Current position {}", position);
        }

        // Calculate optimal position for each signal
        let mut optimal_positions = HashMap::new();
        for entry in self.signals.iter() {
            let signal = entry.value();
            let signal_allocation = max_allocation_per_signal * signal.weight;
            let res = self.persistence.read_latest_tick(event_time, &signal.instrument).await?;
            match res {
                Some(tick) => {
                    let quantity = signal_allocation / tick.mid_price();
                    let optimal_position = OptimalPosition {
                        instrument: signal.instrument.clone(),
                        price: tick.mid_price(),
                        quantity: quantity
                            .round_dp_with_strategy(signal.instrument.quantity_precision, RoundingStrategy::ToZero),
                    };
                    optimal_positions.insert(signal.instrument.clone(), optimal_position);
                }
                None => {
                    warn!("No tick found for instrument: {}", signal.instrument);
                }
            }
        }
        for (instrument, position) in optimal_positions.iter() {
            info!(
                "Optimal position for {} with price {} and quantity {}",
                instrument, position.price, position.quantity
            );
        }

        // Calculate difference between current and expected positions
        let current_positions = self
            .portfolio
            .list_open_positions_with_quote_asset(&self.reference_currency)
            .await;
        let mut position_diff = HashMap::new();
        for (instrument, optimal_position) in optimal_positions.iter() {
            let diff_position = if let Some(position) = current_positions.get(instrument) {
                // TODO: Handle position side
                let diff = (optimal_position.quantity - position.quantity_with_side())
                    .round_dp_with_strategy(instrument.quantity_precision, RoundingStrategy::ToZero);
                DiffPosition {
                    instrument: instrument.clone(),
                    price: optimal_position.price,
                    quantity: optimal_position.quantity,
                    diff,
                }
            } else {
                let diff = optimal_position
                    .quantity
                    .round_dp_with_strategy(instrument.quantity_precision, RoundingStrategy::ToZero);
                DiffPosition {
                    instrument: instrument.clone(),
                    price: optimal_position.price,
                    quantity: optimal_position.quantity,
                    diff,
                }
            };
            position_diff.insert(instrument, diff_position);
        }
        for (instrument, position) in position_diff.iter() {
            info!(
                "Change to optimal position for {} with price {} would be {}",
                instrument, position.price, position.diff
            );
        }

        // Create execution orders
        let mut execution_orders = Vec::with_capacity(position_diff.len());
        for (instrument, position) in position_diff.into_iter() {
            // Skip if quantity is zero
            let value = position.price * position.diff.abs();
            if value < self.min_trade_value {
                info!(
                    "Skipping trade for {} as value of {} is below minimum trade size of {}",
                    instrument, value, self.min_trade_value
                );
                continue;
            }

            // Determine order side
            let order_side = if position.diff > Decimal::zero() {
                MarketSide::Buy
            } else {
                MarketSide::Sell
            };

            // Create execution order
            let order = ExecutionOrderBuilder::default()
                .event_time(event_time)
                .instrument(instrument.clone())
                .execution_type(ExecutionOrderStrategy::Market(
                    MarketBuilder::default()
                        .side(order_side)
                        .quantity(position.diff.abs())
                        .build()
                        .unwrap(),
                ))
                .build()
                .expect("Failed to build ExecutionOrder");

            execution_orders.push(order);
        }

        for order in execution_orders.iter() {
            info!("Publishing execution order: {}", order);
            self.pubsub.publish::<ExecutionOrder>(order.clone());
        }

        Ok(execution_orders)
    }
}
