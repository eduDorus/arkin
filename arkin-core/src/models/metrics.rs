use std::{fmt, sync::Arc};

use rust_decimal::prelude::*;
use sqlx::Type;
use strum::{Display, EnumString};
use time::UtcDateTime;
use typed_builder::TypedBuilder;

use crate::Instrument;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type, EnumString)]
#[strum(serialize_all = "snake_case")]
// #[sqlx(type_name = "metric_type", rename_all = "snake_case")]
pub enum MetricType {
    IndexPrice,
    MarkPrice,
    OpenInterest,
    OpenInterestNotional,
    FundingRate,
    CountTopTraderLongShortRatio,
    VolumeTopTraderLongShortRatio,
    CountLongShortRatio,
    VolumeTakerLongShortRatio,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypedBuilder)]
pub struct Metric {
    pub event_time: UtcDateTime,
    pub instrument: Arc<Instrument>,
    pub metric_type: MetricType,
    pub value: Decimal,
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} = {}", self.instrument.symbol, self.metric_type, self.value)
    }
}
