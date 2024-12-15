use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::*;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;
use yata::{
    methods::{DMA, EMA, SMA, TMA},
    prelude::*,
};

use arkin_core::prelude::*;

use crate::{state::InsightsState, Computation};

#[derive(Debug, Clone, TypedBuilder)]
pub struct MovingAverageFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    #[builder(default)]
    sma_store: DashMap<Arc<Instrument>, SMA>,
    #[builder(default)]
    ema_store: DashMap<Arc<Instrument>, EMA>,
    #[builder(default)]
    dma_store: DashMap<Arc<Instrument>, DMA>,
    #[builder(default)]
    tma_store: DashMap<Arc<Instrument>, TMA>,
    ma_type: String,
    input: FeatureId,
    output: FeatureId,
    periods: usize,
}

impl Computation for MovingAverageFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instruments: &[Arc<Instrument>], timestamp: OffsetDateTime) -> Result<Vec<Arc<Insight>>> {
        debug!("Calculating {}...", self.ma_type);

        let insights = instruments
            .par_iter()
            .filter_map(|instrument| {
                // Get data from state
                let value = self
                    .insight_state
                    .last(Some(instrument.clone()), self.input.clone(), timestamp)?;
                let value_f64 = value.to_f64()?;

                // I know this is horrible but I have no time to do this in a optimal way. If you see this and have some time, please refactor this.
                match self.ma_type.as_str() {
                    "SMA" => {
                        if let Some(mut sma) = self.sma_store.get_mut(instrument) {
                            let sma_value = sma.next(&value_f64);
                            let sma_value = Decimal::from_f64(sma_value)?;
                            let insight = Insight::builder()
                                .event_time(timestamp)
                                .pipeline(self.pipeline.clone())
                                .instrument(Some(instrument.clone()))
                                .feature_id(self.output.clone())
                                .value(sma_value)
                                .build();
                            Some(Arc::new(insight))
                        } else {
                            let sma = SMA::new(self.periods as u8, &value_f64).ok()?;
                            self.sma_store.insert(instrument.clone(), sma);
                            None
                        }
                    }
                    "EMA" => {
                        if let Some(mut ema) = self.ema_store.get_mut(instrument) {
                            let ema_value = ema.next(&value_f64);
                            let ema_value = Decimal::from_f64(ema_value)?;
                            let insight = Insight::builder()
                                .event_time(timestamp)
                                .pipeline(self.pipeline.clone())
                                .instrument(Some(instrument.clone()))
                                .feature_id(self.output.clone())
                                .value(ema_value)
                                .build();
                            Some(Arc::new(insight))
                        } else {
                            let ema = EMA::new(self.periods as u8, &value_f64).ok()?;
                            self.ema_store.insert(instrument.clone(), ema);
                            None
                        }
                    }
                    "DMA" => {
                        if let Some(mut dma) = self.dma_store.get_mut(instrument) {
                            let dma_value = dma.next(&value_f64);
                            let dma_value = Decimal::from_f64(dma_value)?;
                            let insight = Insight::builder()
                                .event_time(timestamp)
                                .pipeline(self.pipeline.clone())
                                .instrument(Some(instrument.clone()))
                                .feature_id(self.output.clone())
                                .value(dma_value)
                                .build();
                            Some(Arc::new(insight))
                        } else {
                            let dma = DMA::new(self.periods as u8, &value_f64).ok()?;
                            self.dma_store.insert(instrument.clone(), dma);
                            None
                        }
                    }
                    "TMA" => {
                        if let Some(mut tma) = self.tma_store.get_mut(instrument) {
                            let tma_value = tma.next(&value_f64);
                            let tma_value = Decimal::from_f64(tma_value)?;
                            let insight = Insight::builder()
                                .event_time(timestamp)
                                .pipeline(self.pipeline.clone())
                                .instrument(Some(instrument.clone()))
                                .feature_id(self.output.clone())
                                .value(tma_value)
                                .build();
                            Some(Arc::new(insight))
                        } else {
                            let tma = TMA::new(self.periods as u8, &value_f64).ok()?;
                            self.tma_store.insert(instrument.clone(), tma);
                            None
                        }
                    }
                    _ => {
                        warn!("Unknown MA type {} from [SMA, EMA, TMA]", self.ma_type);
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        self.insight_state.insert_batch(&insights);
        Ok(insights)
    }
}
