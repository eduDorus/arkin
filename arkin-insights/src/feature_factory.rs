use std::{collections::HashMap, fs::File, io::BufReader, sync::Arc, time::Duration};

use arkin_core::prelude::*;
use tracing::info;

use crate::{
    config::{FeatureConfig, WindowFeature},
    features::{OHLCVFeature, TimeFeature},
    scaler::{RobustScaler, ScalerData},
    state::InsightsState,
    ta::{
        AverageDirectionalIndexFeature, ChaikinMoneyFlowFeature, ChaikinOscillatorFeature, MovingAverageFeature,
        RelativeStrengthIndexFeature,
    },
    Feature,
};

pub struct FeatureFactory {}

impl FeatureFactory {
    pub fn from_config(
        pipeline: Arc<Pipeline>,
        state: Arc<InsightsState>,
        configs: &[FeatureConfig],
    ) -> Vec<Arc<dyn Feature>> {
        // Create nodes
        configs
            .iter()
            .map(|config| {
                let feature: Arc<dyn Feature> = match config {
                    FeatureConfig::OHLCV(c) => Arc::new(
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
                            .persist(c.persist)
                            .build(),
                    ),
                    FeatureConfig::Time(c) => Arc::new(
                        TimeFeature::builder()
                            .pipeline(pipeline.clone())
                            .input(c.input.clone())
                            .output_day_of_week(c.output_day_of_week.clone())
                            .output_hour_of_day(c.output_hour_of_day.clone())
                            .output_minute_of_day(c.output_minute_of_day.clone())
                            .output_minute_of_hour(c.output_minute_of_hour.clone())
                            .persist(c.persist)
                            .build(),
                    ),
                    FeatureConfig::Window(c) => Arc::new(
                        WindowFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .window(Duration::from_secs(c.periods))
                            .persist(c.persist)
                            .build(),
                    ),
                    FeatureConfig::MA(c) => Arc::new(
                        MovingAverageFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .ma_type(c.ma_type.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .persist(c.persist)
                            .build(),
                    ),
                    FeatureConfig::RSI(c) => Arc::new(
                        RelativeStrengthIndexFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .persist(c.persist)
                            .build(),
                    ),
                    FeatureConfig::ADX(c) => Arc::new(
                        AverageDirectionalIndexFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .persist(c.persist)
                            .build(),
                    ),
                    FeatureConfig::CMF(c) => Arc::new(
                        ChaikinMoneyFlowFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods(c.periods)
                            .persist(c.persist)
                            .build(),
                    ),
                    FeatureConfig::CO(c) => Arc::new(
                        ChaikinOscillatorFeature::builder()
                            .pipeline(pipeline.clone())
                            .insight_state(state.clone())
                            .input(c.input.clone())
                            .output(c.output.clone())
                            .periods_fast(c.periods_fast)
                            .periods_slow(c.periods_slow)
                            .persist(c.persist)
                            .build(),
                    ),
                    FeatureConfig::RobustScaler(c) => {
                        // Read the json scaler file
                        let file = File::open(&c.scaler_data_location).expect("Failed to open scaler file at location");
                        let reader = BufReader::new(file);

                        // Deserialize into a Vec<ScalerData>
                        let scaler_data: Vec<ScalerData> =
                            serde_json::from_reader(reader).expect("Failed to deserialize scaler data");

                        let map = scaler_data
                            .into_iter()
                            .map(|data| (data.feature_id.clone(), data))
                            .collect::<HashMap<_, _>>();

                        for (k, v) in &map {
                            info!("Scaler data: {} -> {:?}", k, v);
                        }

                        let inputs = map.keys().cloned().collect();

                        Arc::new(
                            RobustScaler::builder()
                                .pipeline(pipeline.clone())
                                .insight_state(state.clone())
                                .scalers(map)
                                .input(inputs)
                                .output(c.output.clone())
                                .persist(c.persist)
                                .build(),
                        )
                    }
                    // FeatureConfig::CatBoost(c) => Arc::new(
                    //     CatBoostFeature::builder()
                    //         .insight_state(state.clone())
                    //         .model_location(c.model_location.clone())
                    //         .model_name(c.model_name.clone())
                    //         .model_version(c.model_version.clone())
                    //         .input_numerical(c.input_numerical.clone())
                    //         .input_categorical(c.input_categorical.clone())
                    //         .output(c.output.clone())
                    //         .persist(c.persist)
                    //         .build(),
                    // ),
                    // FeatureConfig::MeanVariance(c) => Arc::new(
                    //     MeanVarianceFeature::builder()
                    //         .insight_state(state.clone())
                    //         .input_expected_returns(c.input_expected_returns.clone())
                    //         .input_returns(c.input_returns.clone())
                    //         .output(c.output.clone())
                    //         .periods_returns(c.periods_returns)
                    //         .risk_aversion(c.risk_aversion)
                    //         .risk_free_rate(c.risk_free_rate)
                    //         .max_exposure_long(c.max_exposure_long)
                    //         .max_exposure_short(c.max_exposure_short)
                    //         .max_exposure_long_per_asset(c.max_exposure_long_per_asset)
                    //         .max_exposure_short_per_asset(c.max_exposure_short_per_asset)
                    //         .transaction_cost(c.transaction_cost)
                    //         .persist(c.persist)
                    //         .build(),
                    // ),
                    _ => unimplemented!(),
                };
                feature
            })
            .collect()
    }
}
