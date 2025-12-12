use std::{
    collections::{BTreeMap, HashMap},
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use crate::traits::HttpProvider;
use anyhow::Result;
use arkin_core::Channel;
use reqwest::{Client, Method};
use serde_json::Value;
use time::UtcDateTime;
use tokio::time::interval;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub backoff_duration: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_duration: Duration::from_millis(100),
        }
    }
}

#[allow(async_fn_in_trait)]
pub struct HttpExecutor<P: HttpProvider> {
    provider: P,
    client: Client,
    retry_config: RetryConfig,
}

impl<P: HttpProvider> HttpExecutor<P> {
    pub fn new(provider: P, retry_config: RetryConfig) -> Self {
        Self {
            provider,
            client: Client::builder()
                .pool_max_idle_per_host(5)
                .build()
                .expect("Failed to create reqwest client"),
            retry_config,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let requests = self.provider.get_endpoints();

        // Handle one-shot requests
        for request in &requests {
            if let RequestMode::OneShot = &request.mode {
                self.execute_with_retry(request).await?;
            }
        }

        // Handle range requests
        for request in &requests {
            if let RequestMode::Range { start, end } = &request.mode {
                let mut current = start.to_owned();
                let end = end.to_owned();
                while current < end {
                    let chunk_end = (current + Duration::from_secs(3600)).min(end); // Default 1-hour chunks; make configurable later
                                                                                    // Note: Assuming provider handles range params in context or modify here if needed
                    self.execute_with_retry(request).await?;
                    current = chunk_end;
                }
            }
        }

        // Handle polling requests in a loop
        let polling_requests: Vec<_> = requests
            .iter()
            .filter(|r| matches!(r.mode, RequestMode::Polling { .. }))
            .collect();
        if !polling_requests.is_empty() {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let now = UtcDateTime::now().unix_timestamp() as u64;
                for request in &polling_requests {
                    if let RequestMode::Polling { fetch_interval } = &request.mode {
                        let last_fetched = request.context.last_fetched.load(std::sync::atomic::Ordering::Relaxed);
                        if now - last_fetched >= fetch_interval.as_secs() {
                            self.execute_with_retry(request).await?;
                            request.context.last_fetched.store(now, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn execute_with_retry(&self, request: &HttpRequest) -> Result<()> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            let req = self.provider.build_request(&request.context)?;
            match self.client.execute(req).await {
                Ok(response) => {
                    let status = response.status();
                    debug!("Response status: {}", status);
                    let headers = response.headers().clone();
                    let body = response.text().await?;
                    if let Some(event) = self.provider.parse(&headers, &body, &request.context.channel).await {
                        info!("Sending event: {}", event);
                    }
                    return Ok(());
                }
                Err(e) if attempts < self.retry_config.max_attempts => {
                    error!(
                        "Attempt {} failed: {}. Retrying in {:?}",
                        attempts, e, self.retry_config.backoff_duration
                    );
                    tokio::time::sleep(self.retry_config.backoff_duration).await;
                }
                Err(e) => {
                    error!("Failed after {} attempts: {}", self.retry_config.max_attempts, e);
                    return Err(e.into());
                }
            }
        }
    }
}

pub struct HttpRequest {
    pub context: HttpRequestContext,
    pub mode: RequestMode,
}

impl HttpRequest {
    pub fn new_oneshot(context: HttpRequestContext) -> Self {
        Self {
            context,
            mode: RequestMode::OneShot,
        }
    }

    pub fn new_polling(context: HttpRequestContext, fetch_interval: Duration) -> Self {
        Self {
            context,
            mode: RequestMode::Polling { fetch_interval },
        }
    }

    pub fn new_range(context: HttpRequestContext, start: UtcDateTime, end: UtcDateTime) -> Self {
        Self {
            context,
            mode: RequestMode::Range { start, end },
        }
    }
}

pub enum RequestMode {
    OneShot,
    Polling {
        fetch_interval: Duration,
    },
    Range {
        start: UtcDateTime,
        end: UtcDateTime,
    },
}

pub struct HttpRequestContext {
    pub channel: Channel,
    pub method: Method,
    pub endpoint: String,
    pub params: BTreeMap<String, Value>,
    pub is_signed: bool,
    pub custom_headers: Option<HashMap<String, String>>,
    pub last_fetched: Arc<AtomicU64>,
}
