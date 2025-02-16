use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::prelude::*;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{Algorithm, StrategyError, StrategyService};

#[derive(Debug, Clone, TypedBuilder)]
#[allow(unused)]
pub struct CrossoverStrategy {
    pubsub: Arc<PubSub>,
    id: Arc<Strategy>,
    fast_ma: FeatureId,
    slow_ma: FeatureId,
}

#[async_trait]
impl StrategyService for CrossoverStrategy {}

#[async_trait]
impl RunnableService for CrossoverStrategy {
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting Crossover Strategy...");

        let mut rx = self.pubsub.subscribe();

        loop {
            select! {
                Ok(event) = rx.recv() => {
                    match event {
                        Event::InsightTick(tick) => {
                            debug!("CrossoverStrategy received insight tick: {}", tick.event_time);
                            self.insight_tick(tick).await?;
                        }
                        _ => {}
                    }
                }
                _ = _shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Algorithm for CrossoverStrategy {
    async fn insight_tick(&self, tick: Arc<InsightTick>) -> Result<(), StrategyError> {
        debug!("Processing insight tick for Crossover Strategy...");
        let signals = tick
            .instruments
            .iter()
            .filter_map(|i| {
                let fast_ma = tick.insights.iter().find(|x| {
                    if let Some(inst) = x.instrument.as_ref() {
                        inst == i && x.feature_id == self.fast_ma
                    } else {
                        false
                    }
                });

                let slow_ma = tick.insights.iter().find(|x| {
                    if let Some(inst) = x.instrument.as_ref() {
                        inst == i && x.feature_id == self.slow_ma
                    } else {
                        false
                    }
                });

                let weight = match (fast_ma, slow_ma) {
                    (Some(f), Some(s)) => {
                        info!("Crossover comparing fast_ma: {} and slow_ma: {}", f.value, s.value);
                        match f.value > s.value {
                            true => Decimal::ONE,
                            false => Decimal::NEGATIVE_ONE,
                        }
                    }
                    _ => Decimal::ZERO,
                };
                let signal = Signal::builder()
                    .event_time(tick.event_time)
                    .instrument(i.clone())
                    .strategy(self.id.clone())
                    .weight(weight)
                    .build();
                let signal = Arc::new(signal);
                return Some(signal);
            })
            .collect::<Vec<_>>();

        info!("Crossover sending {} signals", signals.len());
        for signal in signals {
            info!("Crossover sending signal: {}", signal);
            self.pubsub.publish(Event::Signal(signal.clone())).await;
        }
        Ok(())
    }
}
