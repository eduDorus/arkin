use std::{pin::Pin, sync::Arc};

use async_trait::async_trait;

use arkin_core::prelude::*;
use tracing::{debug, info};

use crate::{DataProviderConfig, ProviderFactory, WsClient};

pub struct DataProviderService {
    config: DataProviderConfig,
}

impl DataProviderService {
    pub fn new(config: DataProviderConfig) -> Self {
        Self { config }
    }

    pub fn load_config() -> Self {
        let config = load::<DataProviderConfig>();
        Self { config }
    }
}

pub async fn start_ws_task(mut client: WsClient, core_ctx: Arc<CoreCtx>, service_ctx: Arc<ServiceCtx>) {
    // Start the client
    let shutdown = service_ctx.get_shutdown_token();

    let (tx, rx) = kanal::unbounded_async();
    let ws_client_shutdown = shutdown.clone();
    service_ctx.spawn(async move {
        client.run(tx, ws_client_shutdown).await;
    });

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
              debug!("Received message from WS client: {:?}", msg);
              debug!("Publishing message to pubsub");
              core_ctx.publish(msg).await;
            }
            _ = shutdown.cancelled() => {
              info!("Shutdown signal received, stopping WS client task");
                break;
            }
        }
    }
}

#[async_trait]
impl Runnable for DataProviderService {
    async fn get_tasks(
        self: Arc<Self>,
        service_ctx: Arc<ServiceCtx>,
        core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        let (_http_providers, ws_providers) =
            ProviderFactory::from_config(self.config.clone(), core_ctx.persistence.clone());
        let mut tasks: Vec<Pin<Box<dyn Future<Output = ()> + Send>>> = vec![];

        // Start WS Providers
        for ws_config in ws_providers {
            tasks.push(Box::pin(start_ws_task(ws_config, core_ctx.clone(), service_ctx.clone())));
        }
        tasks
    }
}
