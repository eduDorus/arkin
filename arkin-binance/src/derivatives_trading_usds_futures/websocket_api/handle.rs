use crate::common::{config::ConfigurationWebsocketApi, models::WebsocketApiConnectConfig};

use super::WebsocketApi;

#[derive(Clone)]
pub struct WebsocketApiHandle {
    configuration: ConfigurationWebsocketApi,
}

impl WebsocketApiHandle {
    pub fn new(configuration: ConfigurationWebsocketApi) -> Self {
        Self { configuration }
    }

    /// Connects to the WebSocket API using default configuration.
    ///
    /// # Returns
    ///
    /// A `Result` containing the connected `WebsocketApi` instance if successful,
    /// or an error if the connection fails.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if the connection fails.
    ///
    pub async fn connect(&self) -> anyhow::Result<WebsocketApi> {
        self.connect_with_config(Default::default()).await
    }

    /// Connects to the WebSocket API with a custom configuration.
    ///
    /// # Arguments
    ///
    /// * `cfg` - A configuration object specifying connection parameters.
    ///
    /// # Returns
    ///
    /// A `Result` containing the connected `WebsocketApi` instance if successful,
    /// or an error if the connection fails.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if the connection fails.
    ///
    pub async fn connect_with_config(&self, cfg: WebsocketApiConnectConfig) -> anyhow::Result<WebsocketApi> {
        WebsocketApi::connect(self.configuration.clone(), cfg.mode).await
    }
}
