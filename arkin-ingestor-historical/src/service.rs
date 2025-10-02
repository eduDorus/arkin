use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use arkin_core::prelude::*;
use async_trait::async_trait;
use bytes::Bytes;
use futures::{stream, Stream, StreamExt};
use serde::de::DeserializeOwned;
use time::macros::format_description;
use time::UtcDateTime;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::pin;
use tracing::{debug, error, info, warn};
use typed_builder::TypedBuilder;

use crate::mapping;

use super::http::TardisHttpClient;

pub fn parse_stream_event(json: &str, venue: &Exchange, channel: &Channel) -> Result<ExchangeStreamEvent> {
    match venue {
        Exchange::BinanceSpot
        | Exchange::BinanceUsdmFutures
        | Exchange::BinanceCoinmFutures
        | Exchange::BinanceOptions => match channel {
            Channel::AggTrades | Channel::Trades | Channel::OrderBook | Channel::LongShortRatio | Channel::Ticker => {
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
        },
        Exchange::BybitSpot | Exchange::BybitDerivatives | Exchange::BybitOptions => {
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
                    let root: BybitTickerMessage =
                        serde_json::from_str(json).map_err(|e| anyhow!("Bybit ticker parse error: {}", e))?;

                    let timestamp =
                        time::UtcDateTime::from_unix_timestamp(root.ts / 1000).unwrap_or(time::UtcDateTime::UNIX_EPOCH);
                    let instrument = root.data.symbol.clone();

                    // Create tick data directly since we have the timestamp
                    let tick_data = TickData {
                        event_time: timestamp,
                        transaction_time: timestamp,
                        update_id: 0, // Bybit doesn't provide update IDs in ticker data
                        bid_price: root.data.bid_price,
                        bid_quantity: root.data.bid_quantity,
                        ask_price: root.data.ask_price,
                        ask_quantity: root.data.ask_quantity,
                    };

                    Ok(ExchangeStreamEvent {
                        venue: *venue,
                        channel: *channel,
                        instrument,
                        timestamp,
                        data: ExchangeEventData::Tick(tick_data),
                    })
                }
                _ => Err(anyhow!("Unsupported Bybit channel: {:?}", channel)),
            }
        }
        Exchange::OkxSpot | Exchange::OkxSwap | Exchange::OkxFutures | Exchange::OkxOptions => {
            // First parse the arg to determine the channel type
            let arg_value: serde_json::Value =
                serde_json::from_str(json).map_err(|e| anyhow!("OKX arg parse error: {}", e))?;
            let channel_name = arg_value
                .get("arg")
                .and_then(|arg| arg.get("channel"))
                .and_then(|channel| channel.as_str())
                .ok_or_else(|| anyhow!("OKX message missing arg.channel field"))?;

            match channel_name {
                "trades" => {
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
                "open-interest" => {
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
                "tickers" => {
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
                _ => Err(anyhow!("Unsupported OKX channel: {}", channel_name)),
            }
        }
        _ => Err(anyhow!("Unsupported venue for parsing: {}", venue)),
    }
}

pub struct TardisRequest {
    pub exchange: Exchange,
    pub channel: Channel,
    pub instruments: Vec<String>,
    pub start: UtcDateTime,
    pub end: UtcDateTime,
}

impl TardisRequest {
    pub fn new(
        exchange: Exchange,
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
    pub venue: Exchange,
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

    let venue = match persistence_service.get_venue_by_name(&ingestor.venue.to_string()).await {
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
                        debug!(target: "ingestor::tardis", "Send agg trade update");
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
                        core_ctx.publish(Event::TickUpdate(tick.into())).await;
                        debug!(target: "ingestor::tardis", "Send tick update");
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
