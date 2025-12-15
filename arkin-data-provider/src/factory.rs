use std::sync::Arc;

use arkin_core::PersistenceReader;

use crate::ws_providers::{
    BinanceSpotUserWsProvider, BinanceSpotWsProvider, BinanceUsdmUserWsProvider, BinanceUsdmWsProvider,
};
use crate::{DataProviderConfig, HttpProvider, ProviderConfig, WsClient};

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn from_config(
        config: DataProviderConfig,
        persistence: Arc<dyn PersistenceReader>,
    ) -> (Vec<Box<dyn HttpProvider>>, Vec<WsClient>) {
        let mut http_providers: Vec<Box<dyn HttpProvider>> = vec![];
        let mut ws_clients: Vec<WsClient> = vec![];

        for provider_config in config.data_providers {
            let mut https = Self::new_http(provider_config.clone(), persistence.clone());
            http_providers.append(&mut https);

            let mut wss = Self::new_ws(provider_config, persistence.clone());
            ws_clients.append(&mut wss);
        }

        (http_providers, ws_clients)
    }

    fn new_http(config: ProviderConfig, persistence: Arc<dyn PersistenceReader>) -> Vec<Box<dyn HttpProvider>> {
        match config {
            ProviderConfig::BinanceUsdmUser(c) => {
                if c.enabled {
                    vec![Box::new(
                        crate::http_providers::BinanceUsdmUserHttpProvider::builder()
                            .api_key(c.api_key)
                            .api_secret(c.api_secret)
                            .http_url(c.http_url.to_string())
                            .persistence(persistence)
                            .build(),
                    )]
                } else {
                    vec![]
                }
            }
            ProviderConfig::BinanceSpotUser(c) => {
                if c.enabled {
                    vec![Box::new(
                        crate::http_providers::BinanceSpotUserHttpProvider::builder()
                            .api_key(c.api_key)
                            .api_secret(c.api_secret)
                            .http_url(c.http_url.to_string())
                            .persistence(persistence)
                            .build(),
                    )]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    fn new_ws(config: ProviderConfig, persistence: Arc<dyn PersistenceReader>) -> Vec<WsClient> {
        let mut clients: Vec<WsClient> = vec![];

        match config {
            ProviderConfig::BinanceSpot(c) => {
                for endpoint in c.ws_endpoints {
                    if endpoint.enabled {
                        let provider = BinanceSpotWsProvider::builder()
                            .persistence(persistence.clone())
                            .url(c.ws_url.clone())
                            .channel(endpoint.channel)
                            .symbols(endpoint.symbols)
                            .build();
                        clients.push(WsClient::new(Box::new(provider)));
                    }
                }
            }
            ProviderConfig::BinanceUsdm(c) => {
                for endpoint in c.ws_endpoints {
                    if endpoint.enabled {
                        let provider = BinanceUsdmWsProvider::builder()
                            .persistence(persistence.clone())
                            .url(c.ws_url.clone())
                            .channel(endpoint.channel)
                            .symbols(endpoint.symbols)
                            .build();
                        clients.push(WsClient::new(Box::new(provider)));
                    }
                }
            }
            ProviderConfig::BinanceUsdmUser(c) => {
                if c.enabled {
                    let provider = BinanceUsdmUserWsProvider::builder()
                        .api_key(c.api_key)
                        .api_secret(c.api_secret)
                        .http_url(c.http_url)
                        .ws_url(c.ws_url)
                        .persistence(persistence.clone())
                        .build();
                    clients.push(
                        WsClient::builder()
                            .provider(Box::new(provider))
                            .stale_connection_timeout_secs(0)
                            .build(),
                    );
                }
            }
            ProviderConfig::BinanceSpotUser(c) => {
                if c.enabled {
                    let provider = BinanceSpotUserWsProvider::builder()
                        .api_key(c.api_key)
                        .api_secret(c.api_secret)
                        .http_url(c.http_url)
                        .ws_url(c.ws_url)
                        .persistence(persistence.clone())
                        .build();
                    clients.push(
                        WsClient::builder()
                            .provider(Box::new(provider))
                            .stale_connection_timeout_secs(0)
                            .build(),
                    );
                }
            }
        }
        clients
    }
}
