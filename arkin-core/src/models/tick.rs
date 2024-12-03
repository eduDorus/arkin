use std::{cmp::Ordering, fmt, sync::Arc};

use derive_builder::Builder;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::OffsetDateTime;

use crate::{
    constants::{
        TICK_ASK_PRICE_FEATURE_ID, TICK_ASK_QUANTITY_FEATURE_ID, TICK_BID_PRICE_FEATURE_ID,
        TICK_BID_QUANTITY_FEATURE_ID,
    },
    Event, EventType, EventTypeOf, InsightBuilder, Price, Quantity,
};

use super::{Insight, Instrument, Pipeline};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct Tick {
    #[builder(default = OffsetDateTime::now_utc())]
    pub event_time: OffsetDateTime,
    pub instrument: Arc<Instrument>,
    pub tick_id: u64,
    pub bid_price: Price,
    pub bid_quantity: Quantity,
    pub ask_price: Price,
    pub ask_quantity: Quantity,
}

impl Tick {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Arc<Instrument>,
        tick_id: u64,
        bid_price: Price,
        bid_quantity: Quantity,
        ask_price: Price,
        ask_quantity: Quantity,
    ) -> Self {
        Self {
            event_time,
            instrument,
            tick_id,
            bid_price,
            bid_quantity,
            ask_price,
            ask_quantity,
        }
    }

    pub fn to_insights(self, pipeline: Arc<Pipeline>) -> Vec<Insight> {
        vec![
            InsightBuilder::default()
                .event_time(self.event_time)
                .pipeline(pipeline.clone())
                .instrument(Some(self.instrument.clone()))
                .feature_id(TICK_BID_PRICE_FEATURE_ID.clone())
                .value(self.bid_price)
                .build()
                .unwrap(),
            InsightBuilder::default()
                .event_time(self.event_time)
                .pipeline(pipeline.clone())
                .instrument(Some(self.instrument.clone()))
                .feature_id(TICK_BID_QUANTITY_FEATURE_ID.clone())
                .value(self.bid_quantity)
                .build()
                .unwrap(),
            InsightBuilder::default()
                .event_time(self.event_time)
                .pipeline(pipeline.clone())
                .instrument(Some(self.instrument.clone()))
                .feature_id(TICK_ASK_PRICE_FEATURE_ID.clone())
                .value(self.ask_price)
                .build()
                .unwrap(),
            InsightBuilder::default()
                .event_time(self.event_time)
                .pipeline(pipeline.clone())
                .instrument(Some(self.instrument.clone()))
                .feature_id(TICK_ASK_QUANTITY_FEATURE_ID.clone())
                .value(self.ask_quantity)
                .build()
                .unwrap(),
        ]
    }

    pub fn spread(&self) -> Decimal {
        self.ask_price - self.bid_price
    }

    pub fn mid_price(&self) -> Price {
        (self.bid_price + self.ask_price) / dec!(2)
    }

    pub fn bid_price(&self) -> Price {
        self.bid_price
    }

    pub fn ask_price(&self) -> Price {
        self.ask_price
    }
}

impl EventTypeOf for Tick {
    fn event_type() -> EventType {
        EventType::Tick
    }
}

impl From<Tick> for Event {
    fn from(tick: Tick) -> Self {
        Event::Tick(tick)
    }
}

impl PartialEq for Tick {
    fn eq(&self, other: &Self) -> bool {
        self.event_time == other.event_time
            && self.tick_id == other.tick_id
            && Arc::ptr_eq(&self.instrument, &other.instrument)
    }
}

impl Eq for Tick {}

impl PartialOrd for Tick {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tick {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.event_time, self.tick_id).cmp(&(other.event_time, other.tick_id))
    }
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "instrument={} bid_price={} bid_quantity={} ask_price={} ask_quantity={}",
            self.instrument, self.bid_price, self.bid_quantity, self.ask_price, self.ask_quantity
        )
    }
}
