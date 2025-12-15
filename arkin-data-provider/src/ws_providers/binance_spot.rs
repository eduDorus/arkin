use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
use url::Url;

use arkin_core::prelude::*;

use crate::errors::ProviderError;
use crate::traits::WebSocketProvider;

#[derive(TypedBuilder)]
pub struct BinanceSpotWsProvider {
    persistence: Arc<dyn PersistenceReader>,
    url: Url,
    channel: Channel,
    symbols: Vec<String>,
}

#[async_trait]
impl WebSocketProvider for BinanceSpotWsProvider {
    fn name(&self) -> &str {
        "BinanceSpot"
    }

    fn url(&self) -> String {
        self.url.to_string()
    }

    fn subscribe_msg(&self) -> Option<String> {
        let params = match self.channel {
            Channel::AggTrades => self.symbols.iter().map(|s| format!("{}@aggTrade", s.to_lowercase())).collect(),
            Channel::Trades => self.symbols.iter().map(|s| format!("{}@trade", s.to_lowercase())).collect(),
            Channel::Ticker => self
                .symbols
                .iter()
                .map(|s| format!("{}@bookTicker", s.to_lowercase()))
                .collect(),
            _ => vec![],
        };
        let subscription = Subscription::new(params);
        serde_json::to_string(&subscription).ok()
    }

    async fn parse(&self, msg: &str) -> Result<Option<Event>, ProviderError> {
        debug!("Parsing message: {}", msg);
        let event = match self.channel {
            Channel::AggTrades => {
                let trade = serde_json::from_str::<BinanceSpotAggTrade>(msg).map_err(ProviderError::JsonParseError)?;

                let instrument = self
                    .persistence
                    .get_instrument(
                        &InstrumentQuery::builder()
                            .venue(VenueName::Binance)
                            .instrument_type(InstrumentType::Spot)
                            .synthetic(false)
                            .venue_symbol(trade.symbol.clone())
                            .build(),
                    )
                    .await
                    .map_err(|e| ProviderError::PersistenceError(e.into()))?;
                Event::AggTradeUpdate(
                    AggTrade::builder()
                        .event_time(trade.event_time)
                        .instrument(instrument)
                        .trade_id(trade.trade_id)
                        .side(if trade.is_buyer_maker {
                            MarketSide::Sell
                        } else {
                            MarketSide::Buy
                        })
                        .price(trade.price)
                        .quantity(trade.quantity)
                        .build()
                        .into(),
                )
            }
            Channel::Trades => {
                let trade = serde_json::from_str::<BinanceSpotTrade>(msg).map_err(ProviderError::JsonParseError)?;

                let instrument = self
                    .persistence
                    .get_instrument(
                        &InstrumentQuery::builder()
                            .venue(VenueName::Binance)
                            .instrument_type(InstrumentType::Spot)
                            .synthetic(false)
                            .venue_symbol(trade.symbol.clone())
                            .build(),
                    )
                    .await
                    .map_err(|e| ProviderError::PersistenceError(e.into()))?;
                Event::TradeUpdate(
                    Trade::builder()
                        .event_time(trade.event_time)
                        .instrument(instrument)
                        .trade_id(trade.trade_id)
                        .side(if trade.is_buyer_maker {
                            MarketSide::Sell
                        } else {
                            MarketSide::Buy
                        })
                        .price(trade.price)
                        .quantity(trade.quantity)
                        .build()
                        .into(),
                )
            }
            Channel::Ticker => {
                let top_of_book =
                    serde_json::from_str::<BinanceSpotTopOfBook>(msg).map_err(ProviderError::JsonParseError)?;

                let instrument = self
                    .persistence
                    .get_instrument(
                        &InstrumentQuery::builder()
                            .venue(VenueName::Binance)
                            .instrument_type(InstrumentType::Spot)
                            .synthetic(false)
                            .venue_symbol(top_of_book.symbol.clone())
                            .build(),
                    )
                    .await
                    .map_err(|e| ProviderError::PersistenceError(e.into()))?;
                Event::TickUpdate(
                    Tick::builder()
                        .event_time(UtcDateTime::now())
                        .instrument(instrument)
                        .tick_id(top_of_book.update_id)
                        .bid_price(top_of_book.bid_price)
                        .bid_quantity(top_of_book.bid_quantity)
                        .ask_price(top_of_book.ask_price)
                        .ask_quantity(top_of_book.ask_quantity)
                        .build()
                        .into(),
                )
            }
            _ => return Ok(None),
        };
        Ok(Some(event))
    }
}

// {"e":"aggTrade","E":1762952665217,"s":"ETHUSDT","a":1788977357,"p":"3547.60000000","q":"0.00300000","f":3150299333,"l":3150299334,"T":1762952665216,"m":false,"M":true}
#[derive(Deserialize)]
struct BinanceSpotAggTrade {
    #[serde(rename = "e")]
    _event_type: String,
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    event_time: UtcDateTime,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    _transaction_time: UtcDateTime,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "a")]
    trade_id: u64,
    #[serde(rename = "f")]
    _first_trade_id: u64,
    #[serde(rename = "l")]
    _last_trade_id: u64,
    #[serde(rename = "p")]
    price: Decimal,
    #[serde(rename = "q")]
    quantity: Decimal,
    #[serde(rename = "m")]
    is_buyer_maker: bool,
    #[serde(rename = "M")]
    _ignore: bool,
}

// {"e":"trade","E":1762953264060,"s":"BTCUSDT","t":5473275551,"p":"104897.99000000","q":"0.00084000","T":1762953264060,"m":true,"M":true}
#[derive(Deserialize)]
struct BinanceSpotTrade {
    #[serde(rename = "e")]
    _event_type: String,
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    event_time: UtcDateTime,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    _transaction_time: UtcDateTime,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "t")]
    trade_id: u64,
    #[serde(rename = "p")]
    price: Decimal,
    #[serde(rename = "q")]
    quantity: Decimal,
    #[serde(rename = "m")]
    is_buyer_maker: bool,
    #[serde(rename = "M")]
    _ignore: bool,
}

// {"u":80389305526,"s":"BTCUSDT","b":"104874.62000000","B":"2.99892000","a":"104874.63000000","A":"2.93056000"}
#[derive(Deserialize)]
struct BinanceSpotTopOfBook {
    #[serde(rename = "u")]
    update_id: u64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "b")]
    bid_price: Decimal,
    #[serde(rename = "B")]
    bid_quantity: Decimal,
    #[serde(rename = "a")]
    ask_price: Decimal,
    #[serde(rename = "A")]
    ask_quantity: Decimal,
}

#[derive(Debug, Serialize, Clone)]
struct Subscription {
    pub method: String,
    pub params: Vec<String>,
    pub id: u64,
}

impl Subscription {
    pub fn new(params: Vec<String>) -> Self {
        Self {
            method: "SUBSCRIBE".to_string(),
            params,
            id: 1,
        }
    }
}
