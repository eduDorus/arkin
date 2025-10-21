use std::{pin::Pin, sync::Arc};

use async_trait::async_trait;
use tracing::{error, info};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{
    common::{
        config::ConfigurationWebsocketStreams,
        models::{WebsocketEvent, WebsocketMode},
    },
    derivatives_trading_usds_futures::{
        models::market::{BinanceUSDMMarketEvent, BinanceUSDMMarketStreamEvent},
        websocket_streams::{AggregateTradeStreamsParams, IndividualSymbolBookTickerStreamsParams},
        DerivativesTradingUsdsFuturesWsStreams,
    },
};

#[derive(TypedBuilder)]
pub struct BinanceIngestor {
    pub instruments: Vec<Arc<Instrument>>,
    pub venue: Arc<Venue>,
}

pub async fn start_md_task(ingestor: Arc<BinanceIngestor>, core_ctx: Arc<CoreCtx>, service_ctx: Arc<ServiceCtx>) {
    // Build WebSocket Streams config
    let ws_streams_conf = ConfigurationWebsocketStreams::builder().mode(WebsocketMode::Pool(3)).build();

    // Create the DerivativesTradingUsdsFutures WebSocket Streams client
    let ws_streams_client = DerivativesTradingUsdsFuturesWsStreams::production(ws_streams_conf);

    // Connect to WebSocket
    let connection = ws_streams_client.connect().await.expect("Failed to connect to WebSocket");

    // Subscribe to the streams
    for inst in ingestor.instruments.iter() {
        connection
            .individual_symbol_book_ticker_streams(
                IndividualSymbolBookTickerStreamsParams::builder()
                    .symbol(inst.venue_symbol.clone())
                    .build(),
            )
            .await
            .expect("Failed to subscribe to the book ticker stream");

        connection
            .aggregate_trade_streams(AggregateTradeStreamsParams::builder().symbol(inst.venue_symbol.clone()).build())
            .await
            .expect("Failed to subscribe to the agg trade stream");
    }

    let mut rx = connection.subscribe_on_ws_message();

    let shutdown = service_ctx.get_shutdown_token();

    loop {
        tokio::select! {
              Ok(event) = rx.recv() => {
                if let WebsocketEvent::Message(msg) = event {
                  match serde_json::from_str::<BinanceUSDMMarketStreamEvent>(&msg) {
                      Ok(e) => {
                          match e.data {
                              BinanceUSDMMarketEvent::AggTrade(trade) => {
                                  let instrument = match core_ctx.persistence.get_instrument_by_venue_symbol(&trade.instrument, &ingestor.venue).await {
                                      Ok(i) => i,
                                      Err(e) => {
                                          error!("Failed to get instrument: {}", e);
                                          continue;
                                      }
                                  };
                                  // "m": true: The buyer is the market maker.
                                  // • The trade was initiated by a sell order from the taker.
                                  // • The taker is selling, and the maker (buyer) is buying.
                                  // "m": false: The seller is the market maker.
                                  // • The trade was initiated by a buy order from the taker.
                                  // • The taker is buying, and the maker (seller) is selling.
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
                                  let trade = Arc::new(trade);
                                  core_ctx.publish(Event::AggTradeUpdate(trade)).await;
                              }
                              BinanceUSDMMarketEvent::Tick(tick) => {
                                  let instrument = match core_ctx.persistence.get_instrument_by_venue_symbol(&tick.instrument, &ingestor.venue).await {
                                      Ok(i) => i,
                                      Err(e) => {
                                          error!("Failed to get instrument: {}", e);
                                          continue;
                                      }
                                  };
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
                                  core_ctx.publish(Event::TickUpdate(tick)).await;
                              }
                              _ => error!("type not implemented"),
                          }
                      }
                      Err(e) => {
                          error!("Failed to parse WebSocket message: {:?}", e);
                      }
                  }
                }
              }
              _ = shutdown.cancelled() => {
                info!("Shutdown signal received, stopping WebSocket task.");
                connection.disconnect().await.unwrap();
                break;
              }
        }
    }
    info!(target: "ingestor::binance", "WebSocket task finished");
}

#[async_trait]
impl Runnable for BinanceIngestor {
    async fn get_tasks(
        self: Arc<Self>,
        service_ctx: Arc<ServiceCtx>,
        core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        vec![Box::pin(start_md_task(self, core_ctx.clone(), service_ctx.clone()))]
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::{Context, Result};
    use tracing::info;

    use crate::{
        common::config::ConfigurationWebsocketStreams,
        derivatives_trading_usds_futures::{
            websocket_streams::IndividualSymbolBookTickerStreamsParams, DerivativesTradingUsdsFuturesWsStreams,
        },
    };

    #[tokio::test]
    #[test_log::test]
    #[ignore]
    async fn subscribe_binance_agg_trades() -> Result<()> {
        // Build WebSocket Streams config
        let ws_streams_conf = ConfigurationWebsocketStreams::builder().build();

        // Create the DerivativesTradingUsdsFutures WebSocket Streams client
        let ws_streams_client = DerivativesTradingUsdsFuturesWsStreams::production(ws_streams_conf);

        // Connect to WebSocket
        let connection = ws_streams_client
            .connect()
            .await
            .context("Failed to connect to WebSocket Streams")?;

        // Setup the stream parameters
        let params = IndividualSymbolBookTickerStreamsParams::builder()
            .symbol("btcusdt".to_string())
            .build();

        // Subscribe to the stream
        let stream = connection
            .individual_symbol_book_ticker_streams(params)
            .await
            .context("Failed to subscribe to the stream")?;

        // Register callback for incoming messages
        stream.on_message(|data| {
            info!("{:?}", data);
        });

        // Disconnect after 20 seconds
        tokio::time::sleep(Duration::from_secs(20)).await;
        connection.disconnect().await.context("Failed to disconnect WebSocket client")?;

        Ok(())
    }
}
