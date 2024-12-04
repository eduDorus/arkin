use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
use uuid::Uuid;
use yata::{methods::SMA, prelude::*};

use arkin_core::prelude::*;

use crate::{state::InsightsState, Computation};

#[derive(Debug, Clone, TypedBuilder)]
pub struct SimpleMovingAverageFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    #[builder(default)]
    store: DashMap<Arc<Instrument>, SMA>,
    input: FeatureId,
    output: FeatureId,
    periods: usize,
}

impl Computation for SimpleMovingAverageFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instruments: &[Arc<Instrument>], timestamp: OffsetDateTime) -> Result<Vec<Arc<Insight>>> {
        debug!("Calculating SMA");

        // Calculate the mean (SMA)
        let insights = instruments
            .iter()
            .filter_map(|instrument| {
                // Get data from state
                let value = self
                    .insight_state
                    .last(Some(instrument.clone()), self.input.clone(), timestamp)?;
                let value_f64 = value.to_f64()?;

                if let Some(mut sma) = self.store.get_mut(instrument) {
                    let sma_value = sma.next(&value_f64);
                    let sma_value = Decimal::from_f64(sma_value)?;
                    let insight = Insight::builder()
                        .id(Uuid::new_v4())
                        .event_time(timestamp)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output.clone())
                        .value(sma_value)
                        .build();
                    Some(Arc::new(insight))
                } else {
                    let sma = SMA::new(self.periods as u8, &value_f64).ok()?;
                    self.store.insert(instrument.clone(), sma);
                    None
                }
            })
            .collect::<Vec<_>>();

        self.insight_state.insert_batch(&insights);
        Ok(insights)
    }
}
