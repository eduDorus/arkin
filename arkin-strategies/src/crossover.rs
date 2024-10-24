use arkin_core::prelude::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rust_decimal::prelude::*;

use crate::{config::CrossoverConfig, manager::StrategyModule};

#[derive(Clone)]
#[allow(unused)]
pub struct CrossoverStrategy {
    id: StrategyId,
    source: Vec<FeatureId>,
}

impl CrossoverStrategy {
    pub fn from_config(config: &CrossoverConfig) -> Self {
        Self {
            id: config.id.clone(),
            source: vec![config.price_spread_id.to_owned(), config.volume_spread_id.to_owned()],
        }
    }
}

impl StrategyModule for CrossoverStrategy {
    fn id(&self) -> &StrategyId {
        &self.id
    }

    fn sources(&self) -> &[FeatureId] {
        &self.source
    }

    fn calculate(&self, insights: &InsightsSnapshot) -> Vec<Signal> {
        insights
            .instruments()
            .par_iter()
            .map(|i| {
                let price_spread = insights
                    .get_instrument_insight(i, &self.source[0])
                    .expect("Missing vwap spread");
                let volume_spread = insights
                    .get_instrument_insight(i, &self.source[1])
                    .expect("Missing volume spread");

                let weight = if volume_spread.value() > Decimal::ZERO {
                    match price_spread.value().cmp(Decimal::ZERO) {
                        std::cmp::Ordering::Greater => Weight::from(-1),
                        std::cmp::Ordering::Less => Weight::from(1),
                        std::cmp::Ordering::Equal => Weight::from(0),
                    }
                } else {
                    Weight::from(0)
                };

                vec![Signal::new(i.clone(), self.id.clone(), weight, price_spread.event_time.clone())]
            })
            .flatten()
            .collect()
    }
}
