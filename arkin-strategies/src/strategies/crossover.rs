use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use rust_decimal::prelude::*;
use tokio::{select, sync::RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{Algorithm, StrategyError, StrategyService};

#[derive(TypedBuilder)]
#[allow(unused)]
pub struct CrossoverStrategy {
    pubsub: PubSubHandle,
    strategy: Arc<Strategy>,
    fast_ma: FeatureId,
    slow_ma: FeatureId,
    #[builder(default = RwLock::new(HashMap::new()))]
    current_weight: RwLock<HashMap<Arc<Instrument>, Decimal>>,
}

#[async_trait]
impl StrategyService for CrossoverStrategy {}

#[async_trait]
impl RunnableService for CrossoverStrategy {
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting Crossover Strategy...");

        loop {
            select! {
                Ok((event, barrier)) = self.pubsub.rx.recv() => {
                  info!("CrossoverStrategy received event");
                    match event {
                        Event::InsightsUpdate(tick) => {
                            debug!("CrossoverStrategy received insight tick: {}", tick.event_time);
                            self.insight_tick(tick).await?;
                        }
                        Event::Finished => {
                          barrier.wait().await;
                          break;
                      }
                        _ => {}
                    }
                    info!("CrossoverStrategy event processed");
                    barrier.wait().await;
                }
                _ = _shutdown.cancelled() => {
                    break;
                }
            }
        }
        info!("Crossover Strategy stopped.");
        Ok(())
    }
}

#[async_trait]
impl Algorithm for CrossoverStrategy {
    async fn insight_tick(&self, tick: Arc<InsightsUpdate>) -> Result<(), StrategyError> {
        debug!("Processing insight tick for Crossover Strategy...");

        // Lock the current weight map
        let mut current_weight = self.current_weight.write().await;

        let mut signals = vec![];
        for i in &tick.instruments {
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
                (Some(f), Some(s)) => match f.value > s.value {
                    true => Decimal::ONE,
                    false => Decimal::NEGATIVE_ONE,
                },
                _ => Decimal::ZERO,
            };

            // Check if the weight has changed and update or insert
            let current_weight = current_weight.entry(i.clone()).or_insert(Decimal::ZERO);
            if *current_weight == weight {
                continue;
            }
            *current_weight = weight;

            let signal = Signal::builder()
                .event_time(tick.event_time)
                .instrument(i.clone())
                .strategy(self.strategy.clone())
                .weight(weight)
                .build();
            let signal = Arc::new(signal);
            signals.push(signal);
        }

        debug!("Crossover sending {} signals", signals.len());
        for signal in signals {
            info!("Crossover sending signal: {}", signal);
            self.pubsub.publish(signal).await;
        }
        Ok(())
    }
}
