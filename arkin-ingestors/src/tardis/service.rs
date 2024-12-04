use std::fmt;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{bail, Error, Result};
use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{stream, Stream, StreamExt};
use serde::de::DeserializeOwned;
use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::pin;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use crate::config::TardisIngestorConfig;
use crate::traits::Ingestor;
use crate::IngestorError;

use super::binance_swap::BinanceSwapsEvent;
use super::http::TardisHttpClient;

#[derive(Debug, Clone)]
pub enum TardisExchange {
    BinanceSpot,
    BinanceUSDM,
    BinanceCOINM,
    BinanceOptions,
    OkxFutures,
    OkxSwap,
    OkxSpot,
    OkxOptions,
}

impl fmt::Display for TardisExchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TardisExchange::BinanceSpot => "binance",
                TardisExchange::BinanceUSDM => "binance-futures",
                TardisExchange::BinanceCOINM => "binance-delivery",
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
            "binance-usdm" => Ok(TardisExchange::BinanceUSDM),
            "binance-coinm" => Ok(TardisExchange::BinanceCOINM),
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
            TardisExchange::BinanceUSDM => match channel {
                TardisChannel::Book => Ok("depth".to_string()),
                TardisChannel::Trade => Ok("trade".to_string()),
                TardisChannel::AggTrade => Ok("aggTrade".to_string()),
                TardisChannel::Tick => Ok("bookTicker".to_string()),
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

impl Default for TardisExchange {
    fn default() -> Self {
        TardisExchange::BinanceUSDM // Set default to VariantA
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

impl Default for TardisChannel {
    fn default() -> Self {
        TardisChannel::AggTrade // Set default to VariantA
    }
}

impl fmt::Display for TardisChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TardisChannel::Book => "book",
                TardisChannel::Trade => "trades",
                TardisChannel::AggTrade => "agg-trades",
                TardisChannel::Perpetual => "perpetual",
                TardisChannel::Quote => "quotes",
                TardisChannel::Tick => "ticks",
                TardisChannel::Snapshot => "snapshots",
                TardisChannel::OpenInterest => "open-interest",
                TardisChannel::FundingRate => "funding",
            }
        )
    }
}

impl FromStr for TardisChannel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "book" => Ok(TardisChannel::Book),
            "trades" => Ok(TardisChannel::Trade),
            "agg-trades" => Ok(TardisChannel::AggTrade),
            "perpetual" => Ok(TardisChannel::Perpetual),
            "quotes" => Ok(TardisChannel::Quote),
            "ticks" => Ok(TardisChannel::Tick),
            "snapshots" => Ok(TardisChannel::Snapshot),
            "open-interest" => Ok(TardisChannel::OpenInterest),
            "funding" => Ok(TardisChannel::FundingRate),
            _ => Err(Error::msg("invalid channel")),
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

impl TardisRequest {
    pub fn new(
        exchange: TardisExchange,
        channel: TardisChannel,
        instruments: Vec<String>,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Self {
        TardisRequest {
            exchange: exchange.to_owned(),
            channel: channel.to_owned(),
            instruments: instruments.to_owned(),
            start: start.to_owned(),
            end: end.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct TardisIngestor {
    pub pubsub: Arc<PubSub>,
    pub persistence_service: Arc<PersistenceService>,
    pub client: TardisHttpClient,
    pub max_concurrent_requests: usize,
    pub venue: TardisExchange,
    pub channel: TardisChannel,
    pub instruments: Vec<String>,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

impl TardisIngestor {
    pub fn from_config(
        config: &TardisIngestorConfig,
        pubsub: Arc<PubSub>,
        persistence_service: Arc<PersistenceService>,
    ) -> Self {
        let format = format_description!("[year]-[month]-[day] [hour]:[minute]");
        let start = PrimitiveDateTime::parse(&config.start, &format)
            .expect("Failed to parse start date")
            .assume_utc();
        let end = PrimitiveDateTime::parse(&config.end, &format)
            .expect("Failed to parse end date")
            .assume_utc();

        let start_fmt = start.format(TIMESTAMP_FORMAT).unwrap();
        let end_fmt = end.format(TIMESTAMP_FORMAT).unwrap();

        info!("Starting: {} Ending: {}", start_fmt, end_fmt);
        Self {
            pubsub,
            persistence_service,
            client: TardisHttpClient::builder()
                .api_secret(config.api_secret.clone())
                .base_url(config.http_url.clone())
                .build(),
            max_concurrent_requests: config.max_concurrent_requests,
            venue: TardisExchange::from_str(&config.venue).expect("Invalid venue for tardis"),
            channel: TardisChannel::from_str(&config.channel).expect("Invalid channel for tardis"),
            instruments: config.instruments.to_owned(),
            start,
            end,
        }
    }

    pub fn download_stream(
        &self,
        req: TardisRequest,
    ) -> impl Stream<Item = impl Future<Output = Result<Vec<(OffsetDateTime, String)>>> + '_> + '_ {
        let dates = datetime_range_minute(req.start, req.end).expect("Invalid date range");
        stream::iter(dates.into_iter().map(move |datetime| {
            let client = self.client.clone();
            let exchange_str = req.exchange.to_string();
            let channel_str = req.exchange.channel_str(&req.channel).unwrap();
            let instruments = req.instruments.clone();
            let offset = datetime.time().hour() as i64 * 60 + datetime.time().minute() as i64;

            async move {
                info!("Tardis downloading {} from {}", channel_str, datetime);
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

    pub fn stream(&self, req: TardisRequest) -> impl Stream<Item = (OffsetDateTime, String)> + '_ {
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
        bail!("Invalid timestamp: {} in line: {}", &timestamp, line);
    };
    let ts = ts.assume_utc();

    Ok((ts, json.to_string()))
}

#[async_trait]
impl Ingestor for TardisIngestor {
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), IngestorError> {
        info!("Starting tardis ingestor...");
        let persistence_service = Arc::clone(&self.persistence_service);

        let req = TardisRequest::new(
            self.venue.clone(),
            self.channel.clone(),
            self.instruments.clone(),
            self.start,
            self.end,
        );

        let stream = self.stream(req);
        pin!(stream);

        // No need to clone persistence_service for each iteration
        while let Some((_ts, json)) = stream.next().await {
            let event = match serde_json::from_str::<BinanceSwapsEvent>(&json) {
                Ok(e) => Some(e),
                Err(e) => {
                    error!("Failed to parse Binance event: {}", e);
                    error!("Data: {}", json);
                    None
                }
            };

            let event = match event {
                Some(e) => {
                    debug!("{}", e);
                    e
                }
                None => continue,
            };

            let instrument = persistence_service
                .instrument_store
                .read_by_venue_symbol(&event.venue_symbol())
                .await?;

            match event {
                BinanceSwapsEvent::AggTradeStream(stream) => {
                    let trade = stream.data;
                    let side = if trade.maker {
                        MarketSide::Sell
                    } else {
                        MarketSide::Buy
                    };
                    let trade = Trade::new(
                        trade.event_time,
                        instrument,
                        trade.agg_trade_id,
                        side,
                        trade.price,
                        trade.quantity,
                    );
                    let trade = Arc::new(trade);
                    self.pubsub.publish::<Trade>(trade);
                }
                BinanceSwapsEvent::TickStream(stream) => {
                    let tick = stream.data;
                    let tick = Tick::new(
                        tick.event_time,
                        instrument,
                        tick.update_id,
                        tick.bid_price,
                        tick.bid_quantity,
                        tick.ask_price,
                        tick.ask_quantity,
                    );
                    let tick = Arc::new(tick);
                    self.pubsub.publish::<Tick>(tick);
                }
            }
        }
        Ok(())
    }
}
