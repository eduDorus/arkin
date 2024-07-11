use crate::{config::FeatureConfig, features::vwap::VWAPFeature};

use super::FeatureType;

pub struct FeatureFactory {
    pub features: Vec<FeatureConfig>,
}

impl FeatureFactory {
    pub fn new(features: Vec<FeatureConfig>) -> FeatureFactory {
        FeatureFactory { features }
    }

    pub fn create_features(&self) -> Vec<FeatureType> {
        let mut features = Vec::new();

        for config in &self.features {
            match config {
                FeatureConfig::VWAP(config) => {
                    features.push(FeatureType::VWAP(VWAPFeature::new(config)));
                }
                FeatureConfig::SMA(_) => unimplemented!(),
                FeatureConfig::EMA(_) => unimplemented!(),
            }
        }

        features
    }
}
