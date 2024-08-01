use std::fmt;

use time::OffsetDateTime;

use super::{Instrument, Notional};

#[derive(Clone)]
pub struct Allocation {
    pub received_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub strategy_id: String,
    pub notional: Notional,
}

impl fmt::Display for Allocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.event_time, self.strategy_id, self.instrument, self.notional
        )
    }
}
