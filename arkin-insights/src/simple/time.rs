use std::sync::Arc;

use anyhow::Result;
use rayon::prelude::*;
use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{state::InsightsState, Computation};

#[derive(Debug, Clone, TypedBuilder)]
pub struct TimeFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input: FeatureId,
    output_day_of_week: FeatureId,
    output_hour_of_day: FeatureId,
    output_minute_of_day: FeatureId,
    output_minute_of_hour: FeatureId,
    persist: bool,
}

impl Computation for TimeFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![
            self.output_day_of_week.clone(),
            self.output_hour_of_day.clone(),
            self.output_minute_of_day.clone(),
            self.output_minute_of_hour.clone(),
        ]
    }

    fn calculate(&self, instruments: &[Arc<Instrument>], event_time: OffsetDateTime) -> Result<Vec<Arc<Insight>>> {
        debug!("Calculating Time Features...");

        let day_of_week = event_time.weekday().number_from_monday();
        let hour_of_day = event_time.hour();
        let minute_of_day = event_time.hour() as u16 * 60 + event_time.minute() as u16;
        let minute_of_hour = event_time.minute();

        let insights = instruments
            .par_iter()
            .filter_map(|instrument| {
                Some(vec![
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_day_of_week.clone())
                        .value(day_of_week as f64)
                        .persist(self.persist)
                        .build()
                        .into(),
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_hour_of_day.clone())
                        .value(hour_of_day as f64)
                        .persist(self.persist)
                        .build()
                        .into(),
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_minute_of_day.clone())
                        .value(minute_of_day as f64)
                        .persist(self.persist)
                        .build()
                        .into(),
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_minute_of_hour.clone())
                        .value(minute_of_hour as f64)
                        .persist(self.persist)
                        .build()
                        .into(),
                ])
            })
            .flatten()
            .collect::<Vec<_>>();

        self.insight_state.insert_batch(&insights);
        Ok(insights)
    }
}
