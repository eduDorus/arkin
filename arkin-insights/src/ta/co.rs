use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::*;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
use uuid::Uuid;
use yata::{
    helpers::MA,
    indicators::{ChaikinOscillator, ChaikinOscillatorInstance},
    prelude::*,
};

use arkin_core::prelude::*;

use crate::{state::InsightsState, Computation};

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChaikinOscillatorFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    #[builder(default)]
    store: DashMap<Arc<Instrument>, ChaikinOscillatorInstance>,
    input: FeatureId,
    output: FeatureId,
    periods_fast: usize,
    periods_slow: usize,
}

impl Computation for ChaikinOscillatorFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instruments: &[Arc<Instrument>], timestamp: OffsetDateTime) -> Result<Vec<Arc<Insight>>> {
        debug!("Calculating Chaikin Oscillator...");

        let insights = instruments
            .par_iter()
            .filter_map(|instrument| {
                // Get data from state
                let ohlcv = self.insight_state.last_candle(instrument.clone(), timestamp)?;

                if let Some(mut co) = self.store.get_mut(instrument) {
                    let res = co.next(&ohlcv);
                    let values = res.values();
                    if values.is_empty() {
                        return None;
                    }
                    let value = Decimal::from_f64(values[0])?;
                    let insight = Insight::builder()
                        .id(Uuid::new_v4())
                        .event_time(timestamp)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output.clone())
                        .value(value)
                        .build();
                    Some(Arc::new(insight))
                } else {
                    let co = ChaikinOscillator {
                        ma1: MA::DMA(self.periods_fast as u8),
                        ma2: MA::DMA(self.periods_slow as u8),
                        window: 0,
                    }
                    .init(&ohlcv)
                    .expect("Failed to initialize Chaikin Oscillator");
                    self.store.insert(instrument.clone(), co);
                    None
                }
            })
            .collect::<Vec<_>>();

        self.insight_state.insert_batch(&insights);
        Ok(insights)
    }
}
