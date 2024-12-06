use std::sync::Arc;

use anyhow::Result;
use rayon::prelude::*;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
use uuid::Uuid;

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

        let day_of_week =
            Decimal::from_u8(event_time.weekday().number_from_monday()).expect("Day of week should be between 1 and 7");
        let hour_of_day = Decimal::from_u8(event_time.hour()).expect("Hour of day should be between 0 and 23");
        let minute_of_day = Decimal::from_u16(event_time.hour() as u16 * 60 + event_time.minute() as u16)
            .expect("Minute of day should be between 0 and 1440");
        let minute_of_hour = Decimal::from_u8(event_time.minute()).expect("Minute of hour should be between 0 and 59");

        let insights = instruments
            .par_iter()
            .filter_map(|instrument| {
                Some(vec![
                    Insight::builder()
                        .id(Uuid::new_v4())
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_day_of_week.clone())
                        .value(day_of_week)
                        .build()
                        .into(),
                    Insight::builder()
                        .id(Uuid::new_v4())
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_hour_of_day.clone())
                        .value(hour_of_day)
                        .build()
                        .into(),
                    Insight::builder()
                        .id(Uuid::new_v4())
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_minute_of_day.clone())
                        .value(minute_of_day)
                        .build()
                        .into(),
                    Insight::builder()
                        .id(Uuid::new_v4())
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_minute_of_hour.clone())
                        .value(minute_of_hour)
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
