use std::{cmp::Ordering, fmt, sync::Arc};

use rust_decimal::prelude::*;
use time::OffsetDateTime;

use crate::{
    constants::{TRADE_PRICE_FEATURE_ID, TRADE_QUANTITY_FEATURE_ID},
    models::Insight,
    Event, Price, Quantity, UpdateEventType,
};

use super::{Instrument, MarketSide};

#[derive(Debug, Clone)]
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

    pub fn to_insights(self) -> Vec<Insight> {
        vec![
            Insight::new(
                self.event_time,
                Some(self.instrument.clone()),
                TRADE_PRICE_FEATURE_ID.clone(),
                self.price,
            ),
            Insight::new(
                self.event_time,
                Some(self.instrument),
                TRADE_QUANTITY_FEATURE_ID.clone(),
                self.quantity * Decimal::from(self.side),
            ),
        ]
    }
}

impl Event for Trade {
    fn event_type() -> UpdateEventType {
        UpdateEventType::Trade
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {}", self.instrument, self.event_time, self.price, self.quantity)
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
