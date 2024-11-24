use std::{sync::Arc, time::Duration};

use arkin_core::prelude::*;
use mimalloc::MiMalloc;
use tracing::info;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();
    info!("Starting Arkin Order Manager 🚀");

    let pubsub = Arc::new(PubSub::new());

    pubsub.publish::<InsightTick>(InsightTick {});

    let mut rx_insight_tick = pubsub.subscribe::<InsightTick>();
    tokio::spawn(async move {
        while let Ok(insight_tick) = rx_insight_tick.recv().await {
            info!("Received insight tick: {:?}", insight_tick);
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;
    pubsub.publish::<InsightTick>(InsightTick {});

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
}
