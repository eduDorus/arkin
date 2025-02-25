use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::*;
use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
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
    persist: bool,
}

impl Computation for RelativeStrengthIndexFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instruments: &[Arc<Instrument>], timestamp: OffsetDateTime) -> Result<Vec<Arc<Insight>>> {
        debug!("Calculating RSI...");

        let insights = instruments
            .par_iter()
            .filter_map(|instrument| {
                // Get data from state
                let ohlcv = self.insight_state.last_candle(instrument.clone(), timestamp)?;

                if let Some(mut rsi) = self.store.get_mut(instrument) {
                    let res = rsi.next(&ohlcv);
                    let values = res.values();
                    if values.is_empty() {
                        return None;
                    }
                    let rsi_value = values[0];
                    let insight = Insight::builder()
                        .event_time(timestamp)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output.clone())
                        .value(rsi_value)
                        .persist(self.persist)
                        .build();
                    Some(Arc::new(insight))
                } else {
                    let rsi = RSI {
                        ma: MA::DMA(self.periods as u8),
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
