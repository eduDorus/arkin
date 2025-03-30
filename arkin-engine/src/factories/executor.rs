use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_executors::prelude::*;
use arkin_persistence::prelude::*;

use crate::config::ExecutorConfig;

pub struct ExecutorFactory {}

impl ExecutorFactory {
    pub async fn init(_pubsub: Arc<PubSub>, _persistence: Arc<PersistenceService>) -> Arc<dyn ExecutorService> {
        let config = load::<ExecutorConfig>();
        let _executor: Arc<dyn ExecutorService> = match &config.executors {
            // ExecutorTypeConfig::Simulation(_c) => Arc::new(SimulationExecutor::builder().pubsub(pubsub).build()),
            // ExecutorTypeConfig::Binance(c) => Arc::new(
            //     BinanceExecutor::builder()
            //         .pubsub(pubsub)
            //         .persistence(persistence)
            //         .client(Arc::new(
            //             BinanceHttpClient::builder()
            //                 .base_url(Url::parse(&c.base_url).expect("Invalid URL for binance http client"))
            //                 .credentials(Some(Credentials::from_hmac(c.api_key.clone(), c.api_secret.clone())))
            //                 .build(),
            //         ))
            //         .api_key(c.api_key.clone())
            //         .no_trade(c.no_trade)
            //         .build(),
            // ),
            _ => unimplemented!(),
        };
    }

    pub async fn init_simulation(pubsub: Arc<PubSub>) -> Arc<dyn ExecutorService> {
        Arc::new(SimulationExecutor::builder().pubsub(pubsub.handle().await).build())
    }
}
