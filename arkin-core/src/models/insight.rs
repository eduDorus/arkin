use std::{collections::HashSet, fmt, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use strum::Display;
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{EventPayload, FeatureId, InstrumentQuery, PersistenceReader, PipelineQuery};

use super::{Instrument, Pipeline};

#[derive(Debug, Clone, TypedBuilder)]
pub struct InsightsTick {
    pub event_time: UtcDateTime,
    pub frequency: Duration,
}

#[async_trait]
impl EventPayload for InsightsTick {
    type Dto = InsightsTickDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, _persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        Ok(InsightsTick::builder()
            .event_time(dto.event_time)
            .frequency(dto.frequency)
            .build())
    }
}

impl fmt::Display for InsightsTick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InsightsTick: {:?}", self.frequency)
    }
}

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum InsightType {
    Raw,
    Ohlcv,
    Price,
    MovingAverage,
    Continuous,
    Categorical,
    Normalized,
    Prediction,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct Insight {
    pub event_time: UtcDateTime,
    #[builder(default)]
    pub pipeline: Option<Arc<Pipeline>>,
    pub instrument: Arc<Instrument>,
    pub feature_id: FeatureId,
    pub value: f64,
    pub insight_type: InsightType,
    #[builder(default = false)]
    pub persist: bool,
}

#[async_trait]
impl EventPayload for Insight {
    type Dto = InsightDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let instrument = persistence
            .get_instrument(&InstrumentQuery::builder().id(dto.instrument_id).build())
            .await
            .context(format!("Failed to get instrument with id {}", dto.instrument_id))?;

        let pipeline = if let Some(pid) = dto.pipeline_id {
            persistence.get_pipeline(&PipelineQuery::builder().id(pid).build()).await.ok()
        } else {
            None
        };

        Ok(Insight::builder()
            .event_time(dto.event_time)
            .pipeline(pipeline)
            .instrument(instrument)
            .feature_id(dto.feature_id)
            .value(dto.value)
            .insight_type(dto.insight_type)
            .persist(dto.persist)
            .build())
    }
}

impl fmt::Display for Insight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} = {}",
            self.instrument, self.feature_id, self.value
        )
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct InsightsUpdate {
    pub event_time: UtcDateTime,
    pub insights: Vec<Arc<Insight>>,
}

#[async_trait]
impl EventPayload for InsightsUpdate {
    type Dto = InsightsUpdateDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let mut insights = Vec::new();
        for i in dto.insights {
            insights.push(Arc::new(Insight::from_dto(i, persistence.clone()).await?));
        }
        Ok(InsightsUpdate::builder().event_time(dto.event_time).insights(insights).build())
    }
}

impl InsightsUpdate {
    pub fn instruments(&self) -> Vec<Arc<Instrument>> {
        let mut instruments = HashSet::new();
        for insight in &self.insights {
            instruments.insert(insight.instrument.clone());
        }
        instruments.into_iter().collect()
    }
}

impl fmt::Display for InsightsUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InsightsUpdate: {} insights", self.insights.len())
    }
}

#[derive(Serialize, Deserialize)]
pub struct InsightsTickDto {
    pub event_time: UtcDateTime,
    pub frequency: Duration,
}

impl From<InsightsTick> for InsightsTickDto {
    fn from(tick: InsightsTick) -> Self {
        Self {
            event_time: tick.event_time,
            frequency: tick.frequency,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct InsightDto {
    pub event_time: UtcDateTime,
    pub pipeline_id: Option<Uuid>,
    pub instrument_id: Uuid,
    pub feature_id: FeatureId,
    pub value: f64,
    pub insight_type: InsightType,
    pub persist: bool,
}

impl From<Insight> for InsightDto {
    fn from(insight: Insight) -> Self {
        Self {
            event_time: insight.event_time,
            pipeline_id: insight.pipeline.map(|p| p.id),
            instrument_id: insight.instrument.id,
            feature_id: insight.feature_id,
            value: insight.value,
            insight_type: insight.insight_type,
            persist: insight.persist,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct InsightsUpdateDto {
    pub event_time: UtcDateTime,
    pub insights: Vec<InsightDto>,
}

impl From<InsightsUpdate> for InsightsUpdateDto {
    fn from(update: InsightsUpdate) -> Self {
        Self {
            event_time: update.event_time,
            insights: update.insights.iter().map(|i| i.as_ref().clone().into()).collect(),
        }
    }
}
