use std::sync::Arc;

use dashmap::DashMap;
use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
use yata::{
    indicators::{ChaikinMoneyFlow, ChaikinMoneyFlowInstance},
    prelude::*,
};

use arkin_core::prelude::*;

use crate::{state::InsightsState, Feature};

#[derive(Debug, Clone, TypedBuilder)]
pub struct ChaikinMoneyFlowFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    #[builder(default)]
    store: DashMap<Arc<Instrument>, ChaikinMoneyFlowInstance>,
    input: FeatureId,
    output: FeatureId,
    periods: usize,
    persist: bool,
}

impl Feature for ChaikinMoneyFlowFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, timestamp: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating Chaikin Money Flow...");

        // Get data from state
        let ohlcv = self.insight_state.last_candle(instrument.clone(), timestamp)?;

        if let Some(mut rsi) = self.store.get_mut(instrument) {
            let res = rsi.next(&ohlcv);
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
            let cmf = ChaikinMoneyFlow {
                size: self.periods as u8,
            }
            .init(&ohlcv)
            .ok()?;
            self.store.insert(instrument.clone(), cmf);
            None
        }
    }
}
