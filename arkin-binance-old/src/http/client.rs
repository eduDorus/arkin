use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::Client;
use tracing::debug;
use typed_builder::TypedBuilder;
use url::Url;

use crate::http::error::BinanceHttpClientError;

use super::{credentials::Credentials, method::Method, request::Request, response::Response, sign::sign_payload};

// Client Implementatino
#[derive(Debug, Clone, TypedBuilder)]
pub struct BinanceHttpClient {
    #[builder(default)]
    client: Client,
    #[builder(default = Url::parse("https://fapi.binance.com").expect("Default url for binance http client is invalid"))]
    base_url: Url,
    #[builder(default)]
    timestamp_delta: i64,
    credentials: Option<Credentials>,
}

impl BinanceHttpClient {
    pub fn new(client: Client, base_url: &str) -> Self {
        let url = Url::parse(base_url).expect("Invalid URL");
        Self {
            client,
            base_url: url,
            timestamp_delta: 0,
            credentials: None,
        }
    }

    pub fn with_url(base_url: &str) -> Self {
        let url = Url::parse(base_url).expect("Invalid URL");
        Self {
            client: Client::new(),
            base_url: url,
            timestamp_delta: 0,
            credentials: None,
        }
    }

    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    pub fn timestamp_delta(mut self, timestamp_delta: i64) -> Self {
        self.timestamp_delta = timestamp_delta;
        self
    }
}

impl BinanceHttpClient {
    pub async fn send<R: Into<Request>>(&self, request: R) -> Result<Response, BinanceHttpClientError> {
        let Request {
            method,
            path,
            params,
            credentials,
            sign,
        } = request.into();

        // Build URL
        let url: Url = format!("{}{}", self.base_url, path).parse()?;

        let mut req_builder = self.client.request(method.into(), &url.to_string());

        // Set User-Agent in header
        let user_agent = &format!("trading-agent/1.0.0");
        req_builder = req_builder.header("User-Agent", user_agent);

        // Map query parameters
        let mut query_params = vec![];
        let has_params = !params.is_empty();
        if has_params {
            query_params.extend_from_slice(&params);
        }

        let client_credentials = self.credentials.as_ref();
        let request_credentials = credentials.as_ref();
        if let Some(Credentials { api_key, signature }) = request_credentials.or(client_credentials) {
            // Set API-Key in header
            req_builder = req_builder.header("X-MBX-APIKEY", api_key);

            if sign {
                // Use system clock, panic if system clock is behind `std::time::UNIX_EPOCH`
                let mut timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Clock may have gone backwards")
                    .as_millis();

                // Append timestamp delta to sync up with server time.
                timestamp -= self.timestamp_delta as u128;

                // Append timestamp to query parameters
                query_params.push(("timestamp".to_string(), timestamp.to_string()));

                // Stringfy available query parameters and append back to query parameters
                let query_params_str = query_params
                    .iter()
                    .map(|(key, value)| format!("{}={}", key, value))
                    .collect::<Vec<String>>()
                    .join("&");
                let signature =
                    sign_payload(&query_params_str, signature).map_err(|_| BinanceHttpClientError::InvalidApiSecret)?;

                query_params.push(("signature".to_string(), signature));
            }
        }

        // Add query params
        req_builder = req_builder.query(&query_params);

        // Build request
        let req = req_builder.build()?;
        debug!("BinanceHttpClient request: {:?}", req);

        // Send request
        let response = match self.client.execute(req).await {
            Ok(response) => Ok(response),
            Err(err) => Err(BinanceHttpClientError::Send(err)),
        }?;

        debug!("{}", response.status());
        debug!("{:?}", response.headers());

        let body = response.text().await?;
        Ok(Response { body })
    }
}

impl Default for BinanceHttpClient {
    fn default() -> Self {
        Self::new(Client::new(), "https://testnet.binancefuture.com/")
    }
}

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> reqwest::Method {
        match method {
            Method::Post => reqwest::Method::POST,
            Method::Delete => reqwest::Method::DELETE,
            Method::Get => reqwest::Method::GET,
            Method::Put => reqwest::Method::PUT,
        }
    }
}
