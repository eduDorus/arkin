use std::sync::Arc;

use crate::{config::FeatureConfig, features::vwap::VWAPFeature, state::State};

use super::FeatureType;

pub struct FeatureFactory {}

impl FeatureFactory {
    pub fn create_features(state: Arc<State>, config: &[FeatureConfig]) -> Vec<FeatureType> {
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
