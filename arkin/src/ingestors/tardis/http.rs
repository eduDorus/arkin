use bytes::Bytes;

use anyhow::Result;
use backoff::ExponentialBackoff;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::Serialize;
use time::OffsetDateTime;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct TardisHttpClient {
    pub base_url: String,
    pub client: Client,
}

impl TardisHttpClient {
    pub fn builder() -> TardisHttpClientBuilder {
        TardisHttpClientBuilder::default()
    }

    pub async fn request(
        &self,
        exchange: String,
        channel: String,
        symbols: Vec<String>,
        date: OffsetDateTime,
        offset: i64,
    ) -> Result<Bytes> {
        let url = format!("{}/{}", self.base_url, exchange);
        let query = QueryParams::new(channel, symbols, date, offset);
        let res = backoff::future::retry(ExponentialBackoff::default(), || async {
            let req = self.client.get(&url).query(&query.to_query()).build()?;
            debug!("URL: {:?}", req.url().to_string());
            debug!("Request: {:?}", req);
            let res = self.client.execute(req).await.unwrap();
            debug!("Response: {:?}", res);
            let data = res.bytes().await?;
            Ok(data)
        })
        .await?;
        Ok(res)
    }
}

#[derive(Default)]
pub struct TardisHttpClientBuilder {
    pub base_url: String,
    pub api_secret: Option<String>,
}

impl TardisHttpClientBuilder {
    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn api_secret(mut self, api_secret: Option<String>) -> Self {
        self.api_secret = api_secret;
        self
    }

    pub fn build(self) -> TardisHttpClient {
        let client = get_client(&self.api_secret).expect("Failed to create tardis http client");
        TardisHttpClient {
            base_url: self.base_url,
            client,
        }
    }
}

pub fn get_client(api_secret: &Option<String>) -> Result<Client> {
    // Set api bearer token if provided
    let headers = create_headers(api_secret)?;
    let client = Client::builder().default_headers(headers).build()?;
    Ok(client)
}

fn create_headers(api_secret: &Option<String>) -> anyhow::Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_str("application/json")?);
    if let Some(api_key) = api_secret {
        let mut bearer = "Bearer ".to_owned();
        bearer.push_str(api_key);
        headers.insert("Authorization", HeaderValue::from_str(&bearer)?);
    }
    Ok(headers)
}

#[derive(Debug, Clone, Serialize)]
pub struct Filter {
    channel: String,
    symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryParams {
    from: String,
    offset: i64,
    filters: Vec<Filter>,
}

impl QueryParams {
    pub fn new(channel: String, symbols: Vec<String>, date: OffsetDateTime, offset: i64) -> Self {
        Self {
            from: date.date().to_string(),
            offset,
            filters: vec![Filter {
                channel: channel.to_string(),
                symbols: symbols.into_iter().map(|s| s.to_lowercase()).collect(),
            }],
        }
    }

    pub fn to_query(&self) -> [(String, String); 3] {
        [
            ("from".to_string(), self.from.to_string()),
            ("offset".to_string(), self.offset.to_string()),
            ("filters".to_string(), serde_json::to_string(&self.filters).unwrap()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use time::macros::datetime;

    use super::*;

    #[tokio::test]
    async fn test_header_with_api_key() {
        let api_secret = Some("test_secret".to_string());
        let header = create_headers(&api_secret).unwrap();
        assert_eq!(header.get("Content-Type").unwrap(), "application/json");
        assert_eq!(header.get("Authorization").unwrap(), "Bearer test_secret")
    }

    #[tokio::test]
    async fn test_header_without_api_key() {
        let api_secret = None;
        let header = create_headers(&api_secret).unwrap();
        assert_eq!(header.get("Content-Type").unwrap(), "application/json");
        assert_eq!(header.get("Authorization"), None)
    }

    #[tokio::test]
    async fn test_query() {
        let channel = "trades".to_string();
        let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
        let date = datetime!(2021 - 01 - 01 01:00:00).assume_utc();
        let offset = 1;

        let params = QueryParams::new(channel, symbols, date, offset);
        let query = params.to_query();
        assert_eq!(query[0].0, "from");
        assert_eq!(query[0].1, date.date().to_string());
        assert_eq!(query[1].0, "offset");
        assert_eq!(query[1].1, date.hour().to_string().as_str());
        assert_eq!(query[2].0, "filters");
        assert_eq!(query[2].1, r#"[{"channel":"trades","symbols":["btcusdt","ethusdt"]}]"#);
    }
}
