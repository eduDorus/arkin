use clickhouse::Row;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::models::{Tick, Trade};

#[derive(Clone, Serialize, Deserialize, Row)]
pub struct TickCH {
    pub received_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub instrument: String,
    pub bid_price: Decimal,
    pub bid_quantity: Decimal,
    pub ask_price: Decimal,
    pub ask_quantity: Decimal,
    pub source: String,
}

impl From<Tick> for TickCH {
    fn from(tick: Tick) -> Self {
        Self {
            received_time: tick.received_time,
            event_time: tick.event_time,
            instrument: tick.instrument.to_string(),
            bid_price: tick.bid_price.value(),
            bid_quantity: tick.bid_quantity.value(),
            ask_price: tick.ask_price.value(),
            ask_quantity: tick.ask_quantity.value(),
            source: tick.source.to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Row)]
pub struct TradeCH {
    pub received_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub instrument: String,
    pub trade_id: u64,
    pub price: Decimal,
    pub quantity: Decimal, // Negative for sell, positive for buy
    pub source: String,
}

impl From<Trade> for TradeCH {
    fn from(trade: Trade) -> Self {
        Self {
            received_time: trade.received_time,
            event_time: trade.event_time,
            instrument: trade.instrument.to_string(),
            trade_id: trade.trade_id,
            price: trade.price.value(),
            quantity: trade.quantity.value(),
            source: trade.source.to_string(),
        }
    }
}
