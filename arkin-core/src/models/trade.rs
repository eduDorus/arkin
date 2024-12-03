use std::{cmp::Ordering, fmt, sync::Arc};

use derive_builder::Builder;
use rust_decimal::prelude::*;
use time::OffsetDateTime;

use crate::{
    constants::{TRADE_PRICE_FEATURE_ID, TRADE_QUANTITY_FEATURE_ID},
    models::Insight,
    Event, EventType, EventTypeOf, InsightBuilder, Price, Quantity,
};

use super::{Pipeline, Instrument, MarketSide};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct Trade {
    pub event_time: OffsetDateTime,
    pub instrument: Arc<Instrument>,
    pub trade_id: u64,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
}

impl Trade {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Arc<Instrument>,
        trade_id: u64,
        side: MarketSide,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Self {
            event_time,
            instrument,
            trade_id,
            side,
            price,
            quantity,
        }
    }

    pub fn to_insights(self, pipeline: Arc<Pipeline>) -> Vec<Insight> {
        vec![
            InsightBuilder::default()
                .event_time(self.event_time)
                .pipeline(pipeline.clone())
                .instrument(Some(self.instrument.clone()))
                .feature_id(TRADE_PRICE_FEATURE_ID.clone())
                .value(self.price)
                .build()
                .unwrap(),
            InsightBuilder::default()
                .event_time(self.event_time)
                .pipeline(pipeline.clone())
                .instrument(Some(self.instrument.clone()))
                .feature_id(TRADE_QUANTITY_FEATURE_ID.clone())
                .value(self.quantity * Decimal::from(self.side))
                .build()
                .unwrap(),
        ]
    }
}

impl EventTypeOf for Trade {
    fn event_type() -> EventType {
        EventType::Trade
    }
}

impl From<Trade> for Event {
    fn from(trade: Trade) -> Self {
        Event::Trade(trade)
    }
}

impl PartialEq for Trade {
    fn eq(&self, other: &Self) -> bool {
        self.event_time == other.event_time
            && self.trade_id == other.trade_id
            && Arc::ptr_eq(&self.instrument, &other.instrument)
    }
}

impl Eq for Trade {}

impl PartialOrd for Trade {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Trade {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.event_time, self.trade_id).cmp(&(other.event_time, other.trade_id))
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "instrument={} side={} price={} quantity={}",
            self.instrument, self.side, self.price, self.quantity
        )
    }
}
