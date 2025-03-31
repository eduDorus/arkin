use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use rust_decimal::Decimal;
use tokio::{select, sync::RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{Algorithm, StrategyError, StrategyService};

#[derive(TypedBuilder)]
#[allow(unused)]
pub struct ForecastStrategy {
    pubsub: PubSubHandle,
    strategy: Arc<Strategy>,
    inputs: Vec<FeatureId>,
    threshold: f64,
    #[builder(default = RwLock::new(HashMap::new()))]
    current_weight: RwLock<HashMap<Arc<Instrument>, Decimal>>,
}

#[async_trait]
impl Algorithm for ForecastStrategy {
    async fn insight_tick(&self, tick: Arc<InsightsUpdate>) -> Result<(), StrategyError> {
        info!("Processing insight tick for Forecast Strategy...");

        // Extract inputs from the tick
        let inputs = tick
            .insights
            .clone()
            .into_iter()
            .filter(|x| x.instrument.is_some())
            .filter(|x| self.inputs.contains(&x.feature_id))
            .map(|x| (x.instrument.as_ref().unwrap().clone(), x.value))
            .collect::<Vec<_>>();

        // Calculate average predictions per instrument
        let instrument_avg = inputs
            .iter()
            .fold(HashMap::new(), |mut acc, (instrument, value)| {
                let avg = acc.entry(instrument.clone()).or_insert((0.0, 0));
                *avg = (avg.0 + value, avg.1 + 1);
                acc
            })
            .into_iter()
            .map(|(instrument, (sum, count))| (instrument, sum / count as f64))
            .collect::<HashMap<_, _>>();

        // Process each instrument
        for (instrument, avg) in instrument_avg {
            // Get current weight (default to 0 if none exists)
            let current_weight = self
                .current_weight
                .read()
                .await
                .get(&instrument)
                .cloned()
                .unwrap_or(Weight::new(0, 0));

            // Determine new weight based on current position and prediction
            let new_weight = if current_weight == Decimal::ZERO {
                // No position
                if avg > self.threshold {
                    Weight::new(1, 0) // Enter long
                } else if avg < -self.threshold {
                    Weight::new(-1, 0) // Enter short
                } else {
                    Weight::new(0, 0) // Stay neutral
                }
            } else if current_weight > Decimal::ZERO {
                // Currently long
                if avg <= 0.0 {
                    if avg < -self.threshold {
                        Weight::new(-1, 0) // Flip to short
                    } else {
                        Weight::new(0, 0) // Exit to neutral
                    }
                } else {
                    current_weight // Remain long
                }
            } else {
                // Currently short (current_weight.0 < 0)
                if avg >= 0.0 {
                    if avg > self.threshold {
                        Weight::new(1, 0) // Flip to long
                    } else {
                        Weight::new(0, 0) // Exit to neutral
                    }
                } else {
                    current_weight // Remain short
                }
            };

            // Update and send signal only if weight changes
            if new_weight != current_weight {
                self.current_weight.write().await.insert(instrument.clone(), new_weight);
                let signal = Signal::builder()
                    .event_time(tick.event_time)
                    .strategy(self.strategy.clone())
                    .instrument(instrument)
                    .weight(new_weight)
                    .build();
                info!("Forecast sending signal: {}", signal);
                self.pubsub.publish(signal).await;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl StrategyService for ForecastStrategy {}

#[async_trait]
impl RunnableService for ForecastStrategy {
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting Forecast Strategy...");

        loop {
            select! {
                Some(event) = self.pubsub.recv() => {
                    match event {
                        Event::InsightsUpdate(tick) => {
                            debug!("ForecastStrategy received insight tick: {}", tick.event_time);
                            self.insight_tick(tick).await?;
                        }
                        Event::Finished => {
                          break;
                      }
                        _ => {}
                    }
                    self.pubsub.ack().await;
                }
                _ = _shutdown.cancelled() => {
                    break;
                }
            }
        }
        info!("Forecast Strategy stopped.");
        Ok(())
    }
}
