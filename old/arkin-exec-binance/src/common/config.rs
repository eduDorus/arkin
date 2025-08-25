use reqwest::{Client, ClientBuilder};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio_tungstenite::Connector;
use typed_builder::TypedBuilder;

use super::models::{TimeUnit, WebsocketMode};
use super::utils::SignatureGenerator;

#[derive(Clone)]
pub struct AgentConnector(pub Connector);

impl fmt::Debug for AgentConnector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Connector(…)")
    }
}

#[derive(Clone)]
pub struct HttpAgent(pub Arc<dyn Fn(ClientBuilder) -> ClientBuilder + Send + Sync>);

impl fmt::Debug for HttpAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HttpAgent(<custom agent fn>)")
    }
}

#[derive(Debug, Clone)]
pub struct ProxyAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub host: String,
    pub port: u16,
    pub protocol: Option<String>,
    pub auth: Option<ProxyAuth>,
}

#[derive(Debug, Clone)]
pub enum PrivateKey {
    File(String),
    Raw(Vec<u8>),
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ConfigurationRestApi {
    #[builder(setter(into, strip_option), default)]
    pub api_key: Option<String>,

    #[builder(setter(into, strip_option), default)]
    pub api_secret: Option<String>,

    #[builder(setter(into, strip_option), default)]
    pub base_path: Option<String>,

    #[builder(default = 1000)]
    pub timeout: u64,

    #[builder(default = true)]
    pub keep_alive: bool,

    #[builder(default = true)]
    pub compression: bool,

    #[builder(default = 3)]
    pub retries: u32,

    #[builder(default = 1000)]
    pub backoff: u64,

    #[builder(setter(strip_option), default)]
    pub proxy: Option<ProxyConfig>,

    #[builder(setter(strip_option, into), default)]
    pub custom_headers: Option<HashMap<String, String>>,

    #[builder(setter(strip_option), default)]
    pub agent: Option<HttpAgent>,

    #[builder(setter(strip_option), default)]
    pub private_key: Option<PrivateKey>,

    #[builder(setter(strip_option), default)]
    pub private_key_passphrase: Option<String>,

    #[builder(setter(strip_option), default)]
    pub time_unit: Option<TimeUnit>,

    #[builder(default, setter(skip))]
    pub(crate) client: Client,

    #[builder(default, setter(skip))]
    pub(crate) user_agent: String,

    #[builder(default, setter(skip))]
    pub(crate) signature_gen: SignatureGenerator,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ConfigurationWebsocketApi {
    #[builder(setter(into, strip_option), default)]
    pub api_key: Option<String>,

    #[builder(setter(into, strip_option), default)]
    pub api_secret: Option<String>,

    #[builder(setter(into, strip_option), default)]
    pub ws_url: Option<String>,

    #[builder(default = 5000)]
    pub timeout: u64,

    #[builder(default = 1000)]
    pub reconnect_delay: u64,

    #[builder(default = WebsocketMode::Single)]
    pub mode: WebsocketMode,

    #[builder(setter(strip_option), default)]
    pub agent: Option<AgentConnector>,

    #[builder(setter(strip_option), default)]
    pub private_key: Option<PrivateKey>,

    #[builder(setter(strip_option), default)]
    pub private_key_passphrase: Option<String>,

    #[builder(setter(strip_option), default)]
    pub time_unit: Option<TimeUnit>,

    #[builder(default = true)]
    pub auto_session_relogon: bool,

    #[builder(default, setter(skip))]
    pub(crate) user_agent: String,

    #[builder(default, setter(skip))]
    pub(crate) signature_gen: SignatureGenerator,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ConfigurationWebsocketStreams {
    #[builder(setter(into, strip_option), default)]
    pub ws_url: Option<String>,

    #[builder(default = 1000)]
    pub reconnect_delay: u64,

    #[builder(default = WebsocketMode::Single)]
    pub mode: WebsocketMode,

    #[builder(setter(strip_option), default)]
    pub agent: Option<AgentConnector>,

    #[builder(setter(strip_option), default)]
    pub time_unit: Option<TimeUnit>,

    #[builder(default, setter(skip))]
    pub(crate) user_agent: String,
}
