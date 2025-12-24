use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use time::UtcDateTime;
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

#[async_trait]
impl Feature for ChaikinMoneyFlowFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, timestamp: UtcDateTime) -> Option<Vec<Insight>> {
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
                .build();
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

    async fn async_calculate(&self, instrument: &Arc<Instrument>, timestamp: UtcDateTime) -> Option<Vec<Insight>> {
        self.calculate(instrument, timestamp)
    }
}
