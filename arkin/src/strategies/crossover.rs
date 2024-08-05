use super::{Strategy, StrategyId};
use crate::{
    config::CrossoverConfig,
    features::{FeatureEvent, FeatureId},
    models::{Signal, Weight},
};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct CrossoverStrategy {
    id: StrategyId,
    price_spread_id: FeatureId,
    volume_spread_id: FeatureId,
}

impl CrossoverStrategy {
    pub fn from_config(config: &CrossoverConfig) -> Self {
        Self {
            id: config.id.clone(),
            price_spread_id: config.price_spread_id.to_owned(),
            volume_spread_id: config.volume_spread_id.to_owned(),
        }
    }
}

impl Strategy for CrossoverStrategy {
    fn id(&self) -> &StrategyId {
        &self.id
    }

    fn sources(&self) -> Vec<FeatureId> {
        vec![self.price_spread_id.clone(), self.volume_spread_id.clone()]
    }

    fn calculate(&self, data: Vec<FeatureEvent>) -> Vec<Signal> {
        let price_spread = data
            .iter()
            .find(|d| d.id == self.price_spread_id)
            .expect("Missing price spread");
        let volume_spread = data
            .iter()
            .find(|d| d.id == self.volume_spread_id)
            .expect("Missing volume spread");

        // If price is high and volume is high we want to sell
        // If price is low and volume is high we want to buy
        match (price_spread.value, volume_spread.value) {
            (p, v) if p > 0. && v > 0. => vec![Signal::new(
                price_spread.event_time,
                price_spread.instrument.clone(),
                self.id.clone(),
                Weight::from(-1.),
            )],
            (p, v) if p < 0. && v > 0. => vec![Signal::new(
                price_spread.event_time,
                price_spread.instrument.clone(),
                self.id.clone(),
                Weight::from(1.),
            )],
            _ => vec![Signal::new(
                price_spread.event_time,
                price_spread.instrument.clone(),
                self.id.clone(),
                Weight::from(0.),
            )],
        }
    }
}
