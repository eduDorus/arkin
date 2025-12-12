use anyhow::Result;
use async_trait::async_trait;

use arkin_core::prelude::*;
use reqwest::{header::HeaderMap, Request};

use crate::{
    errors::ProviderError,
    http::{HttpRequest, HttpRequestContext},
};

#[async_trait]
pub trait WebSocketProvider: Send + Sync {
    fn name(&self) -> &str;

    fn url(&self) -> &str;

    /// Build a subscription message for the given symbols and channels
    fn subscribe_msg(&self) -> Option<String>;

    /// Parse an incoming WebSocket message and return the corresponding data structure
    async fn parse(&self, msg: &str) -> Option<Event>;
}

#[async_trait]
pub trait HttpProvider: Send + Sync {
    fn get_endpoints(&self) -> Vec<HttpRequest>;

    fn build_request(&self, endpoint: &HttpRequestContext) -> Result<Request, ProviderError>;

    async fn parse(&self, headers: &HeaderMap, body: &str, channel: &Channel) -> Option<Event>;
}
