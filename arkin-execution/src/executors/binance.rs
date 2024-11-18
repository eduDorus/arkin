use async_trait::async_trait;
use flume::{Receiver, Sender};
use tokio::time::{self, Duration};
use tracing::info;

use crate::{
    errors::ExecutorError,
    executor_protocol::{ExecutorRequest, ExecutorResponse, Order, OrderStatus},
};

use super::Executor;

pub struct BinanceExecutor {
    command_rx: Receiver<ExecutorRequest>,
    message_tx: Sender<ExecutorResponse>,
}

impl BinanceExecutor {
    pub fn new(message_tx: Sender<ExecutorResponse>) -> (Self, Sender<ExecutorRequest>) {
        let (command_tx, command_rx) = flume::unbounded::<ExecutorRequest>();
        (
            Self {
                command_rx,
                message_tx,
            },
            command_tx,
        )
    }

    pub async fn place_order(order: Order) -> Result<(), ExecutorError> {
        // Simulate order placement
        info!("Placing order {:?}", order);

        // Simulate success or failure
        if order.quantity <= 0.0 {
            return Err(ExecutorError::InvalidOrder("Amount must be positive".into()));
        }

        // Simulate network delay
        time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    pub async fn cancel_order(order_id: u64) {
        info!("Canceling order {}", order_id);

        // Simulate cancellation
        time::sleep(Duration::from_millis(50)).await;
    }
}

#[async_trait]
impl Executor for BinanceExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    async fn start(&self, command_rx: Receiver<ExecutorRequest>, message_tx: Sender<ExecutorResponse>) {
        info!("{} executor started.", self.name());

        while let Ok(command) = command_rx.recv_async().await {
            match command {
                ExecutorRequest::PlaceOrder(order) => {
                    // Simulate order placement
                    info!("{}: Placing order {:?}", self.name(), order);

                    // Simulate success or failure
                    if order.quantity <= 0.0 {
                        let error = ExecutorError::InvalidOrder("Amount must be positive".into());
                        message_tx.send_async(ExecutorResponse::Error(error)).await.unwrap();
                    } else {
                        // Simulate network delay
                        time::sleep(Duration::from_millis(100)).await;

                        // Send success message
                        message_tx
                            .send_async(ExecutorResponse::OrderUpdate(order.id, OrderStatus::Accepted))
                            .await
                            .unwrap();
                    }
                }
                ExecutorRequest::CancelOrder(order_id) => {
                    info!("{}: Canceling order {}", self.name(), order_id);

                    // Simulate cancellation
                    time::sleep(Duration::from_millis(50)).await;

                    message_tx
                        .send_async(ExecutorResponse::OrderUpdate(order_id, OrderStatus::Canceled))
                        .await
                        .unwrap();
                }
                ExecutorRequest::CancelAllOrders => {
                    info!("{}: Canceling all orders", self.name());
                }
                ExecutorRequest::Shutdown => {
                    info!("{} executor shutting down.", self.name());
                    break;
                }
            }
        }
    }
}
