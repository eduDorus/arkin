use crate::config::FeatureConfig;

use super::{CountFeature, Feature, MeanFeature, PositionFeature, SMAFeature, SpreadFeature, SumFeature, VWAPFeature};

pub struct FeatureFactory {}

impl FeatureFactory {
    pub fn from_config(config: &[FeatureConfig]) -> Vec<Box<dyn Feature>> {
        let mut features = Vec::with_capacity(config.len());

        // Create nodes
        config.iter().for_each(|c| {
            let f: Box<dyn Feature> = match &c {
                FeatureConfig::Count(c) => Box::new(CountFeature::from_config(c)),
                FeatureConfig::Mean(c) => Box::new(MeanFeature::from_config(c)),
                FeatureConfig::Sum(c) => Box::new(SumFeature::from_config(c)),
                FeatureConfig::VWAP(c) => Box::new(VWAPFeature::from_config(c)),
                FeatureConfig::SMA(c) => Box::new(SMAFeature::from_config(c)),
                FeatureConfig::Spread(c) => Box::new(SpreadFeature::from_config(c)),
                FeatureConfig::Position(c) => Box::new(PositionFeature::from_config(c)),
            };
            features.push(f);
        });
        features
    }
}
