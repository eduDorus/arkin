use std::{sync::Arc, time::Duration};

use arkin_core::prelude::*;
use mimalloc::MiMalloc;
use time::OffsetDateTime;
use tracing::info;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();
    info!("Starting Arkin Order Manager 🚀");

    let pubsub = Arc::new(PubSub::new());

    let mut rx_insight_tick = pubsub.subscribe::<InsightTick>();
    tokio::spawn(async move {
        while let Ok(insight_tick) = rx_insight_tick.recv().await {
            info!("Received insight tick: {:?}", insight_tick);
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;
    let tick = InsightTick::builder()
        .event_time(OffsetDateTime::now_utc())
        .insights(vec![])
        .instruments(vec![])
        .build()
        .into();
    pubsub.publish::<InsightTick>(tick);

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
}
