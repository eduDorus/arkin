use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
use uuid::Uuid;
use yata::{
    core::Source,
    helpers::MA,
    indicators::{RelativeStrengthIndexInstance, RSI},
    prelude::*,
};

use arkin_core::prelude::*;

use crate::{state::InsightsState, Computation};

#[derive(Debug, Clone, TypedBuilder)]
pub struct RelativeStrengthIndexFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    #[builder(default)]
    store: DashMap<Arc<Instrument>, RelativeStrengthIndexInstance>,
    input: FeatureId,
    output: FeatureId,
    periods: usize,
}

impl Computation for RelativeStrengthIndexFeature {
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
                let ohlcv = self.insight_state.last_candle(instrument.clone(), timestamp)?;

                if let Some(mut rsi) = self.store.get_mut(instrument) {
                    let rsi_res = rsi.next(&ohlcv);
                    let values = rsi_res.values();
                    if values.is_empty() {
                        return None;
                    }
                    let rsi_value = Decimal::from_f64(values[0])?;
                    let insight = Insight::builder()
                        .id(Uuid::new_v4())
                        .event_time(timestamp)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output.clone())
                        .value(rsi_value)
                        .build();
                    Some(Arc::new(insight))
                } else {
                    let rsi = RSI {
                        ma: MA::TMA(self.periods as u8),
                        zone: 0.2,
                        source: Source::TP,
                    }
                    .init(&ohlcv)
                    .ok()?;
                    self.store.insert(instrument.clone(), rsi);
                    None
                }
            })
            .collect::<Vec<_>>();

        self.insight_state.insert_batch(&insights);
        Ok(insights)
    }
}
