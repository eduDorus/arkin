#![allow(dead_code)]
use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tokio::{
    select,
    sync::{Mutex, RwLock},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use typed_builder::TypedBuilder;

use arkin_accounting::prelude::*;
use arkin_core::prelude::*;
use uuid::Uuid;

use crate::{AllocationOptim, AllocationOptimError, AllocationService};

#[derive(TypedBuilder)]
pub struct SignalAllocationOptim {
    pubsub: PubSubHandle,
    accounting: Arc<dyn Accounting>,
    #[builder(default)]
    prices: Mutex<HashMap<Arc<Instrument>, Arc<Tick>>>,
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
    async fn update_signal(&self, signal: Arc<Signal>) -> bool {
        let key = (signal.strategy.clone(), signal.instrument.clone());
        let mut lock = self.strategy_signals.write().await;
        if let Some(existing_signal) = lock.get(&key) {
            if existing_signal.weight == signal.weight {
                debug!("Signal for {} has not changed, skipping optimization", signal.instrument);
                return false;
            }
        }
        lock.insert(key, signal);
        return true;
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
        debug!("Available capital for {}: {}", signal.instrument, capital);
        if capital.is_zero() {
            warn!("No capital available for allocation");
            return Decimal::ZERO;
        }

        // Get combined weight of all active signals
        let num_signals = self.signals_count().await;
        debug!("Number of active signals: {}", num_signals);

        let capital_per_signal = (capital * self.max_allocation) / Decimal::from(num_signals);
        debug!("Capital per signal: {}", capital_per_signal);
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
        debug!("Current position for {}: {}", instrument, current_position);

        // Get current price
        let tick = self
            .prices
            .lock()
            .await
            .get(instrument)
            .cloned()
            .ok_or(AllocationOptimError::NoPriceDataAvailable(instrument.clone()))?;
        let price = tick.mid_price();
        debug!("Current price for {}: {}", instrument, price);

        // Calculate desired portfolio value and quantity based on signal weight
        let desired_allocated_capital = (signal.weight * capital_per_signal) / price;
        debug!("Desired allocated capital for {}: {}", instrument, desired_allocated_capital);

        // Step 3: Check if current position matches desired position
        let diff_quantity = desired_allocated_capital - current_position;
        debug!("Difference in quantity: {}", diff_quantity);

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
        debug!("Trade side for {}: {}", instrument, side);

        // Step 6 & 7: Calculate trade quantity, capped by capital per signal
        let final_quantity = self.round_quantity(diff_quantity.abs(), instrument);
        debug!("Final quantity for {}: {}", instrument, final_quantity);

        // Validate trade value against minimum threshold
        let trade_value = final_quantity * price;
        if trade_value < self.min_trade_value {
            debug!(
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
    async fn optimize(&self, signal: Arc<Signal>) {
        // Update optimal allocation with the new signal
        let optimise = self.update_signal(signal.clone()).await;

        if optimise {
            info!("Signal updated for {}: {}", signal.instrument, signal.weight);
            // Calculate capital per signal
            let capital_per_signal = self.calculate_capital_per_signal(&signal).await;

            // Process each signal and collect orders
            let res = self.process_signal(&signal, capital_per_signal).await;

            match res {
                Ok(Some(order)) => self.pubsub.publish(Event::ExecutionOrderNew(order.clone())).await,
                Err(e) => warn!("{}", e),
                _ => debug!("No New Execution Orders"),
            }
        } else {
            info!("Signal for {} has not changed, skipping optimization", signal.instrument);
        }
    }
}

#[async_trait]
impl RunnableService for SignalAllocationOptim {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting LimitedAllocation...");

        loop {
            select! {
                Some(event) = self.pubsub.recv() => {
                    match event {
                        Event::TickUpdate(tick) => {
                          self.prices.lock().await.insert(tick.instrument.clone(), tick);
                        },
                        Event::SignalUpdate(signal) => self.optimize(signal).await,
                        _ => {}
                    }
                    self.pubsub.ack().await;
                }
                _ = shutdown.cancelled() => {
                    info!("LimitedAllocationOptim shutdown...");
                    break;
                }
            }
        }
        info!("LimitedAllocationOptim stopped.");
        Ok(())
    }
}

#[async_trait]
impl AllocationService for SignalAllocationOptim {}
