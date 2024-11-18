use async_trait::async_trait;
use derive_builder::Builder;
use rust_decimal::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, instrument};

use arkin_core::prelude::*;

use crate::{AllocationOptim, AllocationOptimError};

#[derive(Debug, Builder)]
#[builder(setter(into))]
#[allow(unused)]
pub struct LimitedAllocationOptim {
    max_allocation: Decimal,
    max_allocation_per_signal: Decimal,
}

#[async_trait]
impl AllocationOptim for LimitedAllocationOptim {
    #[instrument(skip(self))]
    async fn start(
        &self,
        _task_tracker: TaskTracker,
        _shutdown: CancellationToken,
    ) -> Result<(), AllocationOptimError> {
        info!("Starting LimitedAllocation...");
        info!("LimitedAllocation started");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cleanup(&self) -> Result<(), AllocationOptimError> {
        info!("Cleaning up LimitedAllocation...");
        info!("LimitedAllocation cleaned up");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn new_signal(&self, _signal: Signal) -> Result<(), AllocationOptimError> {
        Ok(())
    }

    #[instrument(skip(self))]
    async fn new_signals(&self, _signals: Vec<Signal>) -> Result<(), AllocationOptimError> {
        Ok(())
    }

    #[instrument(skip(self))]
    async fn optimize(&self) -> Result<Vec<ExecutionOrder>, AllocationOptimError> {
        Ok(vec![])
        // let signals = strategy_snapshot.signals();

        // // Check if we have any signals
        // if signals.len() == 0 {
        //     return Vec::new();
        // }

        // // Calculate money allocated to each signal
        // let max_allocation = portfolio_snapshot.capital() * self.max_allocation;
        // let max_allocation_per_signal = portfolio_snapshot.capital() * self.max_allocation_per_signal;
        // let allocation_per_signal = (max_allocation / Decimal::from(signals.len())).min(max_allocation_per_signal);

        // // Calculate current position size for each signal
        // let current_positions = portfolio_snapshot
        //     .positions()
        //     .into_iter()
        //     .map(|position| {
        //         let quantity = match position.side {
        //             PositionSide::Long => position.quantity,
        //             PositionSide::Short => -position.quantity,
        //         };
        //         ((position.strategy, position.instrument), quantity)
        //     })
        //     .collect::<HashMap<_, _>>();

        // // Calculate expected position size for each signal
        // let expected_positions = signals
        //     .into_iter()
        //     .map(|signal| {
        //         let signal_allocation = allocation_per_signal * signal.weight;
        //         let current_tick = market_snapshot.last_tick(&signal.instrument).unwrap();
        //         let quantityu = signal_allocation / current_tick.mid_price();
        //         ((signal.strategy, signal.instrument), quantityu)
        //     })
        //     .collect::<HashMap<_, _>>();

        // // Calculate the difference between current and expected positions
        // let position_diff = expected_positions
        //     .into_iter()
        //     .map(|(key, expected_quantity)| {
        //         let current_quantity = current_positions.get(&key).unwrap_or(Decimal::zero()).to_owned();
        //         info!("Expected Amount: {} Current Amount: {}", expected_quantity, current_quantity);
        //         let diff = (expected_quantity - current_quantity).round_dp(4);
        //         (key, diff)
        //     })
        //     .collect::<HashMap<_, _>>();

        // // Calculate the orders to be executed
        // position_diff
        //     .into_iter()
        //     .filter_map(|((strategy_id, instrument), quantity)| {
        //         if quantity == Decimal::zero() {
        //             return None;
        //         }

        //         let order_side = if quantity > Decimal::zero() {
        //             Side::Buy
        //         } else {
        //             Side::Sell
        //         };
        //         let order_price = market_snapshot.last_tick(&instrument).unwrap().mid_price();
        //         info!("Order price: {}", order_price);

        //         let order = ExecutionOrder::new(
        //             market_snapshot.timestamp(),
        //             0,
        //             strategy_id.to_owned(),
        //             instrument.to_owned(),
        //             order_side,
        //             ExecutionOrderType::Maker,
        //             VenueOrderTimeInForce::Gtc,
        //             quantity.abs(),
        //         );
        //         Some(order)
        //     })
        //     .collect::<Vec<_>>()
    }
}
