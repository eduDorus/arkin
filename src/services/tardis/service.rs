use anyhow::bail;
use anyhow::Error;
use anyhow::Result;
use bytes::Bytes;
use futures_util::stream;
use futures_util::Future;
use futures_util::Stream;
use futures_util::StreamExt;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::fmt::{self, Display};
use std::str::FromStr;
use time::macros::format_description;
use time::OffsetDateTime;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tracing::debug;
use tracing::error;

use crate::config::TardisConfig;
use crate::utils;

use super::http::TardisHttpClient;

#[derive(Debug, Clone)]
pub enum TardisExchange {
    BinanceSpot,
    BinanceSwaps,
    BinanceFutures,
    BinanceOptions,
    OkxFutures,
    OkxSwap,
    OkxSpot,
    OkxOptions,
}

impl Display for TardisExchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TardisExchange::BinanceSpot => "binance",
                TardisExchange::BinanceSwaps => "binance-futures",
                TardisExchange::BinanceFutures => "binance-delivery",
                TardisExchange::BinanceOptions => "binance-european-options",
                TardisExchange::OkxSpot => "okex-spot",
                TardisExchange::OkxSwap => "okex-swap",
                TardisExchange::OkxFutures => "okex-futures",
                TardisExchange::OkxOptions => "okex-options",
            }
        )
    }
}

impl FromStr for TardisExchange {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "binance-spot" => Ok(TardisExchange::BinanceSpot),
            "binance-swaps" => Ok(TardisExchange::BinanceSwaps),
            "binance-futures" => Ok(TardisExchange::BinanceFutures),
            "binance-options" => Ok(TardisExchange::BinanceOptions),
            "okex-spot" => Ok(TardisExchange::OkxSpot),
            "okex-swaps" => Ok(TardisExchange::OkxSwap),
            "okex-futures" => Ok(TardisExchange::OkxFutures),
            "okex-options" => Ok(TardisExchange::OkxOptions),
            _ => Err(Error::msg("invalid exchange")),
        }
    }
}

impl TardisExchange {
    pub fn channel_str(&self, channel: &TardisChannel) -> Result<String> {
        match self {
            TardisExchange::BinanceSwaps => match channel {
                TardisChannel::Book => Ok("depth".to_string()),
                TardisChannel::Trade => Ok("trade".to_string()),
                TardisChannel::AggTrade => Ok("aggTrade".to_string()),
                TardisChannel::Tick => Ok("ticker".to_string()),
                _ => bail!("Channel not supported for Binance exchange"),
            },
            TardisExchange::BinanceOptions => match channel {
                TardisChannel::Book => Ok("depth100".to_string()),
                TardisChannel::Trade => Ok("trade".to_string()),
                TardisChannel::Tick => Ok("ticker".to_string()),
                TardisChannel::OpenInterest => Ok("openInterest".to_string()),
                _ => bail!("Channel not supported for Binance exchange".to_string()),
            },
            TardisExchange::OkxSwap => match channel {
                TardisChannel::Book => Ok("books".to_string()),
                TardisChannel::Trade => Ok("trades-all".to_string()),
                TardisChannel::Tick => Ok("tickers".to_string()),
                TardisChannel::OpenInterest => Ok("open-interest".to_string()),
                TardisChannel::FundingRate => Ok("funding-rate".to_string()),
                _ => bail!("Channel not supported for Okex exchange"),
            },
            TardisExchange::OkxOptions => match channel {
                TardisChannel::Book => Ok("books".to_string()),
                TardisChannel::Trade => Ok("trades-all".to_string()),
                TardisChannel::Tick => Ok("opt-summary".to_string()),
                TardisChannel::OpenInterest => Ok("open-interest".to_string()),
                _ => bail!("Channel not supported for Okex exchange"),
            },
            _ => bail!("Exchange not supported yet"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TardisChannel {
    Book,
    Trade,
    AggTrade,
    Perpetual,
    Quote,
    Tick,
    Snapshot,
    OpenInterest,
    FundingRate,
}

impl FromStr for TardisChannel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "book" => Ok(TardisChannel::Book),
            "trade" => Ok(TardisChannel::Trade),
            "agg-trade" => Ok(TardisChannel::AggTrade),
            "perpetual" => Ok(TardisChannel::Perpetual),
            "quote" => Ok(TardisChannel::Quote),
            "tick" => Ok(TardisChannel::Tick),
            "snapshot" => Ok(TardisChannel::Snapshot),
            "open-interest" => Ok(TardisChannel::OpenInterest),
            "funding" => Ok(TardisChannel::FundingRate),
            _ => Err(Error::msg("invalid channel")),
        }
    }
}

pub struct TardisService {
    pub client: TardisHttpClient,
    pub max_concurrent_requests: usize,
}

impl TardisService {
    pub fn builder() -> TardisServiceBuilder {
        TardisServiceBuilder::default()
    }
}

#[derive(Default)]
pub struct TardisServiceBuilder {
    pub api_secret: Option<String>,
    pub base_url: String,
    pub max_concurrent_requests: usize,
}

#[allow(clippy::assigning_clones)]
impl TardisServiceBuilder {
    pub fn config(mut self, config: &TardisConfig) -> Self {
        self.api_secret = config.api_secret.to_owned();
        self.base_url = config.base_url.to_owned();
        self.max_concurrent_requests = config.max_concurrent_requests;
        self
    }

    pub fn api_secret(mut self, api_secret: Option<String>) -> Self {
        self.api_secret = api_secret;
        self
    }

    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn max_concurrent_requests(mut self, max_concurrent_requests: usize) -> Self {
        self.max_concurrent_requests = max_concurrent_requests;
        self
    }

    pub fn build(self) -> TardisService {
        let client = TardisHttpClient::builder()
            .base_url(self.base_url.to_owned())
            .api_secret(self.api_secret.to_owned())
            .build();

        TardisService {
            client,
            max_concurrent_requests: self.max_concurrent_requests,
        }
    }
}

pub struct TardisRequest {
    pub exchange: TardisExchange,
    pub channel: TardisChannel,
    pub instruments: Vec<String>,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

impl TardisService {
    pub fn download_stream(
        &self,
        req: TardisRequest,
    ) -> impl Stream<Item = impl Future<Output = Result<Vec<(OffsetDateTime, String)>>> + '_> + '_ {
        let dates = utils::datetime_range_minute(&req.start, &req.end).expect("Invalid date range");
        stream::iter(dates.into_iter().map(move |datetime| {
            let client = self.client.clone();
            let exchange_str = req.exchange.to_string();
            let channel_str = req.exchange.channel_str(&req.channel).unwrap();
            let instruments = req.instruments.clone();
            let offset = datetime.time().hour() as i64 * 60 + datetime.time().minute() as i64;

            async move {
                debug!("Downloading: {} Offset: {}", datetime, offset);
                let buffer: Bytes = client.request(exchange_str, channel_str, instruments, datetime, offset).await?;

                // Write response to buffer
                let mut reader = BufReader::new(buffer.as_ref());

                let mut values = Vec::new();
                let mut line = String::new();
                while reader.read_line(&mut line).await? > 0 {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let (ts, json) = parse_line(line.trim())?;
                    values.push((ts, json));
                    line.clear();
                }

                Ok(values)
            }
        }))
    }

    pub fn stream_parsed<T: DeserializeOwned + 'static>(
        &self,
        req: TardisRequest,
    ) -> impl Stream<Item = (OffsetDateTime, T)> + '_ {
        self.download_stream(req)
            .buffer_unordered(self.max_concurrent_requests)
            .filter_map(|result| async move {
                match result {
                    Ok(values) => Some(stream::iter(values)),
                    Err(e) => {
                        error!("Error: {}", e);
                        None
                    }
                }
            })
            .flat_map(|stream| stream)
            .filter_map(|(ts, data)| async move {
                let res = serde_json::from_str::<T>(&data);
                match res {
                    Ok(value) => Some((ts, value)),
                    Err(e) => {
                        error!("{:?}", e);
                        error!("Data: {}", data);
                        None
                    }
                }
            })
    }
}

fn parse_line(line: &str) -> Result<(OffsetDateTime, String)> {
    let mut parts = line.splitn(2, ' ');

    // Timestamp part
    let timestamp = parts.next().unwrap_or_default();
    let timestamp = timestamp.trim();
    let timestamp = timestamp.trim_end_matches(':');
    debug!("Timestamp: {}", &timestamp);

    // Json part
    let json = parts.next().unwrap_or_default();
    let json = json.trim();
    debug!("Json: {}", &json);

    let format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]Z");
    let Ok(ts) = time::PrimitiveDateTime::parse(timestamp, format) else {
        bail!("Invalid timestamp: {}", &timestamp);
    };
    let ts = ts.assume_utc();

    Ok((ts, json.to_string()))
}
