use std::sync::Arc;

use arkin_core::Pipeline;

use crate::{config::FeatureConfig, state::InsightsState, ta::SimpleMovingAverageFeature, Computation};

pub struct FeatureFactory {}

impl FeatureFactory {
    pub fn from_config(
        configs: &[FeatureConfig],
        pipeline: Arc<Pipeline>,
        state: Arc<InsightsState>,
    ) -> Vec<Box<dyn Computation>> {
        // Create nodes
        configs
            .iter()
            .map(|config| {
                let feature: Box<dyn Computation> = match config {
                    // FeatureConfig::OHLCV(c) => Box::new(OHLCVFeature::from_config(c)),
                    // FeatureConfig::VWAP(c) => Box::new(VWAPFeature::from_config(c)),
                    // FeatureConfig::PctChange(c) => Box::new(PctChangeFeature::from_config(c)),
                    // FeatureConfig::HistVol(c) => Box::new(HistVolFeature::from_config(c)),
                    // FeatureConfig::TradeCount(c) => Box::new(TradeCountFeature::from_config(c)),
                    // FeatureConfig::StdDev(c) => Box::new(StdDevFeature::from_config(c)),
                    FeatureConfig::SMA(c) => Box::new(
                        SimpleMovingAverageFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .build(),
                    ),
                    // FeatureConfig::EMA(c) => Box::new(ExponentialMovingAverageFeature::from_config(c)),
                    // FeatureConfig::MACD(c) => Box::new(MACDFeature::from_config(c)),
                    // FeatureConfig::BB(c) => Box::new(BollingerBandsFeature::from_config(c)),
                    // FeatureConfig::RSI(c) => Box::new(RelativeStrengthIndexFeature::from_config(c)),
                };
                feature
            })
            .collect()
    }
}
