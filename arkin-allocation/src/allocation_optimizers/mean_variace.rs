use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use rust_decimal::Decimal;
use tokio::{select, sync::Mutex};
use tokio_util::sync::CancellationToken;
use tracing::info;
use typed_builder::TypedBuilder;

use arkin_accounting::prelude::*;
use arkin_core::prelude::*;

use crate::{AllocationOptim, AllocationService};

#[derive(TypedBuilder)]
pub struct MeanVarianceOptim {
    pubsub: PubSubHandle,
    accounting: Arc<dyn Accounting>,
    #[builder(default)]
    returns: Mutex<HashMap<Arc<Instrument>, f64>>,
    #[builder(default)]
    signals: Mutex<HashMap<(Arc<Strategy>, Arc<Instrument>), Decimal>>,
}

#[async_trait]
impl AllocationOptim for MeanVarianceOptim {
    async fn optimize(&self, signal: Arc<Signal>) {
        
    }
}

#[async_trait]
impl RunnableService for MeanVarianceOptim {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting LimitedAllocation...");

        loop {
            select! {
                Some(event) = self.pubsub.recv() => {
                    match event {
                        Event::InsightsUpdate(insights) => {
                          // This event contains all insights like returns etc.
                        },
                        Event::SignalUpdate(signal) => {
                            // This event contains a new signal update.
                        },
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
impl AllocationService for MeanVarianceOptim {}
