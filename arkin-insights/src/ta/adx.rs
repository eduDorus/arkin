use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::*;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
use yata::{
    helpers::MA,
    indicators::{AverageDirectionalIndex, AverageDirectionalIndexInstance},
    prelude::*,
};

use arkin_core::prelude::*;

use crate::{state::InsightsState, Computation};

#[derive(Debug, Clone, TypedBuilder)]
pub struct AverageDirectionalIndexFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    #[builder(default)]
    store: DashMap<Arc<Instrument>, AverageDirectionalIndexInstance>,
    input: FeatureId,
    output: FeatureId,
    periods: usize,
    persist: bool,
}

impl Computation for AverageDirectionalIndexFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instruments: &[Arc<Instrument>], timestamp: OffsetDateTime) -> Result<Vec<Arc<Insight>>> {
        debug!("Calculating ADX...");

        // Calculate the mean (SMA)
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
                    let value = Decimal::from_f64(values[0])?;
                    let insight = Insight::builder()
                        .event_time(timestamp)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output.clone())
                        .value(value)
                        .persist(self.persist)
                        .build();
                    Some(Arc::new(insight))
                } else {
                    let rsi = AverageDirectionalIndex {
                        method1: MA::TMA(self.periods as u8),
                        method2: MA::TMA(self.periods as u8),
                        period1: 1,
                        zone: 0.2,
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
