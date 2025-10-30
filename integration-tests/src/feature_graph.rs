use arkin_core::prelude::*;
use arkin_insights::prelude::*;

pub fn build_simple_pipeline_config() -> InsightsConfig {
    InsightsConfig {
        insights_service: InsightsServiceConfig {
            pipeline: PipelineConfig {
                version: "test_pipeline".to_string(),
                reference_currency: "USD".to_string(),
                warmup_steps: 60,
                state_ttl: 86400,
                min_interval: 60,
                parallel: true,
                // Global filter applied to all features
                instrument_filter: InstrumentFilter {
                    base_asset: vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string(), "XRP".to_string()],
                    quote_asset: vec!["USDT".to_string(), "USDC".to_string()],
                    venue: vec![VenueName::BinanceUsdmFutures, VenueName::BinanceSpot],
                    instrument_type: vec![InstrumentType::Perpetual, InstrumentType::Spot],
                    ..Default::default()
                },
                features: vec![
                    // ========================================================================
                    // STAGE 1: Raw Trades → 1m Aggregates (per instrument)
                    // ========================================================================

                    // 1m notional volume (sum over 60 second window)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument {
                            filter: InstrumentFilter::default(), // Uses global filter
                        },
                        input: vec!["trade_notional".to_string()],
                        output: vec!["notional_01m".to_string()],
                        data: vec![RangeData::Window(60)],
                        method: RangeAlgo::AbsSum,
                        fill_strategy: FillStrategy::Zero, // Volume, zero if no data
                    }),
                    // 1m buy notional (positive values only)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument {
                            filter: InstrumentFilter::default(), // Uses global filter
                        },
                        input: vec!["trade_notional".to_string()],
                        output: vec!["notional_buy_01m".to_string()],
                        data: vec![RangeData::Window(60)],
                        method: RangeAlgo::SumAbsPositive,
                        fill_strategy: FillStrategy::Zero, // Volume, zero if no data
                    }),
                    // 1m sell notional (negative values only, absolute value)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument {
                            filter: InstrumentFilter::default(), // Uses global filter
                        },
                        input: vec!["trade_notional".to_string()],
                        output: vec!["notional_sell_01m".to_string()],
                        data: vec![RangeData::Window(60)],
                        method: RangeAlgo::SumAbsNegative,
                        fill_strategy: FillStrategy::Zero, // Volume, zero if no data
                    }),
                    // 1m trade count
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument {
                            filter: InstrumentFilter::default(), // Uses global filter
                        },
                        input: vec!["trade_notional".to_string()],
                        output: vec!["trade_count_01m".to_string()],
                        data: vec![RangeData::Window(60)],
                        method: RangeAlgo::Count,
                        fill_strategy: FillStrategy::Zero, // Count, zero if no data
                    }),
                    // // 1m mean price (mean over 60 second window)
                    // FeatureConfig::Range(RangeConfig {
                    //     aggregation_type: AggregationType::Instrument,
                    //     filter: InstrumentFilter::default(),
                    //     group_by: GroupBy::default(),
                    //     input: vec!["trade_price".to_string()],
                    //     output: vec!["price_01m".to_string()],
                    //     data: vec![RangeData::Window(60)],
                    //     method: RangeAlgo::Mean,
                    //     fill_strategy: FillStrategy::ForwardFill, // Price, forward fill
                    // }),
                    // // 1m VWAP (volume-weighted average price over 60 second window)
                    // FeatureConfig::DualRange(DualRangeConfig {
                    //     aggregation_type: AggregationType::Instrument,
                    //     filter: InstrumentFilter::default(),
                    //     group_by: GroupBy::default(),
                    //     input_1: vec!["trade_price".to_string()],
                    //     input_2: vec!["trade_notional".to_string()], // Use notional as weight
                    //     output: vec!["vwap_01m".to_string()],
                    //     data: vec![RangeData::Window(60)],
                    //     method: DualRangeAlgo::WeightedMean,
                    //     fill_strategy: FillStrategy::ForwardFill, // Price-based, forward fill
                    // }),
                    // ========================================================================
                    // STAGE 2: 1m → Multi-timeframe Aggregates (per instrument)
                    // ========================================================================

                    // Notional for 5m, 60min (single config for all timeframes)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument {
                            filter: InstrumentFilter::default(), // Uses global filter
                        },
                        input: vec![
                            "notional_01m".to_string(),
                            "notional_buy_01m".to_string(),
                            "notional_sell_01m".to_string(),
                            "notional_01m".to_string(),
                            "notional_buy_01m".to_string(),
                            "notional_sell_01m".to_string(),
                        ],
                        output: vec![
                            "notional_05m".to_string(),
                            "notional_buy_05m".to_string(),
                            "notional_sell_05m".to_string(),
                            "notional_60min".to_string(),
                            "notional_buy_60min".to_string(),
                            "notional_sell_60min".to_string(),
                        ],
                        data: vec![
                            RangeData::Interval(5),
                            RangeData::Interval(5),
                            RangeData::Interval(5),
                            RangeData::Interval(60),
                            RangeData::Interval(60),
                            RangeData::Interval(60),
                        ],
                        method: RangeAlgo::Sum,
                        fill_strategy: FillStrategy::Zero, // Volume, zero if no data
                    }),
                    // Imbalance for all timeframes
                    FeatureConfig::TwoValue(TwoValueConfig {
                        aggregation_type: AggregationType::Instrument {
                            filter: InstrumentFilter::default(), // Uses global filter
                        },
                        input_filter_1: None,
                        input_filter_2: None,
                        input_1: vec!["notional_buy_05m".to_string(), "notional_buy_60min".to_string()],
                        input_2: vec!["notional_sell_05m".to_string(), "notional_sell_60min".to_string()],
                        output: vec!["notional_imbalance_05m".to_string(), "notional_imbalance_60min".to_string()],
                        method: TwoValueAlgo::Imbalance,
                        fill_strategy: FillStrategy::Zero, // Volume imbalance, zero if no data
                    }),
                    // // Price for all timeframes (only need 5m for now)
                    // FeatureConfig::Range(RangeConfig {
                    //     aggregation_type: AggregationType::Instrument,
                    //     filter: InstrumentFilter::default(),
                    //     group_by: GroupBy::default(),
                    //     input: vec!["price_01m".to_string()],
                    //     output: vec!["price_05m".to_string()],
                    //     data: vec![RangeData::Interval(5)],
                    //     method: RangeAlgo::Mean,
                    //     fill_strategy: FillStrategy::ForwardFill, // Price, forward fill
                    // }),
                    // // VWAP for all timeframes (5m, 60min)
                    // FeatureConfig::DualRange(DualRangeConfig {
                    //     aggregation_type: AggregationType::Instrument,
                    //     filter: InstrumentFilter::default(),
                    //     group_by: GroupBy::default(),
                    //     input_1: vec!["vwap_01m".to_string(), "vwap_01m".to_string()],
                    //     input_2: vec!["notional_01m".to_string(), "notional_01m".to_string()],
                    //     output: vec!["vwap_05m".to_string(), "vwap_60min".to_string()],
                    //     data: vec![RangeData::Interval(5), RangeData::Interval(60)],
                    //     method: DualRangeAlgo::WeightedMean,
                    //     fill_strategy: FillStrategy::ForwardFill, // Price-based, forward fill
                    // }),
                    // ========================================================================
                    // STAGE 3: 1m Aggregates to combined synthetic index
                    // ========================================================================

                    // Grouped synthetics: Create syn-btc-usd@index, syn-eth-usd@index, etc.
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Grouped {
                            filter: InstrumentFilter::default(), // Uses global filter
                            group_by: GroupBy {
                                quote_asset: vec!["USDT".to_string(), "USDC".to_string()],
                                instrument_type: false, // Don't group by type
                                venue: None,            // Global synthetics
                            },
                        },
                        input: vec![
                            "notional_01m".to_string(),
                            "notional_05m".to_string(),
                            "notional_60min".to_string(),
                        ],
                        output: vec![
                            "grouped_notional_01m".to_string(),
                            "grouped_notional_05m".to_string(),
                            "grouped_notional_60min".to_string(),
                        ],
                        data: vec![RangeData::Interval(1), RangeData::Interval(1), RangeData::Interval(1)],
                        method: RangeAlgo::Sum,
                        fill_strategy: FillStrategy::Zero,
                    }),
                    // // Grouped by instrument type: Create syn-perpetual-btc-usd@index, syn-spot-btc-usd@index, etc.
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Grouped {
                            filter: InstrumentFilter::default(), // Uses global filter
                            group_by: GroupBy {
                                quote_asset: vec!["USDT".to_string(), "USDC".to_string()],
                                instrument_type: true, // Group by instrument type too
                                venue: None,           // Global synthetics
                            },
                        },
                        input: vec!["notional_01m".to_string()],
                        output: vec!["grouped_type_notional_01m".to_string()],
                        data: vec![RangeData::Interval(1)],
                        method: RangeAlgo::Sum,
                        fill_strategy: FillStrategy::Zero,
                    }),
                    // // Market index: Create index-global-usd from all base synthetics
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Index {
                            filter: InstrumentFilter {
                                synthetic: Some(true),
                                ..Default::default()
                            },
                        },
                        input: vec![
                            "grouped_notional_01m".to_string(),
                            "grouped_notional_05m".to_string(),
                            "grouped_notional_60min".to_string(),
                        ],
                        output: vec![
                            "index_notional_01m".to_string(),
                            "index_notional_05m".to_string(),
                            "index_notional_60min".to_string(),
                        ],
                        data: vec![RangeData::Interval(1), RangeData::Interval(1), RangeData::Interval(1)],
                        method: RangeAlgo::Sum,
                        fill_strategy: FillStrategy::Zero,
                    }),
                    // Market index: Create index-global-usd from all base synthetics
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Index {
                            filter: InstrumentFilter {
                                synthetic: Some(true),
                                ..Default::default()
                            },
                        },
                        input: vec![
                            "grouped_notional_01m".to_string(),
                            "grouped_notional_05m".to_string(),
                            "grouped_notional_60min".to_string(),
                        ],
                        output: vec![
                            "index_notional_01m".to_string(),
                            "index_notional_05m".to_string(),
                            "index_notional_60min".to_string(),
                        ],
                        data: vec![RangeData::Interval(1), RangeData::Interval(1), RangeData::Interval(1)],
                        method: RangeAlgo::Sum,
                        fill_strategy: FillStrategy::Zero,
                    }),
                ],
            },
        },
    }
}
