use std::sync::Arc;

use arkin_core::{ExecutionOrder, ExecutionOrderStrategy, PubSub};
use tokio_util::sync::CancellationToken;

use crate::ExecutionStrategy;

use super::WideQuoterBuilder;

pub struct ExecutionStrategyFactory {}

impl ExecutionStrategyFactory {
    pub fn new(pubsub: Arc<PubSub>, order: ExecutionOrder, shutdown: CancellationToken) -> Arc<dyn ExecutionStrategy> {
        match order.execution_type {
            ExecutionOrderStrategy::Market(_s) => unimplemented!(),
            ExecutionOrderStrategy::Limit(_s) => unimplemented!(),
            ExecutionOrderStrategy::WideQuoting(s) => Arc::new(
                WideQuoterBuilder::default()
                    .execution_order_id(order.id)
                    .order_manager(order_manager)
                    .executor(executor)
                    .shutdown(shutdown)
                    .spread_from_mid(s.spread_from_mid)
                    .requote_price_move_pct(s.requote_price_move_pct)
                    .build()
                    .unwrap(),
            ),
        }
    }
}
