#![allow(dead_code)]
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tokio::{select, sync::RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use typed_builder::TypedBuilder;

use arkin_accounting::prelude::*;
use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use uuid::Uuid;

use crate::{AllocationOptim, AllocationOptimError, AllocationService};

#[derive(Debug, TypedBuilder)]
pub struct SignalAllocationOptim {
    pubsub: Arc<PubSub>,
    persistence: Arc<PersistenceService>,
    accounting: Arc<dyn Accounting>,
    #[builder(default = RwLock::new(HashMap::new()))]
    strategy_signals: RwLock<HashMap<(Arc<Strategy>, Arc<Instrument>), Arc<Signal>>>,
    leverage: Decimal,
    min_trade_value: Decimal,
    #[builder(default = dec!(0.9))]
    max_allocation: Decimal,
    allocation_feature_id: FeatureId,
    reference_currency: Arc<Asset>,
}

impl SignalAllocationOptim {
    async fn insert_signal(&self, signal: Arc<Signal>) {
        let key = (signal.strategy.clone(), signal.instrument.clone());
        let mut lock = self.strategy_signals.write().await;
        lock.insert(key, signal);
    }

    async fn signals_count(&self) -> usize {
        let lock = self.strategy_signals.read().await;
        lock.len()
    }

    async fn calculate_capital_per_signal(&self, signal: &Arc<Signal>) -> Decimal {
        // Get available capital for allocation
        let venue = &signal.instrument.venue;
        let margin_asset = &signal.instrument.margin_asset;
        let capital = self.accounting.asset_margin_balance(venue, margin_asset).await;
        info!("Available capital for {}: {}", signal.instrument, capital);
        if capital.is_zero() {
            warn!("No capital available for allocation");
            return Decimal::ZERO;
        }

        // Get combined weight of all active signals
        let num_signals = self.signals_count().await;
        info!("Number of active signals: {}", num_signals);

        let capital_per_signal = (capital * self.max_allocation) / Decimal::from(num_signals);
        info!("Capital per signal: {}", capital_per_signal);
        capital_per_signal
    }

    /// Processes a single signal to determine if a trade is needed and creates an order.
    async fn process_signal(
        &self,
        signal: &Signal,
        capital_per_signal: Decimal,
    ) -> Result<Option<Arc<ExecutionOrder>>, AllocationOptimError> {
        let instrument = &signal.instrument;

        // Get current position
        let current_position = self.accounting.strategy_instrument_position(&signal.strategy, instrument).await;
        info!("Current position for {}: {}", instrument, current_position);

        // Get current price
        let tick = self
            .persistence
            .tick_store
            .get_last_tick(instrument)
            .await
            .ok_or(AllocationOptimError::NoPriceDataAvailable(instrument.clone()))?;
        let price = tick.mid_price();
        info!("Current price for {}: {}", instrument, price);

        // Calculate desired portfolio value and quantity based on signal weight
        let desired_allocated_capital = (signal.weight * capital_per_signal) / price;
        info!("Desired allocated capital for {}: {}", instrument, desired_allocated_capital);

        // Step 3: Check if current position matches desired position
        let diff_quantity = desired_allocated_capital - current_position;
        info!("Difference in quantity: {}", diff_quantity);

        // Step 4: If difference is negligible, skip trading
        if diff_quantity.abs() < instrument.lot_size {
            debug!("Position for {} is already optimal", instrument);
            return Ok(None);
        }

        // Determine trade direction
        let side = if diff_quantity > Decimal::ZERO {
            MarketSide::Buy
        } else {
            MarketSide::Sell
        };
        info!("Trade side for {}: {}", instrument, side);

        // Step 6 & 7: Calculate trade quantity, capped by capital per signal
        let final_quantity = self.round_quantity(diff_quantity.abs(), instrument);
        info!("Final quantity for {}: {}", instrument, final_quantity);

        // Validate trade value against minimum threshold
        let trade_value = final_quantity * price;
        if trade_value < self.min_trade_value {
            info!(
                "Skipping trade for {}: value {} below minimum {}",
                instrument, trade_value, self.min_trade_value
            );
            return Ok(None);
        }

        // Step 8: Create execution order
        let order = ExecutionOrder::builder()
            .id(Uuid::new_v4())
            .event_time(tick.event_time)
            .strategy(Some(signal.strategy.clone()))
            .instrument(instrument.clone())
            .order_type(ExecutionOrderType::Taker)
            .side(side)
            .quantity(final_quantity)
            .price(price)
            .updated_at(tick.event_time)
            .build();
        info!("Created execution order: {}", order);

        Ok(Some(Arc::new(order)))
    }

    /// Rounds the quantity to the instrument's lot size and precision.
    fn round_quantity(&self, quantity: Decimal, instrument: &Instrument) -> Decimal {
        let scaling_factor = Decimal::ONE / instrument.lot_size;
        let scaled = quantity * scaling_factor;
        let rounded = scaled.round();
        (rounded / scaling_factor).round_dp(instrument.quantity_precision)
    }
}

#[async_trait]
impl AllocationOptim for SignalAllocationOptim {
    async fn optimize(&self, signal: Arc<Signal>) -> Result<Arc<ExecutionOrder>, AllocationOptimError> {
        // Update optimal allocation with the new signal
        self.insert_signal(signal.clone()).await;

        // Step 6: Calculate capital per signal with 30% cap
        info!("Calculating capital per signal...");
        let capital_per_signal = self.calculate_capital_per_signal(&signal).await;

        // Process each signal and collect orders
        info!("Processing signal...");
        let execution_order = self.process_signal(&signal, capital_per_signal).await?;

        // Publish orders
        info!("Publishing orders...");
        if let Some(order) = execution_order {
            info!("Publishing order: {}", order);
            self.pubsub.publish(order.clone()).await;
            return Ok(order);
        } else {
            warn!("No orders to publish");
            return Err(AllocationOptimError::NoOrdersToPublish(signal));
        }
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
