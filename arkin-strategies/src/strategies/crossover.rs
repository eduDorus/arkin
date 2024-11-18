use async_trait::async_trait;
use derive_builder::Builder;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, instrument};

use arkin_core::prelude::*;

use crate::{Algorithm, StrategyError};

#[derive(Debug, Clone, Builder)]
#[allow(unused)]
pub struct CrossoverStrategy {
    id: StrategyId,
    price_source: FeatureId,
    volume_source: FeatureId,
}

#[async_trait]
impl Algorithm for CrossoverStrategy {
    #[instrument(skip(self))]
    async fn start(&self, _task_tracker: TaskTracker, _shutdown: CancellationToken) -> Result<(), StrategyError> {
        info!("Starting Crossover Strategy...");
        info!("Crossover Strategy started");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cleanup(&self) -> Result<(), StrategyError> {
        info!("Cleaning up Crossover Strategy...");
        info!("Crossover Strategy cleaned up");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_insights(&self, _insights: Vec<Insight>) -> Result<Vec<Signal>, StrategyError> {
        info!("Processing insights for Crossover Strategy...");
        // insights
        //     .instruments()
        //     .par_iter()
        //     .map(|i| {
        //         let price_spread = insights
        //             .get_instrument_insight(i, &self.source[0])
        //             .expect("Missing vwap spread");
        //         let volume_spread = insights
        //             .get_instrument_insight(i, &self.source[1])
        //             .expect("Missing volume spread");

        //         let weight = if volume_spread.value() > Decimal::ZERO {
        //             match price_spread.value().cmp(Decimal::ZERO) {
        //                 std::cmp::Ordering::Greater => Weight::from(-1),
        //                 std::cmp::Ordering::Less => Weight::from(1),
        //                 std::cmp::Ordering::Equal => Weight::from(0),
        //             }
        //         } else {
        //             Weight::from(0)
        //         };

        //         vec![Signal::new(i.clone(), self.id.clone(), weight, price_spread.event_time.clone())]
        //     })
        //     .flatten()
        //     .collect()
        Ok(vec![])
    }
}
