use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::info;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{Algorithm, StrategyError};

#[derive(Debug, Clone, TypedBuilder)]
#[allow(unused)]
pub struct CrossoverStrategy {
    pubsub: Arc<PubSub>,
    id: Arc<Strategy>,
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
        insights: &[Arc<Insight>],
    ) -> Result<Vec<Arc<Signal>>, StrategyError> {
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

                let weight = match (fast_ma, slow_ma) {
                    (Some(f), Some(s)) => match f.value > s.value {
                        true => Decimal::ONE,
                        false => Decimal::NEGATIVE_ONE,
                    },
                    _ => Decimal::ZERO,
                };
                let signal = Signal::builder()
                    .event_time(event_time)
                    .instrument(i.clone())
                    .strategy(self.id.clone())
                    .weight(weight)
                    .build();
                let signal = Arc::new(signal);
                return Some(signal);
            })
            .collect::<Vec<_>>();
        Ok(signals)
    }

    async fn insight_tick(&self, _tick: Arc<InsightTick>) -> Result<(), StrategyError> {
        info!("Processing insight tick for Crossover Strategy...");
        // let signals = self
        //     .insight_update(&tick.instruments, tick.event_time, tick.insights.as_slice())
        //     .await?;
        // let signal_tick = AllocationTick::builder()
        //     .event_time(tick.event_time)
        //     .instruments(tick.instruments.clone())
        //     .signals(signals)
        //     .build();
        // let signal_tick = Arc::new(signal_tick);

        // info!(
        //     "Publishing signal tick: {} with {} signals",
        //     signal_tick.event_time,
        //     signal_tick.weight.len()
        // );
        // self.pubsub.publish::<AllocationTick>(signal_tick);
        Ok(())
    }
}
