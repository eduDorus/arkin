use std::{cmp::Ordering, fmt, sync::Arc};

use time::UtcDateTime;
use typed_builder::TypedBuilder;

use crate::{prelude::TIMESTAMP_FORMAT, Price, Quantity};

use super::{Instrument, MarketSide};

#[derive(Debug, Clone, TypedBuilder)]
pub struct AggTrade {
    pub event_time: UtcDateTime,
    pub instrument: Arc<Instrument>,
    pub trade_id: u64,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
}

impl AggTrade {
    pub fn new(
        event_time: UtcDateTime,
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
}

impl PartialEq for AggTrade {
    fn eq(&self, other: &Self) -> bool {
        self.event_time == other.event_time
            && self.trade_id == other.trade_id
            && Arc::ptr_eq(&self.instrument, &other.instrument)
    }
}

impl Eq for AggTrade {}

impl PartialOrd for AggTrade {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AggTrade {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.event_time, self.trade_id).cmp(&(other.event_time, other.trade_id))
    }
}

impl fmt::Display for AggTrade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let event_time = self.event_time.format(TIMESTAMP_FORMAT).expect("Failed to format timestamp");
        write!(
            f,
            "event_time={} instrument={} side={} price={} quantity={}",
            event_time, self.instrument, self.side, self.price, self.quantity
        )
    }
}
