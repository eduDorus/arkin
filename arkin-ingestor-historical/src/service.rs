use std::future::Future;
use std::sync::Arc;

use anyhow::{bail, Result};
use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use async_trait::async_trait;
use bytes::Bytes;
use futures::{stream, Stream, StreamExt};
use serde::de::DeserializeOwned;
use time::macros::format_description;
use time::UtcDateTime;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::pin;
use tracing::{debug, error, info, instrument};
use typed_builder::TypedBuilder;

use crate::binance_swap::BinanceSwapsEvent;
use crate::mapping;

use super::http::TardisHttpClient;

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
    #[builder(default =  "ingestor::tardis".to_owned())]
    pub identifier: String,
    pub publisher: Arc<dyn Publisher>,
    pub persistence: Arc<Persistence>,
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
    pub fn download_stream(
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

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn download_task(&self, ctx: Arc<ServiceCtx>) {
        info!(target: "ingestor::tardis", "Starting tardis ingestor...");
        let persistence_service = Arc::clone(&self.persistence);

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
        while ctx.is_running().await {
            tokio::select! {
                    event = stream.next() => {
                    let (_ts, json) = match event {
                      Some(e) => e,
                      None => {
                          info!(target: "ingestor::tardis", "Stream ended");
                          break;
                      }
                    };
                    debug!(target: "ingestor::tardis", "Received data: {}", json);
                    let event = match serde_json::from_str::<BinanceSwapsEvent>(&json) {
                        Ok(e) => Some(e),
                        Err(e) => {
                            error!(target: "ingestor::tardis", "Failed to parse Binance event: {}", e);
                            error!(target: "ingestor::tardis", "Data: {}", json);
                            None
                        }
                    };

                    let event = match event {
                        Some(e) => {
                            debug!(target: "ingestor::tardis", "{}", e);
                            e
                        }
                        None => {
                            error!(target: "ingestor::tardis", "Failed to parse event, skipping...");
                            continue
                        },
                    };

                    let instrument = persistence_service
                        .get_instrument_by_venue_symbol(&event.venue_symbol())
                        .await.expect("Failed to look up venue symbol");

                    match event {
                        BinanceSwapsEvent::AggTradeStream(stream) => {
                            let trade = stream.data;
                            let side = if trade.maker {
                                MarketSide::Sell
                            } else {
                                MarketSide::Buy
                            };
                            let trade = AggTrade::new(
                                trade.event_time,
                                instrument,
                                trade.agg_trade_id,
                                side,
                                trade.price,
                                trade.quantity,
                            );
                            debug!(target: "ingestor::tardis", "Send agg trade update");
                            self.publisher.publish(Event::AggTradeUpdate(trade.into())).await;
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
                            self.publisher.publish(Event::TickUpdate(tick.into())).await;
                            debug!(target: "ingestor::tardis", "Send tick update");
                        }
                    }
                },
            }
        }
        info!(target: "ingestor::tardis", "Tardis ingestor service stopped.");
    }
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
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn start_tasks(self: Arc<Self>, ctx: Arc<ServiceCtx>) {
        let exec = self.clone();
        let ctx_clone = ctx.clone();
        ctx.spawn(async move { exec.download_task(ctx_clone).await });
    }
}
