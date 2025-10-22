use std::{sync::Arc, time::Duration};

use anyhow::Result;
use futures::StreamExt;
use rust_decimal::prelude::*;
use time::macros::utc_datetime;
use tracing::{debug, info};

use arkin_core::prelude::*;
use arkin_insights::prelude::*;
use integration_tests::FeatureValidator;

pub fn build_simple_pipeline_config() -> InsightsConfig {
    InsightsConfig {
        insights_service: InsightsServiceConfig {
            pipeline: PipelineConfig {
                version: "test_pipeline".to_string(),
                warmup_steps: 60,
                state_ttl: 86400,
                min_interval: 60,
                parallel: true,
                features: vec![
                    // ========================================================================
                    // STAGE 1: Raw Trades → 1m Aggregates (per instrument)
                    // ========================================================================

                    // 1m notional volume (sum over 60 second window)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input: vec!["trade_notional".to_string()],
                        output: vec!["notional_01m".to_string()],
                        data: vec![RangeData::Window(60)],
                        method: RangeAlgo::SumAbs,
                        persist: true,
                        fill_strategy: FillStrategy::Zero, // Volume, zero if no data
                    }),
                    // 1m buy notional (positive values only)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input: vec!["trade_notional".to_string()],
                        output: vec!["notional_buy_01m".to_string()],
                        data: vec![RangeData::Window(60)],
                        method: RangeAlgo::SumAbsPositive,
                        persist: true,
                        fill_strategy: FillStrategy::Zero, // Volume, zero if no data
                    }),
                    // 1m sell notional (negative values only, absolute value)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input: vec!["trade_notional".to_string()],
                        output: vec!["notional_sell_01m".to_string()],
                        data: vec![RangeData::Window(60)],
                        method: RangeAlgo::SumAbsNegative,
                        persist: true,
                        fill_strategy: FillStrategy::Zero, // Volume, zero if no data
                    }),
                    // 1m trade count
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input: vec!["trade_notional".to_string()],
                        output: vec!["trade_count_01m".to_string()],
                        data: vec![RangeData::Window(60)],
                        method: RangeAlgo::Count,
                        persist: true,
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
                    //     persist: true,
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
                    //     persist: true,
                    //     fill_strategy: FillStrategy::ForwardFill, // Price-based, forward fill
                    // }),
                    // ========================================================================
                    // STAGE 2: 1m → Multi-timeframe Aggregates (per instrument)
                    // ========================================================================

                    // Notional for 5m, 60min (single config for all timeframes)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
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
                        persist: true,
                        fill_strategy: FillStrategy::Zero, // Volume, zero if no data
                    }),
                    // Imbalance for all timeframes
                    FeatureConfig::TwoValue(TwoValueConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input_1: vec!["notional_buy_05m".to_string(), "notional_buy_60min".to_string()],
                        input_2: vec!["notional_sell_05m".to_string(), "notional_sell_60min".to_string()],
                        output: vec!["notional_imbalance_05m".to_string(), "notional_imbalance_60min".to_string()],
                        method: TwoValueAlgo::Imbalance,
                        persist: true,
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
                    //     persist: true,
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
                    //     persist: true,
                    //     fill_strategy: FillStrategy::ForwardFill, // Price-based, forward fill
                    // }),
                ],
            },
        },
    }
}

#[tokio::test]
#[test_log::test]
async fn test_crypto_index_with_real_data() -> Result<()> {
    info!("Hello World");

    // Let's stream some agg trades from the database
    let start = utc_datetime!(2025-01-01 00:00:00);
    let end = utc_datetime!(2025-01-01 03:00:00);

    let persistence = integration_tests::init_test_persistence().await;
    persistence.refresh().await?;
    info!("Initialized test persistence.");

    // Build pipeline graph directly (no service wrapper)
    let config = build_simple_pipeline_config();
    let features = FeatureFactory::from_config(&persistence, &config.insights_service.pipeline.features).await;
    let pipeline_meta = Arc::new(
        Pipeline::builder()
            .name("test_pipeline".to_string())
            .description("test_pipeline".to_string())
            .created(start)
            .updated(start)
            .build(),
    );

    let pipeline = FeaturePipeline::new(pipeline_meta.clone(), features, &config.insights_service.pipeline);

    // Print pipeline info
    pipeline.graph().print_summary();
    pipeline.graph().print_tree();
    // pipeline
    //     .graph()
    //     .export_svg("test_crypto_index_pipeline.svg")
    //     .expect("Failed to export graph svg");

    // Query instruments using the new flexible API
    let instruments = persistence
        .query_instruments(
            &InstrumentQuery::builder()
                .venues(vec![VenueName::BinanceUsdmFutures])
                .base_asset_symbols(vec![
                    "BTC".to_string(),
                    "ETH".to_string(),
                    "SOL".to_string(),
                    "BNB".to_string(),
                    "XRP".to_string(),
                ])
                .quote_asset_symbols(vec!["USDT".to_string()])
                .synthetic(Some(false))
                .build(),
        )
        .await?;

    info!("Fetched instruments: {:?}", instruments.len());
    for inst in &instruments {
        info!("  - {} ({}/{})", inst.symbol, inst.base_asset.symbol, inst.quote_asset.symbol);
    }

    let stream = persistence
        .agg_trade_stream_range_buffered(instruments.as_slice(), start, end, 3, arkin_core::prelude::Frequency::Daily)
        .await?;

    tokio::pin!(stream);

    let interval = Duration::from_secs(60);
    let mut next_insights_tick = start + interval;

    // Pre-fetch feature IDs once (don't do this in the hot loop!)
    let trade_price_feature = persistence.get_feature_id("trade_price").await;
    let trade_quantity_feature = persistence.get_feature_id("trade_quantity").await;
    let trade_notional_feature = persistence.get_feature_id("trade_notional").await;

    // Set up feature validator for comprehensive validation of ALL features
    let mut validator = FeatureValidator::new(&persistence, instruments.clone())
        .await
        // Track raw accumulators for Stage 1 validation (raw trades → 1m aggregates)
        .track_accumulator("notional") // Total absolute notional
        .track_accumulator("buy_notional") // Buy notional (positive trades)
        .track_accumulator("sell_notional") // Sell notional (negative trades)
        // Stage 1 Validations: Raw Trades → 1m Aggregates
        .validate_raw_aggregate("notional_01m", "notional", 0.01)
        .validate_raw_aggregate("notional_buy_01m", "buy_notional", 0.01)
        .validate_raw_aggregate("notional_sell_01m", "sell_notional", 0.01)
        // Stage 2 Validations: 1m → Multi-timeframe (5m, 1h)
        .validate_interval_sum("notional_05m", "notional", 5, 0.01)
        .validate_interval_sum("notional_60min", "notional", 60, 0.01)
        .validate_interval_sum("notional_buy_05m", "buy_notional", 5, 0.01)
        .validate_interval_sum("notional_buy_60min", "buy_notional", 60, 0.01)
        .validate_interval_sum("notional_sell_05m", "sell_notional", 5, 0.01)
        .validate_interval_sum("notional_sell_60min", "sell_notional", 60, 0.01);

    // Register all features we want to validate
    validator.register_feature(&persistence, "notional_01m").await;
    validator.register_feature(&persistence, "notional_buy_01m").await;
    validator.register_feature(&persistence, "notional_sell_01m").await;
    validator.register_feature(&persistence, "notional_05m").await;
    validator.register_feature(&persistence, "notional_60min").await;
    validator.register_feature(&persistence, "notional_buy_05m").await;
    validator.register_feature(&persistence, "notional_buy_60min").await;
    validator.register_feature(&persistence, "notional_sell_05m").await;
    validator.register_feature(&persistence, "notional_sell_60min").await;

    let mut trade_count_acc: u64 = 0;
    let mut interval_notional_acc: f64 = 0.0;
    let mut interval_buy_notional_acc: f64 = 0.0;
    let mut interval_sell_notional_acc: f64 = 0.0;

    while let Some(event) = stream.next().await {
        let trade = match event {
            Event::AggTradeUpdate(t) => t,
            _ => continue,
        };

        debug!(
            "Received trade: {} @ {} for {} {} (side: {}, event_time: {})",
            trade.instrument.symbol,
            trade.price,
            trade.quantity,
            trade.instrument.quote_asset.symbol,
            trade.side,
            trade.event_time
        );

        // Insert trade data FIRST
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

        // Check if we should calculate BEFORE accumulating this trade
        // This ensures we only accumulate trades with event_time <= next_insights_tick
        if trade.event_time > next_insights_tick {
            info!("Processing tick at {}", next_insights_tick);
            // Commit state
            pipeline.commit(next_insights_tick).await;

            // Calculate insights for this tick
            let calculated_insights = pipeline.calculate(next_insights_tick, &instruments).await;
            info!(
                "Calculated {} insights for {} instruments",
                calculated_insights.len(),
                instruments.len()
            );

            // ==================================================================
            // VALIDATION: Use FeatureValidator to check aggregations
            // ==================================================================
            let report = validator
                .validate(&calculated_insights, next_insights_tick)
                .expect("Validation failed");

            if !report.is_success() {
                info!("Validation report: {} passed, {} failed", report.passed, report.failed);
                for failure in &report.failures {
                    info!("{}", failure);
                }
            }

            // Debug: For one instrument, let's trace the 5-minute calculation
            if let Some(btc) = instruments.iter().find(|i| i.symbol.contains("BTC")) {
                let notional_05m_id = persistence.get_feature_id("notional_05m").await;
                let notional_01m_id = persistence.get_feature_id("notional_01m").await;

                let notional_05m = calculated_insights
                    .iter()
                    .find(|i| i.instrument.id == btc.id && i.feature_id == notional_05m_id)
                    .map(|i| i.value);

                if let Some(calculated_5m) = notional_05m {
                    // Get the individual 1-minute calculated values to see what the pipeline summed
                    let notional_01m_values: Vec<f64> = calculated_insights
                        .iter()
                        .filter(|i| i.instrument.id == btc.id && i.feature_id == notional_01m_id)
                        .map(|i| i.value)
                        .collect();

                    info!("🔍 BTC notional_05m calculated = {:.2}", calculated_5m);
                    info!("   Current 1m values in this batch: {:?}", notional_01m_values);
                }
            }

            // Commit window for next iteration
            validator.commit_window();

            // Show sample of calculated insights (non-NaN values only)
            let valid_insights: Vec<_> = calculated_insights.iter().filter(|i| !i.value.is_nan()).take(5).collect();

            if !valid_insights.is_empty() {
                info!("Sample calculated insights:");
                for insight in valid_insights {
                    info!(
                        "  {} {} @ {}: {:.2}",
                        insight.feature_id, insight.instrument.symbol, insight.event_time, insight.value
                    );
                }
            }

            // Print accumulator totals for this interval
            info!(
                "Interval accumulators ({} to {}): trade_count={} total_notional={:.2}, buy_notional={:.2}, sell_notional={:.2}",
                next_insights_tick - interval,
                next_insights_tick,
                trade_count_acc,
                interval_notional_acc,
                interval_buy_notional_acc,
                interval_sell_notional_acc,
            );
            // Reset interval accumulators
            trade_count_acc = 0;
            interval_notional_acc = 0.0;
            interval_buy_notional_acc = 0.0;
            interval_sell_notional_acc = 0.0;

            // Advance to next tick
            next_insights_tick += interval;
        }

        // Accumulate this trade for the next interval
        // Only accumulate if event_time <= next_insights_tick (which is now true after potential advancement)
        let notional = (trade.price * trade.quantity).to_f64().unwrap_or(f64::NAN) * f64::from(trade.side);

        trade_count_acc += 1;
        // Total notional (absolute value)
        let notional_abs = notional.abs();
        validator.accumulate(&trade.instrument, "notional", notional_abs);
        interval_notional_acc += notional_abs;

        // Buy/Sell notional (split by side)
        if notional > 0.0 {
            validator.accumulate(&trade.instrument, "buy_notional", notional.abs());
            interval_buy_notional_acc += notional.abs();
        } else if notional < 0.0 {
            validator.accumulate(&trade.instrument, "sell_notional", notional.abs());
            interval_sell_notional_acc += notional.abs();
        }
    }

    Ok(())
}
