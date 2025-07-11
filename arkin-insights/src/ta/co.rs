use std::sync::Arc;

use dashmap::DashMap;
use time::UtcDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
use yata::{
    helpers::MA,
    indicators::{ChaikinOscillator, ChaikinOscillatorInstance},
    prelude::*,
};

use arkin_core::prelude::*;

use crate::{state::InsightsState, Feature};

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
    persist: bool,
}

impl Feature for ChaikinOscillatorFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, timestamp: UtcDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating Chaikin Oscillator...");

        // Get data from state
        let ohlcv = self.insight_state.last_candle(instrument.clone(), timestamp)?;

        if let Some(mut co) = self.store.get_mut(instrument) {
            let res = co.next(&ohlcv);
            let values = res.values();
            if values.is_empty() {
                return None;
            }
            let value = values[0];
            let insight = Insight::builder()
                .event_time(timestamp)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output.clone())
                .value(value)
                .persist(self.persist)
                .insight_type(InsightType::Continuous)
                .build()
                .into();
            Some(vec![insight])
        } else {
            let res = ChaikinOscillator {
                ma1: MA::DMA(self.periods_fast as u8),
                ma2: MA::DMA(self.periods_slow as u8),
                window: 0,
            }
            .init(&ohlcv);
            match res {
                Ok(co) => {
                    self.store.insert(instrument.clone(), co);
                }
                Err(e) => {
                    debug!("Failed to initialize Chaikin Oscillator: {}", e);
                }
            }
            None
        }
    }
}
