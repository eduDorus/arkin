use crate::{config::FeatureConfig, service::FeatureModule, simple::CountFeature};

pub struct FeatureFactory {}

impl FeatureFactory {
    pub fn from_config(configs: &[FeatureConfig]) -> Vec<Box<dyn FeatureModule>> {
        // Create nodes
        configs
            .iter()
            .map(|config| {
                let feature: Box<dyn FeatureModule> = match config {
                    FeatureConfig::Count(c) => Box::new(CountFeature::from_config(c)),
                    // FeatureConfig::Sum(c) => Box::new(SumFeature::from_config(c)),
                    // FeatureConfig::Mean(c) => Box::new(MeanFeature::from_config(c)),
                    // FeatureConfig::CumSum(c) => Box::new(CumSumFeature::from_config(c)),
                    // FeatureConfig::PctChange(c) => Box::new(PctChangeFeature::from_config(c)),
                    // FeatureConfig::StdDev(c) => Box::new(StdDevFeature::from_config(c)),
                    // FeatureConfig::VWAP(c) => Box::new(VWAPFeature::from_config(c)),
                    // FeatureConfig::SMA(c) => Box::new(SMAFeature::from_config(c)),
                    // FeatureConfig::Spread(c) => Box::new(SpreadFeature::from_config(c)),
                };
                feature
            })
            .collect()
    }
}
