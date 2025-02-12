use std::{str::FromStr, sync::Arc};

use url::Url;

use arkin_binance::prelude::*;
use arkin_core::prelude::*;
use arkin_persistence::prelude::*;

use crate::{
    config::{ExecutorConfig, ExecutorTypeConfig},
    executors::{BinanceExecutor, SimulationExecutor},
    traits::ExecutorService,
};

pub struct ExecutorFactory {}

impl ExecutorFactory {
    pub fn init(pubsub: Arc<PubSub>, persistence: Arc<PersistenceService>) -> Arc<dyn ExecutorService> {
        let config = load::<ExecutorConfig>();
        let executor: Arc<dyn ExecutorService> = match &config.executors {
            // ExecutorTypeConfig::Simulation(_c) => Arc::new(SimulationExecutor::builder().pubsub(pubsub).build()),
            ExecutorTypeConfig::Binance(c) => Arc::new(
                BinanceExecutor::builder()
                    .pubsub(pubsub)
                    .persistence(persistence)
                    .client(Arc::new(
                        BinanceHttpClient::builder()
                            .base_url(Url::from_str(&c.base_url).expect("Invalid URL for binance http client"))
                            .credentials(Some(Credentials::from_hmac(c.api_key.clone(), c.api_secret.clone())))
                            .build(),
                    ))
                    .api_key(c.api_key.clone())
                    .no_trade(c.no_trade)
                    .build(),
            ),
            _ => unimplemented!(),
        };

        executor
    }

    pub fn init_simulation(pubsub: Arc<PubSub>) -> Arc<dyn ExecutorService> {
        Arc::new(SimulationExecutor::builder().pubsub(pubsub).build())
    }
}
