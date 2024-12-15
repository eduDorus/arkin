use std::sync::Arc;

use anyhow::Result;
use rayon::prelude::*;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{state::InsightsState, Computation};

#[derive(Debug, Clone, TypedBuilder)]
pub struct StdDevFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input: FeatureId,
    output: FeatureId,
    periods: usize,
}

impl Computation for StdDevFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instruments: &[Arc<Instrument>], event_time: OffsetDateTime) -> Result<Vec<Arc<Insight>>> {
        debug!("Calculating StdDev...");

        // Calculate the mean (StdDev)
        let insights = instruments
            .par_iter()
            .filter_map(|instrument| {
                // Get data from state
                let data =
                    self.insight_state
                        .periods(Some(instrument.clone()), self.input.clone(), event_time, self.periods);

                // Check if we have enough data
                if data.len() < self.periods {
                    warn!("Not enough data for StdDev calculation");
                    return None;
                }

                // Calculate StdDev
                let sum = data.iter().sum::<Decimal>();
                let count = Decimal::from(data.len());
                let mean = match count.is_zero() {
                    true => {
                        warn!("Count should not be zero!");
                        return None;
                    }
                    false => sum / count,
                };
                let variance =
                    (Decimal::ONE / (count - Decimal::ONE)) * data.iter().map(|v| (v - mean).powi(2)).sum::<Decimal>();
                if let Some(std_dev) = variance.sqrt() {
                    Some(
                        Insight::builder()
                            .event_time(event_time)
                            .pipeline(self.pipeline.clone())
                            .instrument(Some(instrument.clone()))
                            .feature_id(self.output.clone())
                            .value(std_dev)
                            .build()
                            .into(),
                    )
                } else {
                    warn!("Failed to calculate StdDev: mean: {}, variance: {}", mean, variance);
                    None
                }
            })
            .collect::<Vec<_>>();

        self.insight_state.insert_batch(&insights);
        Ok(insights)
    }
}
