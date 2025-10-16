use std::{cmp::Ordering, fmt, sync::Arc};

use rust_decimal::prelude::*;
use time::UtcDateTime;
use typed_builder::TypedBuilder;

use crate::{
    constants::{
        TICK_ASK_PRICE_FEATURE_ID, TICK_ASK_QUANTITY_FEATURE_ID, TICK_BID_PRICE_FEATURE_ID,
        TICK_BID_QUANTITY_FEATURE_ID,
    },
    prelude::TIMESTAMP_FORMAT,
    Price, Quantity,
};

use super::{Insight, InsightType, Instrument};

#[derive(Debug, Clone, TypedBuilder)]
pub struct Tick {
    pub event_time: UtcDateTime,
    pub instrument: Arc<Instrument>,
    pub tick_id: u64,
    pub bid_price: Price,
    pub bid_quantity: Quantity,
    pub ask_price: Price,
    pub ask_quantity: Quantity,
}

impl Tick {
    pub fn new(
        event_time: UtcDateTime,
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

    pub fn to_insights(&self) -> Vec<Arc<Insight>> {
        let insights = vec![
            Insight::builder()
                .event_time(self.event_time)
                .instrument(Some(self.instrument.clone()))
                .feature_id(TICK_BID_PRICE_FEATURE_ID.clone())
                .value(self.bid_price.to_f64().unwrap_or(f64::NAN))
                .insight_type(InsightType::Raw)
                .build(),
            Insight::builder()
                .event_time(self.event_time)
                .instrument(Some(self.instrument.clone()))
                .feature_id(TICK_BID_QUANTITY_FEATURE_ID.clone())
                .value(self.bid_quantity.to_f64().unwrap_or(f64::NAN))
                .insight_type(InsightType::Raw)
                .build(),
            Insight::builder()
                .event_time(self.event_time)
                .instrument(Some(self.instrument.clone()))
                .feature_id(TICK_ASK_PRICE_FEATURE_ID.clone())
                .value(self.ask_price.to_f64().unwrap_or(f64::NAN))
                .insight_type(InsightType::Raw)
                .build(),
            Insight::builder()
                .event_time(self.event_time)
                .instrument(Some(self.instrument.clone()))
                .feature_id(TICK_ASK_QUANTITY_FEATURE_ID.clone())
                .value(self.ask_quantity.to_f64().unwrap_or(f64::NAN))
                .insight_type(InsightType::Raw)
                .build(),
        ];
        insights.into_iter().map(Arc::new).collect::<Vec<_>>()
    }

    pub fn spread(&self) -> Decimal {
        self.round_to_tick(self.ask_price - self.bid_price)
    }

    pub fn mid_price(&self) -> Price {
        self.round_to_tick((self.bid_price + self.ask_price) / dec!(2))
    }

    pub fn bid_price(&self) -> Price {
        self.round_to_tick(self.bid_price)
    }

    pub fn ask_price(&self) -> Price {
        self.round_to_tick(self.ask_price)
    }

    fn round_to_tick(&self, value: Decimal) -> Decimal {
        if value.is_zero() {
            return Decimal::ZERO;
        }
        let tick_size = self.instrument.tick_size;
        let scaling_factor = Decimal::ONE / tick_size;
        let scaled = value * scaling_factor;
        let rounded_scaled = scaled.round();
        let rounded_value = rounded_scaled * tick_size;
        rounded_value.round_dp(self.instrument.price_precision)
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
        let event_time = self.event_time.format(TIMESTAMP_FORMAT).expect("Failed to format timestamp");
        write!(
            f,
            "event_time={} instrument={} bid_price={} bid_quantity={} ask_price={} ask_quantity={}",
            event_time, self.instrument, self.bid_price, self.bid_quantity, self.ask_price, self.ask_quantity
        )
    }
}
