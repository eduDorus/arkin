use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use derive_builder::Builder;
use rust_decimal::prelude::*;
use tokio_util::sync::CancellationToken;
use tracing::info;

use arkin_core::prelude::*;

use crate::{ExecutionStrategy, OrderManager, StrategyError};

#[derive(Debug, Clone, Builder)]
pub struct WideQuoter {
    pubsub: PubSub,
    execution_order_id: ExecutionOrderId,
    order_manager: Arc<dyn OrderManager>,
    spread_from_mid: Decimal,
    requote_price_move_pct: Decimal,
    shutdown: CancellationToken,
}

#[async_trait]
impl ExecutionStrategy for WideQuoter {
    async fn start(&self) -> Result<(), StrategyError> {
        info!("Starting WideQuoter for order {}", self.execution_order_id);
        // let order = VenueOrderBuilder::default().execution_order_id(self.execution_order_id)..build().unwrap();
        // TODO: Place the order
        tokio::select! {
            _ = self.shutdown.cancelled() => {
                info!("Order {} is cancelled", self.execution_order_id);
            }
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                info!("Order {} is done", self.execution_order_id);
                let order = self.order_manager.order_by_id(self.execution_order_id).await.unwrap();
                let fill = FillBuilder::default()
                    .instrument(order.instrument.clone())
                    .venue_order_id(VenueOrderId::new_v4())
                    .execution_order_id(order.id)
                    .side(order.side)
                    .price(Price::from_f64(50000.0).unwrap())
                    .quantity(order.quantity)
                    .commission(Commission::from_f64(2.0).unwrap())
                    .build()
                    .unwrap();
                self.order_manager.order_update(fill).await.unwrap();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{MockExecutor, MockOrderManager};

    use super::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_wide_quoter() {
        // // Create a mock Executor
        // let mock_executor = MockExecutor::new();

        // // Create a mock OrderManager
        // let mut mock_order_manager = MockOrderManager::new();
        // // Create a test ExecutionOrder
        // let instrument = binance_btc_usdt_perp();
        // let first_order = ExecutionOrderBuilder::default()
        //     .instrument(instrument.clone())
        //     .execution_type(ExecutionOrderStrategy::Market {
        //         side: MarketSide::Buy,
        //         quantity: Quantity::from_f64(1.0).unwrap(),
        //         split: false,
        //         vwap: false,
        //     })
        //     .side(MarketSide::Buy)
        //     .quantity(Quantity::from_f64(1.0).unwrap())
        //     .build()
        //     .unwrap();
        // let id = first_order.id.clone();
        // mock_order_manager
        //     .expect_order_by_id()
        //     .returning(move |_id| Some(first_order.clone()));

        // // Create a WideQuoter
        // let wide_quoter = WideQuoterBuilder::default()
        //     .execution_order_id(id)
        //     .executor(Arc::new(mock_executor))
        //     .order_manager(Arc::new(mock_order_manager))
        //     .build()
        //     .unwrap();

        // // Call start
        // wide_quoter.start().await.unwrap();
    }
}
