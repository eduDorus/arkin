use crate::{
    config::FeatureConfig,
    service::Computation,
    simple::{HistVolFeature, OHLCVFeature, PctChangeFeature, StdDevFeature, TradeCountFeature, VWAPFeature},
    ta::{BollingerBandsFeature, EMAFeature, MACDFeature, SMAFeature},
};

pub struct FeatureFactory {}

impl FeatureFactory {
    pub fn from_config(configs: &[FeatureConfig]) -> Vec<Box<dyn Computation>> {
        // Create nodes
        configs
            .iter()
            .map(|config| {
                let feature: Box<dyn Computation> = match config {
                    FeatureConfig::OHLCV(c) => Box::new(OHLCVFeature::from_config(c)),
                    FeatureConfig::VWAP(c) => Box::new(VWAPFeature::from_config(c)),
                    FeatureConfig::PctChange(c) => Box::new(PctChangeFeature::from_config(c)),
                    FeatureConfig::HistVol(c) => Box::new(HistVolFeature::from_config(c)),
                    FeatureConfig::TradeCount(c) => Box::new(TradeCountFeature::from_config(c)),
                    FeatureConfig::StdDev(c) => Box::new(StdDevFeature::from_config(c)),
                    FeatureConfig::SMA(c) => Box::new(SMAFeature::from_config(c)),
                    FeatureConfig::EMA(c) => Box::new(EMAFeature::from_config(c)),
                    FeatureConfig::MACD(c) => Box::new(MACDFeature::from_config(c)),
                    FeatureConfig::BollingerBands(c) => Box::new(BollingerBandsFeature::from_config(c)),
                };
                feature
            })
            .collect()
    }
}
