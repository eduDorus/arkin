use async_trait::async_trait;
use derive_builder::Builder;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, instrument};

use arkin_core::prelude::*;

use crate::{Executor, ExecutorError};

#[derive(Debug, Builder)]
pub struct SimulationExecutor {}

#[async_trait]
impl Executor for SimulationExecutor {
    #[instrument(skip(self))]
    async fn start(&self, _task_tracker: TaskTracker, _shutdown: CancellationToken) -> Result<(), ExecutorError> {
        info!("Starting simulation executor...");
        info!("Simulation executor started");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cleanup(&self) -> Result<(), ExecutorError> {
        info!("Cleaning up simulation executor...");
        info!("Simulation executor cleaned up");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn place_order(&self, order: VenueOrder) -> Result<(), ExecutorError> {
        info!("Placing order: {:?}", order);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel_order(&self, order_id: u64) -> Result<(), ExecutorError> {
        info!("Cancelling order: {:?}", order_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel_all_orders(&self) -> Result<(), ExecutorError> {
        info!("Cancelling all orders");
        Ok(())
    }
}
