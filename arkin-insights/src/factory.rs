use std::{sync::Arc, time::Duration};

use arkin_core::prelude::*;

use crate::{
    allocation::MeanVarianceFeature,
    config::FeatureConfig,
    forecast::CatBoostFeature,
    simple::{LogReturnFeature, OHLCVFeature, SignalStrengthFeature, StdDevFeature, SumFeature, TimeFeature},
    state::InsightsState,
    ta::{
        AverageDirectionalIndexFeature, ChaikinMoneyFlowFeature, ChaikinOscillatorFeature, MovingAverageFeature,
        RelativeStrengthIndexFeature,
    },
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
                    FeatureConfig::Time(c) => Box::new(
                        TimeFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output_day_of_week(c.output_day_of_week.clone())
                            .output_hour_of_day(c.output_hour_of_day.clone())
                            .output_minute_of_day(c.output_minute_of_day.clone())
                            .output_minute_of_hour(c.output_minute_of_hour.clone())
                            .build(),
                    ),
                    FeatureConfig::LogReturn(c) => Box::new(
                        LogReturnFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .build(),
                    ),
                    FeatureConfig::StdDev(c) => Box::new(
                        StdDevFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .build(),
                    ),
                    FeatureConfig::Sum(c) => Box::new(
                        SumFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .build(),
                    ),
                    FeatureConfig::SignalStrength(c) => Box::new(
                        SignalStrengthFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input_first(c.input_first.clone())
                            .input_second(c.input_second.clone())
                            .output(c.output.clone())
                            .build(),
                    ),
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
                    FeatureConfig::CO(c) => Box::new(
                        ChaikinOscillatorFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods_fast(c.periods_fast)
                            .periods_slow(c.periods_slow)
                            .build(),
                    ),
                    FeatureConfig::CatBoost(c) => Box::new(
                        CatBoostFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .model_location(c.model_location.clone())
                            .model_file_name(c.model_filename.clone())
                            .input_numerical(c.input_numerical.clone())
                            .input_categorical(c.input_categorical.clone())
                            .output(c.output.clone())
                            .build(),
                    ),
                    FeatureConfig::MeanVariance(c) => Box::new(
                        MeanVarianceFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input_expected_returns(c.input_expected_returns.clone())
                            .input_returns(c.input_returns.clone())
                            .output(c.output.clone())
                            .periods_returns(c.periods_returns)
                            .risk_aversion(c.risk_aversion)
                            .risk_free_rate(c.risk_free_rate)
                            .max_exposure_long(c.max_exposure_long)
                            .max_exposure_short(c.max_exposure_short)
                            .max_exposure_long_per_asset(c.max_exposure_long_per_asset)
                            .max_exposure_short_per_asset(c.max_exposure_short_per_asset)
                            .build(),
                    ),
                };
                feature
            })
            .collect()
    }
}
