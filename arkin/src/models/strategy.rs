use std::fmt;
use time::OffsetDateTime;

use crate::strategies::StrategyId;

use super::{Instrument, Weight};

#[derive(Clone)]
pub struct Signal {
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub strategy_id: StrategyId,
    pub signal: Weight,
}

impl Signal {
    pub fn new(event_time: OffsetDateTime, instrument: Instrument, strategy_id: StrategyId, signal: Weight) -> Self {
        Signal {
            event_time,
            instrument,
            strategy_id,
            signal,
        }
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.event_time, self.strategy_id, self.instrument, self.signal
        )
    }
}
