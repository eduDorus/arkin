use std::time::Duration;

use bytes::Bytes;

use anyhow::Result;
use backoff::ExponentialBackoff;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
    Client,
};
use serde::Serialize;
use time::UtcDateTime;
use tracing::{debug, error};
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, TypedBuilder)]
pub struct TardisHttpClient {
    pub base_url: String,
    pub api_secret: Option<String>,
    pub client: Client,
}

impl TardisHttpClient {
    pub fn new(base_url: String, api_secret: Option<String>) -> Self {
        let client = Client::builder()
            .read_timeout(Duration::from_secs(120))
            .timeout(Duration::from_secs(180))
            .connect_timeout(Duration::from_secs(10))
            .gzip(true)
            // .zstd(true)
            // .brotli(true)
            // .deflate(true)
            .build()
            .expect("Could not initialize tardis http client");

        Self {
            base_url: base_url.to_owned(),
            api_secret,
            client,
        }
    }

    pub async fn request(
        &self,
        exchange: String,
        channel: String,
        symbols: Vec<String>,
        date: UtcDateTime,
        offset: i64,
    ) -> Result<Bytes> {
        let url = format!("{}/{}", self.base_url, exchange);
        let query = QueryParams::new(channel, symbols, date, offset);
        let res = backoff::future::retry(ExponentialBackoff::default(), || async {
            let headers = create_headers(&self.api_secret).expect("Failed to create headers");
            let req = self.client.get(&url).query(&query.to_query()).headers(headers).build()?;
            debug!("URL: {:?}", req.url().to_string());
            debug!("Request: {:?}", req);
            debug!("Request header: {:?}", req.headers());
            let res = self.client.execute(req).await?;
            debug!("Response: {:?}", res);
            debug!("Response header: {:?}", res.headers());
            debug!("Response version: {:?}", res.version());
            match res.error_for_status() {
                Ok(res) => {
                    let data = res.bytes().await?;
                    return Ok(data);
                }
                Err(e) => {
                    error!("Failed to fetch data: {}", e);
                    return Err(backoff::Error::transient(e));
                }
            }
        })
        .await?;
        Ok(res)
    }
}

fn create_headers(api_secret: &Option<String>) -> anyhow::Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    // Set content type to json
    headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);

    // Set user agent
    headers.insert(USER_AGENT, HeaderValue::from_str("nginx/1.18.0")?);

    // Set the Authorization header if api_secret is provided
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
    pub fn new(channel: String, symbols: Vec<String>, date: UtcDateTime, offset: i64) -> Self {
        Self {
            from: date.date().to_string(),
            offset,
            filters: vec![Filter {
                channel: channel.to_string(),
                symbols: symbols.into_iter().map(|s| s).collect(),
            }],
        }
    }

    pub fn to_query(&self) -> [(String, String); 3] {
        [
            ("from".to_string(), self.from.to_string()),
            ("filters".to_string(), serde_json::to_string(&self.filters).unwrap()),
            ("offset".to_string(), self.offset.to_string()),
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
        let symbols = vec!["btcusdt".to_string(), "ethusdt".to_string()];
        let date = datetime!(2021 - 01 - 01 01:00:00 UTC).to_utc();
        let offset = 1;

        let params = QueryParams::new(channel, symbols, date, offset);
        let query = params.to_query();
        assert_eq!(query[0].0, "from");
        assert_eq!(query[0].1, date.date().to_string());
        assert_eq!(query[2].0, "offset");
        assert_eq!(query[2].1, date.hour().to_string().as_str());
        assert_eq!(query[1].0, "filters");
        assert_eq!(query[1].1, r#"[{"channel":"trades","symbols":["btcusdt","ethusdt"]}]"#);
    }
}
