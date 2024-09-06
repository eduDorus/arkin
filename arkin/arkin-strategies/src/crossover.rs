use arkin_common::prelude::*;

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

    fn calculate(&self, _insights: &InsightsSnapshot) -> Vec<Signal> {
        // let price_spread = data.iter().find(|d| d.id == self.source[0]).expect("Missing price spread");
        // let volume_spread = data.iter().find(|d| d.id == self.source[1]).expect("Missing volume spread");

        // let weight = if volume_spread.value > Decimal::ZERO {
        //     match price_spread.value.cmp(&Decimal::ZERO) {
        //         std::cmp::Ordering::Greater => Weight::from(-1),
        //         std::cmp::Ordering::Less => Weight::from(1),
        //         std::cmp::Ordering::Equal => Weight::from(0),
        //     }
        // } else {
        //     Weight::from(0)
        // };

        // vec![Signal::new(
        //     price_spread.event_time,
        //     price_spread.instrument.clone(),
        //     self.id.clone(),
        //     weight,
        // )]
        vec![]
    }
}
