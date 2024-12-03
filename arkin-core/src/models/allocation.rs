use std::{fmt, sync::Arc};

use derive_builder::Builder;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use uuid::Uuid;

use super::{Instrument, Portfolio, Signal, Strategy};

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
#[builder(setter(into))]
pub struct Allocation {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub event_time: OffsetDateTime,
    pub group_id: Uuid,
    pub portfolio: Arc<Portfolio>,
    pub strategy: Arc<Strategy>,
    pub instrument: Arc<Instrument>,
    pub signal: Arc<Signal>,
    pub weight: Decimal,
}

impl fmt::Display for Allocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "group={} portfolio={} strategy={} instrument={} signal={} weight={}",
            self.group_id,
            self.portfolio.name,
            self.strategy.name,
            self.instrument.symbol,
            self.signal.weight,
            self.weight,
        )
    }
}
