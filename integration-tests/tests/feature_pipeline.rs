use std::{sync::Arc, time::Duration, vec};

use anyhow::Result;
use futures::StreamExt;
use rust_decimal::prelude::*;
use time::macros::utc_datetime;
use tracing::info;

use arkin_core::prelude::*;
use arkin_insights::prelude::*;
use integration_tests::FeatureValidator;

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
                    base_asset: vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()],
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

#[tokio::test]
#[test_log::test]
async fn test_crypto_index_with_real_data() -> Result<()> {
    let start = utc_datetime!(2025-01-01 00:00:00);
    let end = utc_datetime!(2025-01-07 00:00:00); // Extended to 3 hours to see calculations after warmup

    let persistence = integration_tests::init_test_persistence().await;
    persistence.refresh().await?;

    // Build pipeline
    let config = build_simple_pipeline_config();
    let features = FeatureFactory::from_config(&persistence, &config.insights_service.pipeline).await;
    let pipeline_meta = Arc::new(
        Pipeline::builder()
            .name("test_pipeline".to_string())
            .description("test_pipeline".to_string())
            .created(start)
            .updated(start)
            .build(),
    );
    let pipeline = FeaturePipeline::new(pipeline_meta.clone(), features, &config.insights_service.pipeline);

    // Get synthetic instruments from the pipeline
    let instruments = pipeline.real_instruments();
    let synthetic_instruments = pipeline.synthetic_instruments();

    info!("Found {} real instruments from pipeline", instruments.len());
    info!("Found {} synthetic instruments from pipeline", synthetic_instruments.len());

    let stream = persistence
        .agg_trade_stream_range_buffered(instruments.as_slice(), start, end, 3, arkin_core::prelude::Frequency::Daily)
        .await?;

    tokio::pin!(stream);

    let interval = Duration::from_secs(60);
    let mut next_insights_tick = start + interval;

    // Pre-fetch feature IDs
    let trade_price_feature = persistence.get_feature_id("trade_price").await;
    let trade_quantity_feature = persistence.get_feature_id("trade_quantity").await;
    let trade_notional_feature = persistence.get_feature_id("trade_notional").await;

    // Set up feature validator
    let mut validator = FeatureValidator::new(&persistence, instruments.clone())
        .await
        .add_instruments(synthetic_instruments.clone())
        .track_accumulator("notional")
        .track_accumulator("buy_notional")
        .track_accumulator("sell_notional")
        .track_accumulator("trade_count")
        .validate_raw_aggregate("notional_01m", "notional", 0.01)
        .validate_raw_aggregate("notional_buy_01m", "buy_notional", 0.01)
        .validate_raw_aggregate("notional_sell_01m", "sell_notional", 0.01)
        .validate_raw_aggregate("trade_count_01m", "trade_count", 0.01)
        .validate_interval_sum("notional_05m", "notional", 5, 0.01)
        .validate_interval_sum("notional_60min", "notional", 60, 0.01)
        .validate_interval_sum("notional_buy_05m", "buy_notional", 5, 0.01)
        .validate_interval_sum("notional_buy_60min", "buy_notional", 60, 0.01)
        .validate_interval_sum("notional_sell_05m", "sell_notional", 5, 0.01)
        .validate_interval_sum("notional_sell_60min", "sell_notional", 60, 0.01)
        .validate_computed_from_intervals(
            "notional_imbalance_05m",
            "buy_notional",
            "sell_notional",
            5,
            |buy_values, sell_values| {
                let buy_sum: f64 = buy_values.iter().sum();
                let sell_sum: f64 = sell_values.iter().sum();
                let total = buy_sum + sell_sum;
                if total > 0.0 {
                    Some((buy_sum - sell_sum) / total)
                } else {
                    Some(0.0)
                }
            },
            0.01,
        )
        .validate_computed_from_intervals(
            "notional_imbalance_60min",
            "buy_notional",
            "sell_notional",
            60,
            |buy_values, sell_values| {
                let buy_sum: f64 = buy_values.iter().sum();
                let sell_sum: f64 = sell_values.iter().sum();
                let total = buy_sum + sell_sum;
                if total > 0.0 {
                    Some((buy_sum - sell_sum) / total)
                } else {
                    Some(0.0)
                }
            },
            0.01,
        );

    validator.register_feature(&persistence, "notional_01m").await;
    validator.register_feature(&persistence, "notional_buy_01m").await;
    validator.register_feature(&persistence, "notional_sell_01m").await;
    validator.register_feature(&persistence, "trade_count_01m").await;
    validator.register_feature(&persistence, "notional_05m").await;
    validator.register_feature(&persistence, "notional_60min").await;
    validator.register_feature(&persistence, "notional_buy_05m").await;
    validator.register_feature(&persistence, "notional_buy_60min").await;
    validator.register_feature(&persistence, "notional_sell_05m").await;
    validator.register_feature(&persistence, "notional_sell_60min").await;
    validator.register_feature(&persistence, "notional_imbalance_05m").await;
    validator.register_feature(&persistence, "notional_imbalance_60min").await;

    // Add validation for synthetic instruments
    // Group instruments by base asset for grouped synthetic validation
    for base_asset in ["BTC", "ETH", "SOL", "BNB", "XRP"] {
        // Find the synthetic instrument for this base asset
        let synthetic = synthetic_instruments
            .iter()
            .find(|i| {
                i.symbol.contains(&format!("syn-{}", base_asset.to_lowercase()))
                    && !i.symbol.contains("perpetual")
                    && !i.symbol.contains("spot")
            })
            .cloned();

        if let Some(syn_inst) = synthetic {
            // Find all real instruments with this base asset
            let source_instruments: Vec<Arc<Instrument>> = instruments
                .iter()
                .filter(|i| i.base_asset.symbol == base_asset)
                .cloned()
                .collect();

            info!(
                "Validating {} from {} source instruments",
                syn_inst.symbol,
                source_instruments.len()
            );

            // Validate that grouped features = sum of real instrument features
            validator = validator
                .validate_synthetic_sum(
                    syn_inst.clone(),
                    "grouped_notional_01m",
                    "notional_01m",
                    source_instruments.clone(),
                    0.01,
                )
                .validate_synthetic_sum(
                    syn_inst.clone(),
                    "grouped_notional_05m",
                    "notional_05m",
                    source_instruments.clone(),
                    0.01,
                )
                .validate_synthetic_sum(
                    syn_inst.clone(),
                    "grouped_notional_60min",
                    "notional_60min",
                    source_instruments,
                    0.01,
                );
        }
    }

    validator.register_feature(&persistence, "grouped_notional_01m").await;
    validator.register_feature(&persistence, "grouped_notional_05m").await;
    validator.register_feature(&persistence, "grouped_notional_60min").await;

    // Add validation for index features
    // Find the index instruments (they use the feature name as their symbol)
    let index_01m = synthetic_instruments
        .iter()
        .find(|i| i.symbol == "index_notional_01m@index")
        .cloned();
    let index_05m = synthetic_instruments
        .iter()
        .find(|i| i.symbol == "index_notional_05m@index")
        .cloned();
    let index_60min = synthetic_instruments
        .iter()
        .find(|i| i.symbol == "index_notional_60min@index")
        .cloned();

    // Get all base grouped synthetic instruments (not type-specific, not index)
    let base_synthetics: Vec<Arc<Instrument>> = synthetic_instruments
        .iter()
        .filter(|i| {
            i.symbol.starts_with("syn-")
                && i.symbol.ends_with("@index")
                && !i.symbol.contains("perpetual")
                && !i.symbol.contains("spot")
        })
        .cloned()
        .collect();

    info!(
        "Found {} base synthetic instruments for index validation",
        base_synthetics.len()
    );

    if let Some(index_inst) = index_01m {
        info!(
            "Validating {} from {} base synthetic instruments",
            index_inst.symbol,
            base_synthetics.len()
        );
        validator = validator.validate_synthetic_sum(
            index_inst,
            "index_notional_01m",
            "grouped_notional_01m",
            base_synthetics.clone(),
            0.01,
        );
    }

    if let Some(index_inst) = index_05m {
        validator = validator.validate_synthetic_sum(
            index_inst,
            "index_notional_05m",
            "grouped_notional_05m",
            base_synthetics.clone(),
            0.01,
        );
    }

    if let Some(index_inst) = index_60min {
        validator = validator.validate_synthetic_sum(
            index_inst,
            "index_notional_60min",
            "grouped_notional_60min",
            base_synthetics,
            0.01,
        );
    }

    validator.register_feature(&persistence, "index_notional_01m").await;
    validator.register_feature(&persistence, "index_notional_05m").await;
    validator.register_feature(&persistence, "index_notional_60min").await;

    while let Some(event) = stream.next().await {
        let trade = match event {
            Event::AggTradeUpdate(t) => t,
            _ => continue,
        };

        // Insert trade data
        let insights = vec![
            Insight::builder()
                .event_time(trade.event_time)
                .instrument(trade.instrument.clone())
                .feature_id(trade_price_feature.clone())
                .value(trade.price.to_f64().unwrap_or(f64::NAN))
                .insight_type(InsightType::Raw)
                .build()
                .into(),
            Insight::builder()
                .event_time(trade.event_time)
                .instrument(trade.instrument.clone())
                .feature_id(trade_quantity_feature.clone())
                .value(trade.quantity.to_f64().unwrap_or(f64::NAN) * f64::from(trade.side))
                .insight_type(InsightType::Raw)
                .build()
                .into(),
            Insight::builder()
                .event_time(trade.event_time)
                .instrument(trade.instrument.clone())
                .feature_id(trade_notional_feature.clone())
                .value((trade.price * trade.quantity).to_f64().unwrap_or(f64::NAN) * f64::from(trade.side))
                .insight_type(InsightType::Raw)
                .build()
                .into(),
        ];
        pipeline.insert_batch(&insights).await;

        // Check if we should calculate
        if trade.event_time > next_insights_tick {
            // Commit state and calculate insights
            pipeline.commit(next_insights_tick).await;
            let calculated_insights = pipeline.calculate(next_insights_tick).await;

            if !calculated_insights.is_empty() {
                info!("After warmup: calculated {} insights", calculated_insights.len());
            }

            // Validate aggregations
            let report = validator
                .validate(&calculated_insights, next_insights_tick)
                .expect("Validation failed");

            if !report.is_success() {
                info!("Validation report: {} passed, {} failed", report.passed, report.failed);
                for failure in &report.failures {
                    info!("{}", failure);
                }
            }

            validator.commit_window();
            next_insights_tick += interval;
        }

        // Accumulate this trade for the next interval
        let notional = (trade.price * trade.quantity).to_f64().unwrap_or(f64::NAN) * f64::from(trade.side);

        // Total notional (absolute value)
        let notional_abs = notional.abs();
        validator.accumulate(&trade.instrument, "notional", notional_abs);

        // Buy/Sell notional (split by side)
        if notional > 0.0 {
            validator.accumulate(&trade.instrument, "buy_notional", notional.abs());
        } else if notional < 0.0 {
            validator.accumulate(&trade.instrument, "sell_notional", notional.abs());
        }

        // Trade count
        validator.accumulate(&trade.instrument, "trade_count", 1.0);
    }

    info!("✅ Validation complete - all features validated successfully");

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_synthetic_instrument_generation() -> Result<()> {
    info!("Testing synthetic instrument generation");

    let persistence = integration_tests::init_test_persistence().await;
    persistence.refresh().await?;
    info!("Initialized test persistence.");

    // Use the same config as the main test
    let config = build_simple_pipeline_config();

    // Create features and pipeline
    let features = FeatureFactory::from_config(&persistence, &config.insights_service.pipeline).await;
    let pipeline_meta = Arc::new(
        Pipeline::builder()
            .name("test-synthetic".to_string())
            .description("Test synthetic generation".to_string())
            .created(utc_datetime!(2024-01-01 00:00:00))
            .updated(utc_datetime!(2024-01-01 00:00:00))
            .build(),
    );
    let pipeline = FeaturePipeline::new(pipeline_meta, features, &config.insights_service.pipeline);

    // Query instruments from pipeline
    let real_instruments = pipeline.real_instruments();
    let synthetic_instruments = pipeline.synthetic_instruments();

    // Verify results
    info!("Real instruments: {}", real_instruments.len());
    info!("Synthetic instruments: {}", synthetic_instruments.len());
    info!("Features created: {}", pipeline.graph().node_count());

    assert_eq!(
        real_instruments.len(),
        20,
        "Expected 20 real instruments (5 bases × 2 quotes × 2 types)"
    );
    assert_eq!(
        synthetic_instruments.len(),
        21,
        "Expected 21 synthetic instruments (11 grouped base + 10 grouped type)"
    );
    assert_eq!(
        pipeline.graph().node_count(),
        22,
        "Expected 22 features from main pipeline (10 configs, some create multiple features due to multiple outputs)"
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_crypto_index_with_real_data_no_validation() -> Result<()> {
    let start = utc_datetime!(2025-01-01 00:00:00);
    let end = utc_datetime!(2025-01-01 03:00:00); // Extended to 3 hours to see calculations after warmup

    let persistence = integration_tests::init_test_persistence().await;
    persistence.refresh().await?;

    // Build pipeline
    let config = build_simple_pipeline_config();
    let features = FeatureFactory::from_config(&persistence, &config.insights_service.pipeline).await;
    let pipeline_meta = Arc::new(
        Pipeline::builder()
            .name("test_pipeline".to_string())
            .description("test_pipeline".to_string())
            .created(start)
            .updated(start)
            .build(),
    );
    let pipeline = FeaturePipeline::new(pipeline_meta.clone(), features, &config.insights_service.pipeline);
    pipeline
        .graph()
        .export_svg("./pipeline_v2.0.0.svg")
        .expect("Failed to export pipeline graph");

    // Print the features in the pipeline
    pipeline.graph().print_summary();
    pipeline.graph().print_tree();

    // Get synthetic instruments from the pipeline
    let instruments = pipeline.real_instruments();
    let synthetic_instruments = pipeline.synthetic_instruments();

    info!("Found {} real instruments from pipeline", instruments.len());
    info!("Found {} synthetic instruments from pipeline", synthetic_instruments.len());

    let stream = persistence
        .agg_trade_stream_range_buffered(instruments.as_slice(), start, end, 3, arkin_core::prelude::Frequency::Daily)
        .await?;

    tokio::pin!(stream);

    let interval = Duration::from_secs(60);
    let mut next_insights_tick = start + interval;

    // Pre-fetch feature IDs
    let trade_price_feature = persistence.get_feature_id("trade_price").await;
    let trade_quantity_feature = persistence.get_feature_id("trade_quantity").await;
    let trade_notional_feature = persistence.get_feature_id("trade_notional").await;

    let mut total_trades = 0;
    let mut total_calculated_insights = 0;

    while let Some(event) = stream.next().await {
        let trade = match event {
            Event::AggTradeUpdate(t) => t,
            _ => continue,
        };

        total_trades += 1;

        // Insert trade data
        let insights = vec![
            Insight::builder()
                .event_time(trade.event_time)
                .instrument(trade.instrument.clone())
                .feature_id(trade_price_feature.clone())
                .value(trade.price.to_f64().unwrap_or(f64::NAN))
                .insight_type(InsightType::Raw)
                .build()
                .into(),
            Insight::builder()
                .event_time(trade.event_time)
                .instrument(trade.instrument.clone())
                .feature_id(trade_quantity_feature.clone())
                .value(trade.quantity.to_f64().unwrap_or(f64::NAN) * f64::from(trade.side))
                .insight_type(InsightType::Raw)
                .build()
                .into(),
            Insight::builder()
                .event_time(trade.event_time)
                .instrument(trade.instrument.clone())
                .feature_id(trade_notional_feature.clone())
                .value((trade.price * trade.quantity).to_f64().unwrap_or(f64::NAN) * f64::from(trade.side))
                .insight_type(InsightType::Raw)
                .build()
                .into(),
        ];
        pipeline.insert_batch(&insights).await;

        // Check if we should calculate
        if trade.event_time > next_insights_tick {
            // Commit state and calculate insights
            pipeline.commit(next_insights_tick).await;
            let calculated_insights = pipeline.calculate(next_insights_tick).await;

            if !calculated_insights.is_empty() {
                total_calculated_insights += calculated_insights.len();
                info!(
                    "After warmup: calculated {} insights at {}",
                    calculated_insights.len(),
                    next_insights_tick
                );
            }

            next_insights_tick += interval;
        }
    }

    info!(
        "✅ Pipeline execution complete - processed {} trades and calculated {} insights total",
        total_trades, total_calculated_insights
    );

    Ok(())
}
