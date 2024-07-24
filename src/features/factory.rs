use std::sync::Arc;

use crate::{config::FeatureConfig, features::vwap::VWAPFeature, state::StateManager};

use super::FeatureType;

pub struct FeatureFactory {}

impl FeatureFactory {
    pub fn from_config(state: Arc<StateManager>, config: &[FeatureConfig]) -> Vec<FeatureType> {
        let mut features = Vec::new();

        for config in config {
            match config {
                FeatureConfig::VWAP(config) => {
                    features.push(FeatureType::VWAP(VWAPFeature::new(state.to_owned(), config)));
                }
                FeatureConfig::SMA(_) => unimplemented!(),
                FeatureConfig::EMA(_) => unimplemented!(),
            }
        }

        features
    }
}
