use std::{pin::Pin, sync::Arc};

use async_trait::async_trait;

use arkin_core::prelude::*;

use crate::{WsClient, WsConfig};

pub struct WsService {
    configs: Vec<WsConfig>,
}

impl WsService {
    pub fn new(configs: Vec<WsConfig>) -> Self {
        Self { configs }
    }
}

pub async fn start_ws_task(ws_config: WsConfig, core_ctx: Arc<CoreCtx>, service_ctx: Arc<ServiceCtx>) {
    let (mut client, mut receiver) = WsClient::new(ws_config);

    let shutdown = service_ctx.get_shutdown_token();

    loop {
        tokio::select! {
            Some(msg) = receiver.recv() => {
                // Here you would typically parse the message and handle it
                // For example:
                // let parser = ParserFactory::get_parser(venue, instrument_type).unwrap();
                // match parser.parse(&msg) {
                //     Ok(event) => { /* process event */ }
                //     Err(e) => { /* handle error */ }
                // }
            }
            _ = shutdown.cancelled() => {
                break;
            }
        }
    }
}

#[async_trait]
impl Runnable for WsService {
    async fn get_tasks(
        self: Arc<Self>,
        service_ctx: Arc<ServiceCtx>,
        core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        vec![Box::pin(start_md_task(self, core_ctx.clone(), service_ctx.clone()))]
    }
}
