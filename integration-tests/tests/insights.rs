use std::{sync::Arc, time::Duration};

use anyhow::Result;
use futures::StreamExt;
use rust_decimal::prelude::*;
use time::macros::utc_datetime;
use tracing::info;

use arkin_core::prelude::*;
use arkin_insights::prelude::*;

pub fn build_simple_pipeline_config() -> InsightsConfig {
    InsightsConfig {
        insights_service: InsightsServiceConfig {
            warmup_steps: 10,   // e.g., 10 ticks
            state_ttl: 3600,    // 1 hour
            frequency_secs: 60, // 1 minute
            pipeline: PipelineConfig {
                name: "crypto_market_index".to_string(),
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
                    }),
                    // 1m mean price (mean over 60 second window)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input: vec!["trade_price".to_string()],
                        output: vec!["price_01m".to_string()],
                        data: vec![RangeData::Window(60)],
                        method: RangeAlgo::Mean,
                        persist: true,
                    }),
                    // 1m VWAP (volume-weighted average price over 60 second window)
                    FeatureConfig::DualRange(DualRangeConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input_1: vec!["trade_price".to_string()],
                        input_2: vec!["trade_notional".to_string()], // Use notional as weight
                        output: vec!["vwap_01m".to_string()],
                        data: vec![RangeData::Window(60)],
                        method: DualRangeAlgo::WeightedMean,
                        persist: true,
                    }),
                    // ========================================================================
                    // STAGE 2: 1m → Multi-timeframe Aggregates (per instrument)
                    // ========================================================================

                    // Notional for 5m, 01h, 04h (single config for all timeframes)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input: vec![
                            "notional_01m".to_string(),
                            "notional_01m".to_string(),
                            "notional_01m".to_string(),
                        ],
                        output: vec![
                            "notional_05m".to_string(),
                            "notional_01h".to_string(),
                            "notional_04h".to_string(),
                        ],
                        data: vec![RangeData::Interval(5), RangeData::Interval(60), RangeData::Interval(240)],
                        method: RangeAlgo::Sum,
                        persist: true,
                    }),
                    // Buy/Sell notional for all timeframes
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input: vec![
                            "notional_buy_01m".to_string(),
                            "notional_buy_01m".to_string(),
                            "notional_buy_01m".to_string(),
                            "notional_sell_01m".to_string(),
                            "notional_sell_01m".to_string(),
                            "notional_sell_01m".to_string(),
                        ],
                        output: vec![
                            "notional_buy_05m".to_string(),
                            "notional_buy_01h".to_string(),
                            "notional_buy_04h".to_string(),
                            "notional_sell_05m".to_string(),
                            "notional_sell_01h".to_string(),
                            "notional_sell_04h".to_string(),
                        ],
                        data: vec![
                            RangeData::Interval(5),
                            RangeData::Interval(60),
                            RangeData::Interval(240),
                            RangeData::Interval(5),
                            RangeData::Interval(60),
                            RangeData::Interval(240),
                        ],
                        method: RangeAlgo::Sum,
                        persist: true,
                    }),
                    // Imbalance for all timeframes
                    FeatureConfig::TwoValue(TwoValueConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input_1: vec![
                            "notional_buy_05m".to_string(),
                            "notional_buy_01h".to_string(),
                            "notional_buy_04h".to_string(),
                        ],
                        input_2: vec![
                            "notional_sell_05m".to_string(),
                            "notional_sell_01h".to_string(),
                            "notional_sell_04h".to_string(),
                        ],
                        output: vec![
                            "notional_imbalance_05m".to_string(),
                            "notional_imbalance_01h".to_string(),
                            "notional_imbalance_04h".to_string(),
                        ],
                        horizons: vec![RangeData::Interval(1), RangeData::Interval(1), RangeData::Interval(1)],
                        method: TwoValueAlgo::Imbalance,
                        persist: true,
                    }),
                    // Price for all timeframes (only need 5m for now)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input: vec!["price_01m".to_string()],
                        output: vec!["price_05m".to_string()],
                        data: vec![RangeData::Interval(5)],
                        method: RangeAlgo::Mean,
                        persist: true,
                    }),
                    // VWAP for all timeframes (5m, 01h, 04h)
                    FeatureConfig::DualRange(DualRangeConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter::default(),
                        group_by: GroupBy::default(),
                        input_1: vec!["vwap_01m".to_string(), "vwap_01m".to_string(), "vwap_01m".to_string()],
                        input_2: vec![
                            "notional_01m".to_string(),
                            "notional_01m".to_string(),
                            "notional_01m".to_string(),
                        ],
                        output: vec!["vwap_05m".to_string(), "vwap_01h".to_string(), "vwap_04h".to_string()],
                        data: vec![RangeData::Interval(5), RangeData::Interval(60), RangeData::Interval(240)],
                        method: DualRangeAlgo::WeightedMean,
                        persist: true,
                    }),
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
    let end = utc_datetime!(2025-01-02 00:00:00);

    let persistence = integration_tests::init_test_persistence().await;
    info!("Initialized test persistence.");

    // Build pipeline graph directly (no service wrapper)
    let config = build_simple_pipeline_config();
    let features = FeatureFactory::from_config(&persistence, &config.insights_service.pipeline.features).await;
    let pipeline = Arc::new(
        Pipeline::builder()
            .name("test_pipeline".to_string())
            .description("test_pipeline".to_string())
            .created(start)
            .updated(start)
            .build(),
    );
    let graph = PipelineGraph::new(features);
    let state = InsightsState::new(config.insights_service.state_ttl);

    let venue = persistence.get_venue_by_name(&VenueName::BinanceSpot).await?;
    let instruments = persistence.get_instruments_by_venue(&venue).await?;

    // Filter instruments for base asset BTC, ETH, SOL, BNB, XRP and quote asset USDT, USDC
    let instruments: Vec<_> = instruments
        .into_iter()
        .filter(|inst| {
            matches!(
                inst.base_asset.symbol.to_uppercase().as_str(),
                "BTC" | "ETH" | "SOL" | "BNB" | "XRP"
            )
        })
        .filter(|inst| matches!(inst.quote_asset.symbol.to_uppercase().as_str(), "USDT"))
        .collect();

    info!("Fetched instruments: {:?}", instruments.len());
    for inst in &instruments {
        info!("  - {} ({}/{})", inst.symbol, inst.base_asset.symbol, inst.quote_asset.symbol);
    }

    let stream = persistence
        .agg_trade_stream_range_buffered(instruments.as_slice(), start, end, 1, arkin_core::prelude::Frequency::Daily)
        .await?;

    tokio::pin!(stream);

    let interval = Duration::from_secs(60);
    let mut next_insights_tick = start + interval;

    // Pre-fetch feature IDs once (don't do this in the hot loop!)
    let trade_price_feature = persistence.get_feature_id("trade_price").await;
    let trade_quantity_feature = persistence.get_feature_id("trade_quantity").await;
    let trade_notional_feature = persistence.get_feature_id("trade_notional").await;

    while let Some(event) = stream.next().await {
        let trade = match event {
            Event::AggTradeUpdate(t) => t,
            _ => continue,
        };

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
        state.insert_batch_buffered(&insights).await;

        // THEN check if we should calculate
        if trade.event_time >= next_insights_tick {
            info!("Processing tick at {}", next_insights_tick);
            // Commit state
            state.commit(next_insights_tick).await;

            // Calculate insights for this tick
            let calculated_insights = graph.calculate(&state, &pipeline, next_insights_tick, &instruments);
            info!(
                "Calculated {} insights for {} instruments",
                calculated_insights.len(),
                instruments.len()
            );

            // Show sample of calculated insights (non-NaN values only)
            let valid_insights: Vec<_> = calculated_insights.iter().filter(|i| !i.value.is_nan()).take(5).collect();

            if !valid_insights.is_empty() {
                info!("Sample calculated insights:");
                for insight in valid_insights {
                    info!("  {} @ {}: {:.2}", insight.instrument.symbol, insight.event_time, insight.value);
                }
            }

            // Advance to next tick
            next_insights_tick += interval;
        }
    }

    Ok(())
}
