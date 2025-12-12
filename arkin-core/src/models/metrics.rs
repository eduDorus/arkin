use std::{fmt, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use strum::{Display, EnumString};
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{EventPayload, Instrument, InstrumentQuery, PersistenceReader};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type, EnumString, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
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

#[async_trait]
impl EventPayload for Metric {
    type Dto = MetricDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let instrument = persistence
            .get_instrument(&InstrumentQuery::builder().id(dto.instrument_id).build())
            .await
            .context(format!("Failed to get instrument with id {}", dto.instrument_id))?;

        Ok(Metric::builder()
            .event_time(dto.event_time)
            .instrument(instrument)
            .metric_type(dto.metric_type)
            .value(dto.value)
            .build())
    }
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} = {}", self.instrument.symbol, self.metric_type, self.value)
    }
}

#[derive(Serialize, Deserialize)]
pub struct MetricDto {
    pub event_time: UtcDateTime,
    pub instrument_id: Uuid,
    pub metric_type: MetricType,
    pub value: Decimal,
}

impl From<Metric> for MetricDto {
    fn from(metric: Metric) -> Self {
        Self {
            event_time: metric.event_time,
            instrument_id: metric.instrument.id,
            metric_type: metric.metric_type,
            value: metric.value,
        }
    }
}
