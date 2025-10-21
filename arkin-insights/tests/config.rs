use std::sync::Arc;

use arkin_core::{prelude::MockPersistence, InstrumentType};
use arkin_insights::{prelude::*, FeatureFactory, FeatureGraph};
use tracing::info;

#[tokio::test]
#[test_log::test]
async fn load_config() {
    let mock_persistence: Arc<dyn arkin_core::PersistenceReader> = MockPersistence::new();
    let config = InsightsConfig {
        insights_service: InsightsServiceConfig {
            pipeline: PipelineConfig {
                version: "test_pipeline".to_string(),
                warmup_steps: 10,
                state_ttl: 3600,
                frequency_secs: 60,
                features: vec![FeatureConfig::Lag(LagConfig {
                    aggregation_type: AggregationType::Instrument,
                    filter: InstrumentFilter::default(),
                    group_by: GroupBy::default(),
                    input: vec![
                        "trade_price_5m".to_string(),
                        "trade_price_4h".to_string(),
                        "trade_price_24h".to_string(),
                    ],
                    output: vec![
                        "log_change_5m".to_string(),
                        "log_change_4h".to_string(),
                        "log_change_24h".to_string(),
                    ],
                    lag: vec![300, 14400, 86400],
                    method: LagAlgo::LogChange,
                    persist: true,
                })],
            },
        },
    };

    // Create Feature Factory
    let features = FeatureFactory::from_config(&mock_persistence, &config.insights_service.pipeline.features).await;
    assert_eq!(features.len(), 3);

    // print features
    info!("Features created from config:");
    for feature in features {
        info!("{:?}", feature);
    }
}

#[tokio::test]
#[test_log::test]
async fn load_config_range() {
    let mock_persistence: Arc<dyn arkin_core::PersistenceReader> = MockPersistence::new();
    let config = InsightsConfig {
        insights_service: InsightsServiceConfig {
            pipeline: PipelineConfig {
                version: "test_pipeline".to_string(),
                warmup_steps: 10,
                state_ttl: 3600,
                frequency_secs: 60,
                features: vec![FeatureConfig::Range(RangeConfig {
                    aggregation_type: AggregationType::Instrument,
                    filter: InstrumentFilter::default(),
                    group_by: GroupBy::default(),
                    input: vec!["trade_price_5m".to_string(), "trade_price_1h".to_string()],
                    output: vec!["price_mean_5m".to_string(), "price_std_1h".to_string()],
                    data: vec![RangeData::Window(300), RangeData::Interval(10)],
                    method: RangeAlgo::Mean,
                    persist: true,
                })],
            },
        },
    };

    let features = FeatureFactory::from_config(&mock_persistence, &config.insights_service.pipeline.features).await;
    assert_eq!(features.len(), 2);

    info!("Range features created from config:");
    for feature in features {
        info!("{:?}", feature);
    }
}

#[tokio::test]
#[test_log::test]
async fn load_config_dual_range() {
    let mock_persistence: Arc<dyn arkin_core::PersistenceReader> = MockPersistence::new();
    let config = InsightsConfig {
        insights_service: InsightsServiceConfig {
            pipeline: PipelineConfig {
                version: "test_pipeline".to_string(),
                warmup_steps: 10,
                state_ttl: 3600,
                frequency_secs: 60,
                features: vec![FeatureConfig::DualRange(DualRangeConfig {
                    aggregation_type: AggregationType::Instrument,
                    filter: InstrumentFilter::default(),
                    group_by: GroupBy::default(),
                    input_1: vec!["trade_price_5m".to_string()],
                    input_2: vec!["volume_5m".to_string()],
                    output: vec!["price_volume_correlation".to_string()],
                    data: vec![RangeData::Window(300)],
                    method: DualRangeAlgo::Correlation,
                    persist: true,
                })],
            },
        },
    };

    let features = FeatureFactory::from_config(&mock_persistence, &config.insights_service.pipeline.features).await;
    assert_eq!(features.len(), 1);

    info!("DualRange features created from config:");
    for feature in features {
        info!("{:?}", feature);
    }
}

#[tokio::test]
#[test_log::test]
async fn load_config_two_value() {
    let mock_persistence: Arc<dyn arkin_core::PersistenceReader> = MockPersistence::new();
    let config = InsightsConfig {
        insights_service: InsightsServiceConfig {
            pipeline: PipelineConfig {
                version: "test_pipeline".to_string(),
                warmup_steps: 10,
                state_ttl: 3600,
                frequency_secs: 60,
                features: vec![FeatureConfig::TwoValue(TwoValueConfig {
                    aggregation_type: AggregationType::Instrument,
                    filter: InstrumentFilter::default(),
                    group_by: GroupBy::default(),
                    input_1: vec!["bid_volume".to_string()],
                    input_2: vec!["ask_volume".to_string()],
                    output: vec!["volume_imbalance".to_string()],
                    horizons: vec![RangeData::Window(60)],
                    method: TwoValueAlgo::Imbalance,
                    persist: true,
                })],
            },
        },
    };

    let features = FeatureFactory::from_config(&mock_persistence, &config.insights_service.pipeline.features).await;
    assert_eq!(features.len(), 1);

    info!("TwoValue features created from config:");
    for feature in features {
        info!("{:?}", feature);
    }
}

#[tokio::test]
#[test_log::test]
async fn load_config_with_filter_and_groupby() {
    let mock_persistence: Arc<dyn arkin_core::PersistenceReader> = MockPersistence::new();
    let config = InsightsConfig {
        insights_service: InsightsServiceConfig {
            pipeline: PipelineConfig {
                version: "test_pipeline".to_string(),
                warmup_steps: 10,
                state_ttl: 3600,
                frequency_secs: 60,
                features: vec![
                    // Example 1: Filter to specific instruments
                    FeatureConfig::Lag(LagConfig {
                        aggregation_type: AggregationType::Instrument,
                        filter: InstrumentFilter {
                            base_asset: vec!["BTC".to_string(), "ETH".to_string()],
                            instrument_type: vec![InstrumentType::Perpetual],
                            synthetic: Some(false), // Only real instruments
                            ..Default::default()
                        },
                        group_by: GroupBy::default(),
                        input: vec!["trade_price".to_string()],
                        output: vec!["log_returns".to_string()],
                        lag: vec![1],
                        method: LagAlgo::LogChange,
                        persist: true,
                    }),
                    // Example 2: Grouped aggregation
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Grouped,
                        filter: InstrumentFilter {
                            quote_asset: vec!["USDT".to_string(), "USDC".to_string()],
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: true,
                            instrument_type: true,
                            ..Default::default()
                        },
                        input: vec!["trade_notional".to_string()],
                        output: vec!["volume_grouped".to_string()],
                        data: vec![RangeData::Window(300)],
                        method: RangeAlgo::Sum,
                        persist: true,
                    }),
                    // Example 3: Single index across all
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Index,
                        filter: InstrumentFilter {
                            synthetic: Some(true), // Only synthetic instruments
                            ..Default::default()
                        },
                        group_by: GroupBy::default(),
                        input: vec!["log_returns".to_string()],
                        output: vec!["market_index".to_string()],
                        data: vec![RangeData::Window(3600)],
                        method: RangeAlgo::Mean,
                        persist: true,
                    }),
                ],
            },
        },
    };

    let features = FeatureFactory::from_config(&mock_persistence, &config.insights_service.pipeline.features).await;

    info!("Filter and GroupBy features created from config:");
    for feature in features {
        info!("{:?}", feature);
    }
}

/// Test demonstrating crypto market index construction pipeline with multi-stage aggregation
///
/// This test shows the complete flow for creating synthetic USD instruments and market indices
/// from raw trade data using a realistic multi-stage aggregation approach.
///
/// **Pipeline Flow Diagram:**
/// ```
/// Raw Trades (per event)
///   ├─ trade_price
///   ├─ trade_quantity  
///   └─ trade_notional (price * quantity * side, positive = buy, negative = sell)
///        │
///        ▼
/// ┌─────────────────────────────────────────────────────────────┐
/// │ STAGE 1: Raw Trades → 1m Bars (Window: 60s)                 │
/// │ Aggregation: per-instrument                                 │
/// │ Output:                                                     │
/// │   - notional_01m (SumAbs)                                   │
/// │   - notional_buy_01m (SumAbsPositive)                       │
/// │   - notional_sell_01m (SumAbsNegative)                      │
/// │   - price_01m (Mean)                                        │
/// │   - vwap_01m (WeightedMean: price weighted by notional)     │
/// └─────────────────────────────────────────────────────────────┘
///        │
///        ▼
/// ┌─────────────────────────────────────────────────────────────┐
/// │ STAGE 2: 1m → Multi-timeframe Bars (Intervals: 5,60,240)    │
/// │ Aggregation: per-instrument                                 │
/// │ Outputs (for each timeframe):                               │
/// │   - notional_XXm, notional_buy_XXm, notional_sell_XXm       │
/// │   - notional_imbalance_XXm (buy vs sell)                    │
/// │   - price_XXm                                               │
/// │ Timeframes: 5m, 01h, 04h, 1440m                             │
/// └─────────────────────────────────────────────────────────────┘
///        │
///        ▼
/// ┌─────────────────────────────────────────────────────────────┐
/// │ STAGE 3: Create Synthetic USD Instruments (Interval: 1)     │
/// │ Aggregation: grouped by base_asset + instrument_type        │
/// │ Filter: quote_asset IN [USDT, USDC]                         │
/// │ Output: usd_volume_05m, usd_price_05m                       │
/// │         (BTC-USD, ETH-USD, SOL-USD, etc.)                   │
/// └─────────────────────────────────────────────────────────────┘
///        │
///        ▼
/// ┌─────────────────────────────────────────────────────────────┐
/// │ STAGE 4: Calculate Returns (Lag: 5)                         │
/// │ Aggregation: grouped by base_asset                          │
/// │ Output: usd_returns_05m                                     │
/// └─────────────────────────────────────────────────────────────┘
///        │
///        ├────────────────────────────────────────────────────────┐
///        ▼                                                        ▼
/// ┌──────────────────────────────┐  ┌──────────────────────────────┐
/// │ STAGE 5: Market Indices      │  │ STAGE 6: Instrument Betas    │
/// │ Aggregation: index (single)  │  │ Aggregation: grouped         │
/// │ Filter: synthetic = true     │  │ Filter: synthetic = true     │
/// │ Output:                      │  │ Output:                      │
/// │ - market_return_equal_wt     │  │ - beta_correlation_24h       │
/// │ - market_volatility_01h      │  │ - beta_market_24h            │
/// │ - market_volume_total        │  │                              │
/// └──────────────────────────────┘  └──────────────────────────────┘
/// ```
///
/// **Key Concepts:**
/// - **Window**: Time-based aggregation (e.g., 60 seconds of raw data)
/// - **Interval**: Count-based aggregation (e.g., 5 x 1m bars = 5m)
/// - **Instrument**: One feature per instrument (default behavior)
/// - **Grouped**: One feature per group (e.g., BTC-USD, ETH-USD by base_asset)
/// - **Index**: Single feature across all filtered instruments
#[tokio::test]
#[test_log::test]
async fn crypto_market_index_construction() {
    let mock_persistence: Arc<dyn arkin_core::PersistenceReader> = MockPersistence::new();

    let config = InsightsConfig {
        insights_service: InsightsServiceConfig {
            pipeline: PipelineConfig {
                version: "test_pipeline".to_string(),
                warmup_steps: 10,
                state_ttl: 3600,
                frequency_secs: 60,
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
                            "notional_sell_01m".to_string(),
                            "notional_buy_01m".to_string(),
                            "notional_sell_01m".to_string(),
                            "notional_buy_01m".to_string(),
                            "notional_sell_01m".to_string(),
                            "notional_buy_01m".to_string(),
                            "notional_sell_01m".to_string(),
                        ],
                        output: vec![
                            "notional_buy_05m".to_string(),
                            "notional_sell_05m".to_string(),
                            "notional_buy_01h".to_string(),
                            "notional_sell_01h".to_string(),
                            "notional_buy_04h".to_string(),
                            "notional_sell_04h".to_string(),
                            "notional_buy_24h".to_string(),
                            "notional_sell_24h".to_string(),
                        ],
                        data: vec![
                            RangeData::Interval(5),
                            RangeData::Interval(5),
                            RangeData::Interval(60),
                            RangeData::Interval(60),
                            RangeData::Interval(240),
                            RangeData::Interval(240),
                            RangeData::Interval(1440),
                            RangeData::Interval(1440),
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
                    // ========================================================================
                    // STAGE 3: Create Synthetic USD Instruments (grouped by base asset)
                    // ========================================================================

                    // Aggregate USDT/USDC volume into synthetic USD volume
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Grouped,
                        filter: InstrumentFilter {
                            quote_asset: vec!["USDT".to_string(), "USDC".to_string()],
                            instrument_type: vec![InstrumentType::Perpetual],
                            synthetic: Some(false), // Only real instruments
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: true,
                            instrument_type: true,
                            ..Default::default()
                        },
                        input: vec!["notional_05m".to_string()],
                        output: vec!["usd_volume_05m".to_string()],
                        data: vec![RangeData::Interval(1)], // Already aggregated, just group
                        method: RangeAlgo::Sum,
                        persist: true,
                    }),
                    // Aggregate USDT/USDC price into synthetic USD price
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Grouped,
                        filter: InstrumentFilter {
                            quote_asset: vec!["USDT".to_string(), "USDC".to_string()],
                            instrument_type: vec![InstrumentType::Perpetual],
                            synthetic: Some(false),
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: true,
                            instrument_type: true,
                            ..Default::default()
                        },
                        input: vec!["price_05m".to_string()],
                        output: vec!["usd_price_05m".to_string()],
                        data: vec![RangeData::Interval(1)],
                        method: RangeAlgo::Mean,
                        persist: true,
                    }),
                    // ========================================================================
                    // STAGE 4: Calculate Returns (per synthetic instrument)
                    // ========================================================================
                    FeatureConfig::Lag(LagConfig {
                        aggregation_type: AggregationType::Grouped,
                        filter: InstrumentFilter {
                            synthetic: Some(true), // Only synthetic USD instruments
                            instrument_type: vec![InstrumentType::Perpetual],
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: true,
                            ..Default::default()
                        },
                        input: vec!["usd_price_05m".to_string()],
                        output: vec!["usd_returns_05m".to_string()],
                        lag: vec![5], // 5 intervals = 5 mutes
                        method: LagAlgo::LogChange,
                        persist: true,
                    }),
                    // ========================================================================
                    // STAGE 5: Build Market Indices (market-wide aggregation)
                    // ========================================================================

                    // Equal-weighted market return index
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Index,
                        filter: InstrumentFilter {
                            synthetic: Some(true),
                            instrument_type: vec![InstrumentType::Perpetual],
                            ..Default::default()
                        },
                        group_by: GroupBy::default(),
                        input: vec!["usd_returns_05m".to_string()],
                        output: vec!["market_return_equal_weighted".to_string()],
                        data: vec![RangeData::Interval(5)], // Mean over 5 x 5m bars
                        method: RangeAlgo::Mean,
                        persist: true,
                    }),
                    // Market volatility index (realized volatility over 1 hour)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Index,
                        filter: InstrumentFilter {
                            synthetic: Some(true),
                            instrument_type: vec![InstrumentType::Perpetual],
                            ..Default::default()
                        },
                        group_by: GroupBy::default(),
                        input: vec!["usd_returns_05m".to_string()],
                        output: vec!["market_volatility_01h".to_string()],
                        data: vec![RangeData::Interval(60)], // Std over 60 x 1m bars
                        method: RangeAlgo::StandardDeviation,
                        persist: true,
                    }),
                    // Market volume index (total liquidity)
                    FeatureConfig::Range(RangeConfig {
                        aggregation_type: AggregationType::Index,
                        filter: InstrumentFilter {
                            synthetic: Some(true),
                            instrument_type: vec![InstrumentType::Perpetual],
                            ..Default::default()
                        },
                        group_by: GroupBy::default(),
                        input: vec!["usd_volume_05m".to_string()],
                        output: vec!["market_volume_total".to_string()],
                        data: vec![RangeData::Interval(5)], // Sum over 5 x 5m bars
                        method: RangeAlgo::Sum,
                        persist: true,
                    }),
                    // ========================================================================
                    // STAGE 6: Calculate Betas (per synthetic instrument vs market)
                    // ========================================================================

                    // Correlation with market (24h rolling)
                    FeatureConfig::DualRange(DualRangeConfig {
                        aggregation_type: AggregationType::Grouped,
                        filter: InstrumentFilter {
                            synthetic: Some(true),
                            instrument_type: vec![InstrumentType::Perpetual],
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: true,
                            ..Default::default()
                        },
                        input_1: vec!["usd_returns_05m".to_string()],
                        input_2: vec!["market_return_equal_weighted".to_string()],
                        output: vec!["beta_correlation_24h".to_string()],
                        data: vec![RangeData::Interval(288)], // 288 x 5m bars = 24 hours
                        method: DualRangeAlgo::Correlation,
                        persist: true,
                    }),
                    // Beta vs market (proper beta calculation)
                    FeatureConfig::DualRange(DualRangeConfig {
                        aggregation_type: AggregationType::Grouped,
                        filter: InstrumentFilter {
                            synthetic: Some(true),
                            instrument_type: vec![InstrumentType::Perpetual],
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: true,
                            ..Default::default()
                        },
                        input_1: vec!["usd_returns_05m".to_string()],
                        input_2: vec!["market_return_equal_weighted".to_string()],
                        output: vec!["beta_market_24h".to_string()],
                        data: vec![RangeData::Interval(288)],
                        method: DualRangeAlgo::Beta,
                        persist: true,
                    }),
                ],
            },
        },
    };

    let features = FeatureFactory::from_config(&mock_persistence, &config.insights_service.pipeline.features).await;

    info!("Crypto Market Index features created from config:");
    info!("Total features: {}", features.len());
    for feature in &features {
        info!("{:?}", feature);
    }

    // Validate we created the expected features
    assert!(features.len() > 0, "Should create features from the crypto market index config");

    // Build the pipeline graph and validate it
    let graph = FeatureGraph::new(features);

    // Print comprehensive summary
    graph.print_summary();

    // Validate the DAG structure
    graph.validate().expect("DAG validation should pass");

    // Test dependency tracking for a specific feature
    if let Some(deps) = graph.get_dependencies("market_return_equal_weighted") {
        info!("Dependencies for 'market_return_equal_weighted': {} features", deps.len());
        for dep in &deps {
            info!("  - {:?}", dep.outputs());
        }
    }

    // Test dependency tracking for VWAP
    if let Some(deps) = graph.get_dependencies("vwap_04h") {
        info!("Dependencies for 'vwap_04h': {} features", deps.len());
        for dep in &deps {
            info!("  - {:?}", dep.outputs());
        }
    }

    // Export graph to DOT format for visualization
    let dot_output = graph.to_dot_string_simple();

    // Write DOT file
    std::fs::write("./pipeline_graph.dot", &dot_output).expect("Failed to write DOT file");
    info!("Graph exported to ./pipeline_graph.dot");

    // Export to SVG using the built-in method
    match graph.export_svg("./pipeline_graph.svg") {
        Ok(_) => {
            info!("✓ SVG generated: ./pipeline_graph.svg");
        }
        Err(e) => {
            info!("✗ Failed to generate SVG: {}", e);
            info!("  Make sure graphviz is installed: brew install graphviz");
        }
    }
    info!("");

    // Print ASCII tree view
    graph.print_tree();
}
