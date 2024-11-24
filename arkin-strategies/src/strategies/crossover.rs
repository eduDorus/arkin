use std::sync::Arc;

use async_trait::async_trait;
use derive_builder::Builder;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};

use arkin_core::prelude::*;

use crate::{Algorithm, StrategyError};

#[derive(Debug, Clone, Builder)]
#[allow(unused)]
pub struct CrossoverStrategy {
    pubsub: Arc<PubSub>,
    id: StrategyId,
    fast_ma: FeatureId,
    slow_ma: FeatureId,
}

#[async_trait]
impl Algorithm for CrossoverStrategy {
    #[instrument(skip_all)]
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), StrategyError> {
        info!("Starting Crossover Strategy...");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn cleanup(&self) -> Result<(), StrategyError> {
        info!("Cleaning up Crossover Strategy...");
        info!("Crossover Strategy cleaned up");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn insight_update(
        &self,
        instruments: &[Arc<Instrument>],
        event_time: OffsetDateTime,
        insights: &[Insight],
    ) -> Result<Vec<Signal>, StrategyError> {
        info!("Processing insights for Crossover Strategy...");
        let signals = instruments
            .iter()
            .filter_map(|i| {
                let fast_ma = insights.iter().find(|x| {
                    if let Some(inst) = x.instrument.as_ref() {
                        inst == i && x.feature_id == self.fast_ma
                    } else {
                        false
                    }
                });

                let slow_ma = insights.iter().find(|x| {
                    if let Some(inst) = x.instrument.as_ref() {
                        inst == i && x.feature_id == self.slow_ma
                    } else {
                        false
                    }
                });

                match (fast_ma, slow_ma) {
                    (Some(f), Some(s)) => match f.value > s.value {
                        true => {
                            return Some(
                                SignalBuilder::default()
                                    .event_time(event_time)
                                    .instrument(i.clone())
                                    .strateg_id(self.id.clone())
                                    .weight(Decimal::ONE)
                                    .build()
                                    .expect("Failed to create signal"),
                            )
                        }
                        false => {
                            let mut weight = Decimal::ONE;
                            weight.set_sign_negative(true);
                            return Some(
                                SignalBuilder::default()
                                    .event_time(event_time)
                                    .instrument(i.clone())
                                    .strateg_id(self.id.clone())
                                    .weight(weight)
                                    .build()
                                    .expect("Failed to create signal"),
                            );
                        }
                    },
                    _ => return None,
                }
            })
            .collect::<Vec<_>>();
        Ok(signals)
    }
}
