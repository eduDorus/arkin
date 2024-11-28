use std::sync::Arc;

use async_trait::async_trait;
use derive_builder::Builder;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::info;

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
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), StrategyError> {
        info!("Starting Crossover Strategy...");
        let mut insight_ticks = self.pubsub.subscribe::<InsightTick>();
        loop {
            select! {
                Ok(tick) = insight_ticks.recv() => {
                    info!("CrossoverStrategy received insight tick: {}", tick.event_time);
                    self.insight_tick(tick).await?;
                }
                _ = _shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }

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
                                    .strategy_id(self.id.clone())
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
                                    .strategy_id(self.id.clone())
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

    async fn insight_tick(&self, tick: InsightTick) -> Result<(), StrategyError> {
        info!("Processing insight tick for Crossover Strategy...");
        let signals = self
            .insight_update(&tick.instruments, tick.event_time, tick.insights.as_slice())
            .await?;
        let signal_tick = SignalTickBuilder::default()
            .event_time(tick.event_time)
            .instruments(tick.instruments)
            .signals(signals)
            .build()
            .expect("Failed to create signal tick");

        info!(
            "Publishing signal tick: {} with {} signals",
            signal_tick.event_time,
            signal_tick.signals.len()
        );
        self.pubsub.publish::<SignalTick>(signal_tick);
        Ok(())
    }
}
