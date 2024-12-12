use std::{str::FromStr, sync::Arc};

use arkin_binance::{BinanceHttpClient, Credentials};
use arkin_core::PubSub;
use arkin_persistence::PersistenceService;
use url::Url;

use crate::{Executor, ExecutorConfig, ExecutorTypeConfig};

use super::BinanceExecutor;

pub struct ExecutorFactory {}

impl ExecutorFactory {
    pub fn from_config(
        config: &ExecutorConfig,
        pubsub: Arc<PubSub>,
        persistence: Arc<PersistenceService>,
    ) -> Arc<dyn Executor> {
        let executor: Arc<dyn Executor> = match &config.executor {
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
                    .build(),
            ),
            _ => unimplemented!(),
        };

        executor
    }
}
