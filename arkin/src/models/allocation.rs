use super::{Instrument, Notional};
use crate::strategies::StrategyId;
use std::fmt;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct AllocationEvent {
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub strategy_id: StrategyId,
    pub notional: Notional,
}

impl AllocationEvent {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Instrument,
        strategy_id: StrategyId,
        notional: Notional,
    ) -> Self {
        Self {
            event_time,
            instrument,
            strategy_id,
            notional,
        }
    }
}

impl fmt::Display for AllocationEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.event_time, self.strategy_id, self.instrument, self.notional
        )
    }
}
