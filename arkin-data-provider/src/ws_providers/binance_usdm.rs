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
pub struct BinanceUsdmWsProvider {
    persistence: Arc<dyn PersistenceReader>,
    url: Url,
    channel: Channel,
    symbols: Vec<String>,
}

#[async_trait]
impl WebSocketProvider for BinanceUsdmWsProvider {
    fn name(&self) -> &str {
        "Binance Usdm"
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
                let trade = serde_json::from_str::<BinanceUsdmAggTrade>(msg).map_err(ProviderError::JsonParseError)?;
                let instrument = self
                    .persistence
                    .get_instrument(
                        &InstrumentQuery::builder()
                            .venue(VenueName::Binance)
                            .instrument_type(InstrumentType::Perpetual)
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
                let trade = serde_json::from_str::<BinanceUsdmTrade>(msg).map_err(ProviderError::JsonParseError)?;
                let instrument = self
                    .persistence
                    .get_instrument(
                        &InstrumentQuery::builder()
                            .venue(VenueName::Binance)
                            .instrument_type(InstrumentType::Perpetual)
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
                    serde_json::from_str::<BinanceUsdmTopOfBook>(msg).map_err(ProviderError::JsonParseError)?;
                let instrument = self
                    .persistence
                    .get_instrument(
                        &InstrumentQuery::builder()
                            .venue(VenueName::Binance)
                            .instrument_type(InstrumentType::Perpetual)
                            .synthetic(false)
                            .venue_symbol(top_of_book.symbol.clone())
                            .build(),
                    )
                    .await
                    .map_err(|e| ProviderError::PersistenceError(e.into()))?;
                Event::TickUpdate(
                    Tick::builder()
                        .event_time(top_of_book.event_time)
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

// {"e":"aggTrade","E":1762954240628,"a":2944596736,"s":"BTCUSDT","p":"104959.30","q":"0.184","f":6857637492,"l":6857637520,"T":1762954240599,"m":false}
#[derive(Deserialize)]
struct BinanceUsdmAggTrade {
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
}

// {"e":"trade","E":1762954309665,"T":1762954309665,"s":"BTCUSDT","t":6857640351,"p":"104930.60","q":"0.008","X":"MARKET","m":false}
#[derive(Deserialize)]
struct BinanceUsdmTrade {
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
    #[serde(rename = "X")]
    _order_type: String,
}

// {"e":"bookTicker","u":9169638653723,"s":"BTCUSDT","b":"104922.30","B":"12.397","a":"104922.40","A":"0.682","T":1762954152631,"E":1762954152632}
#[derive(Deserialize)]
struct BinanceUsdmTopOfBook {
    #[serde(rename = "e")]
    _event_type: String,
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    event_time: UtcDateTime,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    _transaction_time: UtcDateTime,
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
