use std::{sync::Arc, time::Duration};

use arkin_core::Pipeline;

use crate::{
    config::FeatureConfig,
    simple::OHLCVFeature,
    state::InsightsState,
    ta::{AverageDirectionalIndexFeature, ChaikinMoneyFlowFeature, MovingAverageFeature, RelativeStrengthIndexFeature},
    Computation,
};

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
                    FeatureConfig::OHLCV(c) => Box::new(
                        OHLCVFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input_price(c.input_price.to_owned())
                            .input_quantity(c.input_quantity.to_owned())
                            .output_open(c.output_open.to_owned())
                            .output_high(c.output_high.to_owned())
                            .output_low(c.output_low.to_owned())
                            .output_close(c.output_close.to_owned())
                            .output_typical_price(c.output_typical_price.to_owned())
                            .output_vwap(c.output_vwap.to_owned())
                            .output_volume(c.output_volume.to_owned())
                            .output_buy_volume(c.output_buy_volume.to_owned())
                            .output_sell_volume(c.output_sell_volume.to_owned())
                            .output_notional_volume(c.output_notional_volume.to_owned())
                            .output_buy_notional_volume(c.output_buy_notional_volume.to_owned())
                            .output_sell_notional_volume(c.output_sell_notional_volume.to_owned())
                            .output_trade_count(c.output_trade_count.to_owned())
                            .output_buy_trade_count(c.output_buy_trade_count.to_owned())
                            .output_sell_trade_count(c.output_sell_trade_count.to_owned())
                            .window(Duration::from_secs(c.window))
                            .build(),
                    ),
                    // FeatureConfig::VWAP(c) => Box::new(VWAPFeature::from_config(c)),
                    // FeatureConfig::PctChange(c) => Box::new(PctChangeFeature::from_config(c)),
                    // FeatureConfig::HistVol(c) => Box::new(HistVolFeature::from_config(c)),
                    // FeatureConfig::TradeCount(c) => Box::new(TradeCountFeature::from_config(c)),
                    // FeatureConfig::StdDev(c) => Box::new(StdDevFeature::from_config(c)),
                    FeatureConfig::MA(c) => Box::new(
                        MovingAverageFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .ma_type(c.ma_type.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .build(),
                    ),
                    // FeatureConfig::EMA(c) => Box::new(ExponentialMovingAverageFeature::from_config(c)),
                    // FeatureConfig::MACD(c) => Box::new(MACDFeature::from_config(c)),
                    // FeatureConfig::BB(c) => Box::new(BollingerBandsFeature::from_config(c)),
                    FeatureConfig::RSI(c) => Box::new(
                        RelativeStrengthIndexFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .build(),
                    ),
                    FeatureConfig::ADX(c) => Box::new(
                        AverageDirectionalIndexFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .build(),
                    ),
                    FeatureConfig::CMF(c) => Box::new(
                        ChaikinMoneyFlowFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .build(),
                    ),
                };
                feature
            })
            .collect()
    }
}
