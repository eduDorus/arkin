use crate::common::{config::ConfigurationWebsocketStreams, models::WebsocketStreamsConnectConfig};

use super::WebsocketStreams;

#[derive(Clone)]
pub struct WebsocketStreamsHandle {
    configuration: ConfigurationWebsocketStreams,
}

impl WebsocketStreamsHandle {
    #[must_use]
    pub fn new(configuration: ConfigurationWebsocketStreams) -> Self {
        Self { configuration }
    }

    /// Connects to a WebSocket stream using default configuration.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `WebsocketStreams` instance if the connection is successful,
    /// or an error if the connection fails.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if the connection fails.
    ///
    /// # Examples
    ///
    ///
    /// let handle = `WebsocketStreamsHandle::new(configuration)`;
    /// let streams = handle.connect().await?;
    ///
    pub async fn connect(&self) -> anyhow::Result<WebsocketStreams> {
        self.connect_with_config(Default::default()).await
    }

    /// Connects to a WebSocket stream with a custom configuration.
    ///
    /// # Arguments
    ///
    /// * `cfg` - A configuration object specifying connection details for the WebSocket stream.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `WebsocketStreams` instance if the connection is successful,
    /// or an error if the connection fails.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if the connection fails.
    ///
    /// # Examples
    ///
    ///
    /// let handle = `WebsocketStreamsHandle::new(configuration)`;
    /// let `custom_config` = `WebsocketStreamsConnectConfig::default()`;
    /// let streams = `handle.connect_with_config(custom_config).await`?;
    ///
    pub async fn connect_with_config(&self, cfg: WebsocketStreamsConnectConfig) -> anyhow::Result<WebsocketStreams> {
        WebsocketStreams::connect(self.configuration.clone(), cfg.streams, cfg.mode).await
    }
}
