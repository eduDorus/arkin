use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use arkin_core::prelude::*;
use async_trait::async_trait;
use bytes::Bytes;
use futures::{stream, Stream, StreamExt};
use rust_decimal::prelude::*;
use serde::de::DeserializeOwned;
use time::macros::format_description;
use time::UtcDateTime;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::pin;
use tracing::{debug, error, info, warn};
use typed_builder::TypedBuilder;

use crate::mapping;

use super::http::TardisHttpClient;

pub fn parse_stream_event(json: &str, venue: &VenueName, channel: &Channel) -> Result<ExchangeStreamEvent> {
    match venue {
        VenueName::BinanceSpot => parse_binance_spot_event(json, venue, channel),
        VenueName::BinanceUsdmFutures | VenueName::BinanceCoinmFutures | VenueName::BinanceOptions => {
            parse_binance_futures_event(json, venue, channel)
        }
        VenueName::BybitSpot | VenueName::BybitDerivatives | VenueName::BybitOptions => {
            parse_bybit_event(json, venue, channel)
        }
        VenueName::OkxSpot | VenueName::OkxSwap | VenueName::OkxFutures | VenueName::OkxOptions => {
            parse_okx_event(json, venue, channel)
        }
        _ => Err(anyhow!("Exchange {:?} not yet implemented", venue)),
    }
}

fn parse_binance_spot_event(json: &str, venue: &VenueName, channel: &Channel) -> Result<ExchangeStreamEvent> {
    match channel {
        Channel::AggTrades => {
            // Binance spot uses the same format as futures for agg trades
            let stream_event: BinanceSwapsStreamEvent =
                serde_json::from_str(json).map_err(|e| anyhow!("Binance spot agg trade parse error: {}", e))?;

            let instrument = stream_event.data.venue_symbol().to_string();
            let timestamp = stream_event.data.event_time();

            Ok(ExchangeStreamEvent {
                venue: *venue,
                channel: *channel,
                instrument,
                timestamp,
                data: stream_event.data.to_unified(),
            })
        }
        Channel::Trades => {
            // Binance spot trades have a different format - parse manually
            // Try parsing with stream field first, then fallback to direct data parsing
            let trade_data = if let Ok(stream_event) = serde_json::from_str::<serde_json::Value>(json) {
                if let Some(data) = stream_event.get("data") {
                    serde_json::from_value(data.clone())
                        .map_err(|e| anyhow!("Binance spot trade data parse error: {}", e))?
                } else {
                    // Direct data format without stream wrapper
                    serde_json::from_value(stream_event)
                        .map_err(|e| anyhow!("Binance spot trade direct parse error: {}", e))?
                }
            } else {
                return Err(anyhow!("Binance spot trade JSON parse error"));
            };

            #[derive(serde::Deserialize)]
            struct SpotTradeData {
                #[serde(rename = "E", with = "custom_serde::timestamp")]
                event_time: time::UtcDateTime,
                #[serde(rename = "T", with = "custom_serde::timestamp")]
                transaction_time: time::UtcDateTime,
                #[serde(rename = "s")]
                instrument: String,
                #[serde(rename = "t")]
                trade_id: u64,
                #[serde(rename = "p")]
                price: rust_decimal::Decimal,
                #[serde(rename = "q")]
                quantity: rust_decimal::Decimal,
                #[serde(rename = "m")]
                maker: bool,
            }

            let trade_data: SpotTradeData = trade_data;

            let instrument = trade_data.instrument.clone();
            let timestamp = trade_data.event_time;

            // Convert to unified TradeData
            let trade_data_unified = TradeData {
                event_time: trade_data.event_time,
                transaction_time: trade_data.transaction_time,
                trade_id: trade_data.trade_id.to_string(),
                price: trade_data.price,
                quantity: trade_data.quantity,
                side: if trade_data.maker {
                    MarketSide::Sell
                } else {
                    MarketSide::Buy
                },
                maker: trade_data.maker,
            };

            Ok(ExchangeStreamEvent {
                venue: *venue,
                channel: *channel,
                instrument,
                timestamp,
                data: ExchangeEventData::Trade(trade_data_unified),
            })
        }
        Channel::Ticker => {
            // Binance spot book ticker has a different format - no E/T timestamp fields
            // Try parsing with stream field first, then fallback to direct data parsing
            let ticker_data = if let Ok(stream_event) = serde_json::from_str::<serde_json::Value>(json) {
                if let Some(data) = stream_event.get("data") {
                    serde_json::from_value(data.clone())
                        .map_err(|e| anyhow!("Binance spot ticker data parse error: {}", e))?
                } else {
                    // Direct data format without stream wrapper
                    serde_json::from_value(stream_event)
                        .map_err(|e| anyhow!("Binance spot ticker direct parse error: {}", e))?
                }
            } else {
                return Err(anyhow!("Binance spot ticker JSON parse error"));
            };

            #[derive(serde::Deserialize)]
            struct BinanceSpotBookTickerData {
                #[serde(rename = "u")]
                update_id: u64,
                #[serde(rename = "s")]
                instrument: String,
                #[serde(rename = "b")]
                bid_price: rust_decimal::Decimal,
                #[serde(rename = "B")]
                bid_quantity: rust_decimal::Decimal,
                #[serde(rename = "a")]
                ask_price: rust_decimal::Decimal,
                #[serde(rename = "A")]
                ask_quantity: rust_decimal::Decimal,
            }

            let ticker_data: BinanceSpotBookTickerData = ticker_data;

            let instrument = ticker_data.instrument.clone();
            // For spot tickers, we don't have event time, so use current time as approximation
            let timestamp = time::UtcDateTime::now();

            // Convert to unified TickData
            let tick_data = TickData {
                event_time: timestamp,
                transaction_time: timestamp,
                update_id: ticker_data.update_id,
                bid_price: ticker_data.bid_price,
                bid_quantity: ticker_data.bid_quantity,
                ask_price: ticker_data.ask_price,
                ask_quantity: ticker_data.ask_quantity,
            };

            Ok(ExchangeStreamEvent {
                venue: *venue,
                channel: *channel,
                instrument,
                timestamp,
                data: ExchangeEventData::Tick(tick_data),
            })
        }
        Channel::OrderBook | Channel::LongShortRatio | Channel::OpenInterest => {
            // For other channels, use the futures format for now
            let stream_event: BinanceSwapsStreamEvent =
                serde_json::from_str(json).map_err(|e| anyhow!("Binance parse error: {}", e))?;

            let instrument = stream_event.data.venue_symbol().to_string();
            let timestamp = stream_event.data.event_time();

            Ok(ExchangeStreamEvent {
                venue: *venue,
                channel: *channel,
                instrument,
                timestamp,
                data: stream_event.data.to_unified(),
            })
        }
        _ => Err(anyhow!("Unsupported Binance spot channel: {:?}", channel)),
    }
}

fn parse_binance_futures_event(json: &str, venue: &VenueName, channel: &Channel) -> Result<ExchangeStreamEvent> {
    match channel {
        Channel::AggTrades
        | Channel::Trades
        | Channel::OrderBook
        | Channel::LongShortRatio
        | Channel::Ticker
        | Channel::OpenInterest => {
            let stream_event: BinanceSwapsStreamEvent =
                serde_json::from_str(json).map_err(|e| anyhow!("Binance parse error: {}", e))?;

            let instrument = stream_event.data.venue_symbol().to_string();
            let timestamp = stream_event.data.event_time();

            Ok(ExchangeStreamEvent {
                venue: *venue,
                channel: *channel,
                instrument,
                timestamp,
                data: stream_event.data.to_unified(),
            })
        }
        _ => Err(anyhow!("Unsupported Binance channel: {:?}", channel)),
    }
}

fn parse_bybit_event(json: &str, venue: &VenueName, channel: &Channel) -> Result<ExchangeStreamEvent> {
    match channel {
        Channel::Trades => {
            let root: BybitTradeMessage =
                serde_json::from_str(json).map_err(|e| anyhow!("Bybit trade parse error: {}", e))?;

            // For now, assume first trade, need to handle properly
            let trade = &root.data[0];
            let timestamp = trade.transaction_time;
            let instrument = trade.instrument.clone();

            // Use unified conversion method
            let data = trade.clone().to_unified();

            Ok(ExchangeStreamEvent {
                venue: *venue,
                channel: *channel,
                instrument,
                timestamp,
                data,
            })
        }
        Channel::Ticker => {
            // Bybit spot and derivatives use orderbook.1 channel for ticker data (has bid/ask prices)
            let root: BybitOrderbookMessage =
                serde_json::from_str(json).map_err(|e| anyhow!("Bybit orderbook parse error: {}", e))?;

            // Only process snapshot messages, skip delta messages for ticker data
            if root.type_field != "snapshot" {
                return Err(anyhow!(
                    "Skipping Bybit orderbook delta message (type: {}), only processing snapshots for ticker data",
                    root.type_field
                ));
            }

            // Skip messages that don't provide complete bid/ask data
            if root.data.bids.is_empty() || root.data.asks.is_empty() {
                return Err(anyhow!(
                    "Skipping Bybit orderbook snapshot with incomplete data: bids.len={}, asks.len={}",
                    root.data.bids.len(),
                    root.data.asks.len()
                ));
            }

            let timestamp =
                time::UtcDateTime::from_unix_timestamp(root.ts / 1000).unwrap_or(time::UtcDateTime::UNIX_EPOCH);
            let instrument = root.data.symbol.clone();

            // Extract best bid and ask from orderbook data
            let bid_price = root
                .data
                .bids
                .first()
                .and_then(|bid| Decimal::from_str(&bid[0]).ok())
                .unwrap_or(Decimal::ZERO);
            let bid_quantity = root
                .data
                .bids
                .first()
                .and_then(|bid| Decimal::from_str(&bid[1]).ok())
                .unwrap_or(Decimal::ZERO);
            let ask_price = root
                .data
                .asks
                .first()
                .and_then(|ask| Decimal::from_str(&ask[0]).ok())
                .unwrap_or(Decimal::ZERO);
            let ask_quantity = root
                .data
                .asks
                .first()
                .and_then(|ask| Decimal::from_str(&ask[1]).ok())
                .unwrap_or(Decimal::ZERO);

            debug!(
                "Bybit orderbook parsing: type={}, bids.len={}, asks.len={}, bid_price={}, ask_price={}",
                root.type_field,
                root.data.bids.len(),
                root.data.asks.len(),
                bid_price,
                ask_price
            );

            // Create tick data
            let tick_data = TickData {
                event_time: timestamp,
                transaction_time: timestamp,
                update_id: root.data.update_id as u64,
                bid_price,
                bid_quantity,
                ask_price,
                ask_quantity,
            };

            Ok(ExchangeStreamEvent {
                venue: *venue,
                channel: *channel,
                instrument,
                timestamp,
                data: ExchangeEventData::Tick(tick_data),
            })
        }
        Channel::OrderBook
        | Channel::AggTrades
        | Channel::OpenInterest
        | Channel::FundingRate
        | Channel::LongShortRatio => Err(anyhow!("Channel {:?} not yet implemented for Bybit", channel)),
        _ => Err(anyhow!("Unsupported Bybit channel: {:?}", channel)),
    }
}

fn parse_okx_event(json: &str, venue: &VenueName, channel: &Channel) -> Result<ExchangeStreamEvent> {
    // Get the expected channel string from our mapping
    let expected_channel_str = mapping::get_tardis_channel_str(*venue, *channel)
        .map_err(|e| anyhow!("Failed to get OKX channel mapping: {}", e))?;

    // Parse the arg to validate the channel type matches what we expect
    let arg_value: serde_json::Value = serde_json::from_str(json).map_err(|e| anyhow!("OKX arg parse error: {}", e))?;
    let actual_channel_name = arg_value
        .get("arg")
        .and_then(|arg| arg.get("channel"))
        .and_then(|channel| channel.as_str())
        .ok_or_else(|| anyhow!("OKX message missing arg.channel field"))?;

    // Validate that the channel in the JSON matches what we expect
    if actual_channel_name != expected_channel_str {
        return Err(anyhow!(
            "OKX channel mismatch: expected '{}', got '{}' for channel {:?}",
            expected_channel_str,
            actual_channel_name,
            channel
        ));
    }

    match channel {
        Channel::Trades | Channel::AggTrades => {
            let root: OkxTradeMessage =
                serde_json::from_str(json).map_err(|e| anyhow!("OKX trade parse error: {}", e))?;
            // For now, assume first trade
            let trade = &root.data[0];
            let timestamp = trade.transaction_time;
            let instrument = trade.instrument.clone();

            // Use unified conversion method
            let data = trade.clone().to_unified();

            Ok(ExchangeStreamEvent {
                venue: *venue,
                channel: *channel,
                instrument,
                timestamp,
                data,
            })
        }
        Channel::OpenInterest => {
            let root: OkxOpenInterestMessage =
                serde_json::from_str(json).map_err(|e| anyhow!("OKX open interest parse error: {}", e))?;
            // For now, assume first open interest data point
            let oi = &root.data[0];
            let timestamp = oi.timestamp;
            let instrument = oi.instrument.clone();

            // Use unified conversion method
            let data = oi.clone().to_unified();

            Ok(ExchangeStreamEvent {
                venue: *venue,
                channel: *channel,
                instrument,
                timestamp,
                data,
            })
        }
        Channel::Ticker => {
            let root: OkxTickerMessage =
                serde_json::from_str(json).map_err(|e| anyhow!("OKX ticker parse error: {}", e))?;
            // For now, assume first ticker data point
            let ticker = &root.data[0];
            let timestamp = ticker.timestamp;
            let instrument = ticker.instrument.clone();

            // Use unified conversion method
            let data = ticker.clone().to_unified();

            Ok(ExchangeStreamEvent {
                venue: *venue,
                channel: *channel,
                instrument,
                timestamp,
                data,
            })
        }
        _ => Err(anyhow!("Unsupported OKX channel: {:?}", channel)),
    }
}
pub struct TardisRequest {
    pub exchange: VenueName,
    pub channel: Channel,
    pub instruments: Vec<String>,
    pub start: UtcDateTime,
    pub end: UtcDateTime,
}

impl TardisRequest {
    pub fn new(
        exchange: VenueName,
        channel: Channel,
        instruments: Vec<String>,
        start: UtcDateTime,
        end: UtcDateTime,
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

#[derive(TypedBuilder)]
pub struct TardisIngestor {
    pub max_concurrent_requests: usize,
    pub venue: VenueName,
    pub channel: Channel,
    pub instruments: Vec<String>,
    pub start: UtcDateTime,
    pub end: UtcDateTime,
    pub base_url: String,
    pub api_secret: Option<String>,
}

impl TardisIngestor {
    fn download_stream(
        &self,
        req: TardisRequest,
    ) -> impl Stream<Item = impl Future<Output = Result<Vec<(UtcDateTime, String)>>> + '_> + '_ {
        let dates = datetime_range_minute(req.start, req.end).expect("Invalid date range");
        let client = TardisHttpClient::new(self.base_url.clone(), self.api_secret.clone());
        stream::iter(dates.into_iter().map(move |datetime| {
            let client = client.clone();
            let exchange_str = mapping::get_tardis_exchange_id(req.exchange).expect("No entry in lookup map");
            let channel_str =
                mapping::get_tardis_channel_str(req.exchange, req.channel).expect("No entry in lookup map");
            let instruments = req.instruments.clone();
            let offset = datetime.time().hour() as i64 * 60 + datetime.time().minute() as i64;

            async move {
                info!(target: "ingestor::tardis", "Tardis downloading {} from {}", channel_str, datetime);
                let buffer: Bytes = client
                    .request(exchange_str.to_owned(), channel_str.to_owned(), instruments, datetime, offset)
                    .await?;

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

    pub fn stream(&self, req: TardisRequest) -> impl Stream<Item = (UtcDateTime, String)> + '_ {
        self.download_stream(req)
            .buffer_unordered(self.max_concurrent_requests)
            .filter_map(|result| async move {
                match result {
                    Ok(values) => Some(stream::iter(values)),
                    Err(e) => {
                        error!(target: "ingestor::tardis", "Error: {}", e);
                        None
                    }
                }
            })
            .flat_map(|stream| stream)
    }

    pub fn stream_parsed<T: DeserializeOwned + 'static>(
        &self,
        req: TardisRequest,
    ) -> impl Stream<Item = (UtcDateTime, T)> + '_ {
        self.download_stream(req)
            .buffer_unordered(self.max_concurrent_requests)
            .filter_map(|result| async move {
                match result {
                    Ok(values) => Some(stream::iter(values)),
                    Err(e) => {
                        error!(target: "ingestor::tardis", "Error: {}", e);
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
                        error!(target: "ingestor::tardis", "{:?}", e);
                        error!(target: "ingestor::tardis", "Data: {}", data);
                        None
                    }
                }
            })
    }
}

async fn download_task(ingestor: Arc<TardisIngestor>, service_ctx: Arc<ServiceCtx>, core_ctx: Arc<CoreCtx>) {
    info!(target: "ingestor::tardis", "Starting tardis ingestor...");
    let persistence_service = Arc::clone(&core_ctx.persistence);

    let shutdown = service_ctx.get_shutdown_token();

    let req = TardisRequest::new(
        ingestor.venue.clone(),
        ingestor.channel.clone(),
        ingestor.instruments.clone(),
        ingestor.start,
        ingestor.end,
    );

    let venue = match persistence_service.get_venue_by_name(&ingestor.venue).await {
        Ok(v) => v,
        Err(e) => {
            error!(target: "ingestor::tardis", "Failed to get venue: {}", e);
            return;
        }
    };

    let stream = ingestor.stream(req);
    pin!(stream);

    // No need to clone persistence_service for each iteration
    loop {
        tokio::select! {
                event = stream.next() => {
                let (_ts, json) = match event {
                  Some(e) => e,
                  None => {
                      info!(target: "ingestor::tardis", "Stream ended");
                      break;
                  }
                };
                // Dynamic parsing based on venue (bulletproof with error handling)
                let stream_event = match parse_stream_event(&json, &ingestor.venue, &ingestor.channel) {
                    Ok(e) => e,
                    Err(e) => {
                        error!(target: "ingestor::tardis", "Failed to parse stream event for venue {}: {}", ingestor.venue, e);
                        error!(target: "ingestor::tardis", "Raw JSON: {}", json);
                        // Bulletproof: Inspect JSON structure for debugging (e.g., check event type)
                        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(&json) {
                            if let Some(event_type) = map.get("e").and_then(|v| v.as_str()) {
                                warn!(target: "ingestor::tardis", "Detected event type in JSON: {} (may indicate unsupported exchange or malformed data)", event_type);
                            } else {
                                warn!(target: "ingestor::tardis", "No event type ('e') found in JSON; structure may be invalid");
                            }
                        } else {
                            warn!(target: "ingestor::tardis", "JSON is not a valid object; skipping");
                        }
                        continue; // Skip malformed data and continue processing
                    }
                };

                // Get instrument once (remove duplicate block)
                let instrument = match persistence_service
                    .get_instrument_by_venue_symbol(&stream_event.venue_symbol(), &venue)
                    .await {
                        Ok(i) => i,
                        Err(e) => {
                            error!(target: "ingestor::tardis", "Failed to get instrument: {}", e);
                            continue;
                        }
                    };

                // Handle the inner event data dynamically based on exchange
                match stream_event.into_inner() {
                    ExchangeEventData::AggTrade(trade) => {
                        let side = if trade.maker { MarketSide::Sell } else { MarketSide::Buy };
                        let trade = AggTrade::new(
                            trade.event_time,
                            instrument.clone(),
                            trade.trade_id,
                            side,
                            trade.price,
                            trade.quantity,
                        );
                        info!(target: "ingestor::tardis", "Agg trade update: {}", trade);
                        core_ctx.publish(Event::AggTradeUpdate(trade.into())).await;
                    }
                    ExchangeEventData::Tick(tick) => {
                        let tick = Tick::new(
                            tick.event_time,
                            instrument.clone(),
                            tick.update_id,
                            tick.bid_price,
                            tick.bid_quantity,
                            tick.ask_price,
                            tick.ask_quantity,
                        );
                        info!(target: "ingestor::tardis", "Tick update: {}", tick);
                        core_ctx.publish(Event::TickUpdate(tick.into())).await;
                    }
                    ExchangeEventData::Trade(trade) => {
                        // Individual trades are not currently handled as events
                        // They could be converted to AggTrade with trade_id as agg_trade_id
                        debug!(target: "ingestor::tardis", "Individual trade received: {} {} {}@{}",
                               trade.event_time, instrument.venue_symbol, trade.quantity, trade.price);
                    }
                    ExchangeEventData::BookUpdate(book) => {
                        // Handle book updates - this would need more work for full implementation
                        debug!(target: "ingestor::tardis", "Book update received: {} bids, {} asks", book.bids.len(), book.asks.len());
                    }
                }
            },
            _ = shutdown.cancelled() => break,
        }
    }
    info!(target: "ingestor::tardis", "Tardis ingestor service stopped.");
}

fn parse_line(line: &str) -> Result<(UtcDateTime, String)> {
    let mut parts = line.splitn(2, ' ');

    // Timestamp part
    let timestamp = parts.next().unwrap_or_default();
    let timestamp = timestamp.trim();
    let timestamp = timestamp.trim_end_matches(':');
    debug!(target: "ingestor::tardis", "Timestamp: {}", &timestamp);

    // Json part
    let json = parts.next().unwrap_or_default();
    let json = json.trim();
    debug!(target: "ingestor::tardis", "Json: {}", &json);

    let format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]Z");
    let Ok(ts) = time::PrimitiveDateTime::parse(timestamp, format) else {
        bail!("Invalid timestamp: {} in line: {}", &timestamp, line);
    };
    let ts = ts.assume_utc().to_utc();

    Ok((ts, json.to_string()))
}

#[async_trait]
impl Runnable for TardisIngestor {
    async fn get_tasks(
        self: Arc<Self>,
        service_ctx: Arc<ServiceCtx>,
        core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        vec![Box::pin(download_task(self.clone(), service_ctx.clone(), core_ctx.clone()))]
    }
}
