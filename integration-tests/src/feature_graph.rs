use arkin_core::prelude::*;
use arkin_insights::prelude::*;

pub fn build_pipeline_v2() -> InsightsConfig {
    InsightsConfig {
        insights_service: InsightsServiceConfig {
            pipeline: PipelineConfig {
                version: "v2.0.0".to_string(),
                name: "test_pipeline".to_string(),
                description: "test_pipeline".to_string(),
                reference_currency: "USD".to_string(),
                warmup_steps: 180,
                state_ttl: 86400,
                min_interval: 60,
                parallel: true,
                global_instrument_selector: InstrumentSelector {
                    base_asset: vec!["BTC".to_string(), "ETH".to_string()],
                    quote_asset: vec!["USDT".to_string(), "USDC".to_string()],
                    venue: vec![VenueName::Binance],
                    instrument_type: vec![InstrumentType::Perpetual, InstrumentType::Spot],
                    ..Default::default()
                },
                features: vec![
                    // =======================================================================
                    // STAGE 1: Raw Trades → 1m Aggregates (per synthetic instrument)
                    // =======================================================================
                    FeatureConfig::Range(RangeConfig {
                        instrument_selector: InstrumentSelector {
                            synthetic: Some(false),
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: false,
                            quote_asset: vec!["USDT".to_string(), "USDC".to_string()],
                            instrument_type: true, // Group by type
                            venue: false,          // Global synthetics
                        },
                        input: vec![
                            "trade_price".to_string(),
                            "trade_price".to_string(),
                            "trade_price".to_string(),
                            "trade_notional".to_string(),
                            "trade_notional".to_string(),
                            "trade_notional".to_string(),
                            "trade_notional".to_string(),
                        ],
                        output: vec![
                            "high_01m".to_string(),
                            "low_01m".to_string(),
                            "close_01m".to_string(),
                            "total_notional_01m".to_string(),
                            "buy_notional_01m".to_string(),
                            "sell_notional_01m".to_string(),
                            "diff_notional_01m".to_string(),
                        ],
                        data: vec![
                            RangeData::Window(60),
                            RangeData::Window(60),
                            RangeData::Window(60),
                            RangeData::Window(60),
                            RangeData::Window(60),
                            RangeData::Window(60),
                            RangeData::Window(60),
                        ],
                        method: vec![
                            RangeAlgo::Max,
                            RangeAlgo::Min,
                            RangeAlgo::Last,
                            RangeAlgo::AbsSum,
                            RangeAlgo::AbsSumPositive,
                            RangeAlgo::AbsSumNegative,
                            RangeAlgo::Sum,
                        ],
                        fill_strategy: FillStrategy::ForwardFill, // Volume, zero if no data
                    }),
                    // 1m vwap (volume-weighted average price over 60 second window)
                    FeatureConfig::DualRange(DualRangeConfig {
                        instrument_selector: InstrumentSelector {
                            synthetic: Some(false),
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: false,
                            quote_asset: vec!["USDT".to_string(), "USDC".to_string()],
                            instrument_type: true, // Group by type
                            venue: false,          // Global synthetics
                        },
                        input_1: vec!["trade_price".to_string()],
                        input_2: vec!["trade_quantity".to_string()], // Use notional
                        output: vec!["vwap_01m".to_string()],
                        data: vec![RangeData::Window(60)],
                        method: vec![DualRangeAlgo::WeightedMean],
                        fill_strategy: FillStrategy::ForwardFill, // VWAP, zero if no data
                    }),
                    // =======================================================================
                    // STAGE 2: 1m → Multi-timeframe Aggregates (per synthetic instrument)
                    // =======================================================================
                    FeatureConfig::Range(RangeConfig {
                        instrument_selector: InstrumentSelector {
                            synthetic: Some(true),
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: false,
                            instrument_type: true, // Group by type
                            venue: false,          // Global synthetics
                            ..Default::default()
                        },
                        input: vec![
                            "total_notional_01m".to_string(),
                            "buy_notional_01m".to_string(),
                            "sell_notional_01m".to_string(),
                            "diff_notional_01m".to_string(),
                            "total_notional_01m".to_string(),
                            "buy_notional_01m".to_string(),
                            "sell_notional_01m".to_string(),
                            "diff_notional_01m".to_string(),
                            "total_notional_01m".to_string(),
                            "buy_notional_01m".to_string(),
                            "sell_notional_01m".to_string(),
                            "diff_notional_01m".to_string(),
                        ],
                        output: vec![
                            "total_notional_05m".to_string(),
                            "buy_notional_05m".to_string(),
                            "sell_notional_05m".to_string(),
                            "diff_notional_05m".to_string(),
                            "total_notional_15m".to_string(),
                            "buy_notional_15m".to_string(),
                            "sell_notional_15m".to_string(),
                            "diff_notional_15m".to_string(),
                            "total_notional_60m".to_string(),
                            "buy_notional_60m".to_string(),
                            "sell_notional_60m".to_string(),
                            "diff_notional_60m".to_string(),
                        ],
                        data: vec![
                            RangeData::Interval(5),
                            RangeData::Interval(5),
                            RangeData::Interval(5),
                            RangeData::Interval(5),
                            RangeData::Interval(15),
                            RangeData::Interval(15),
                            RangeData::Interval(15),
                            RangeData::Interval(15),
                            RangeData::Interval(60),
                            RangeData::Interval(60),
                            RangeData::Interval(60),
                            RangeData::Interval(60),
                        ],
                        method: vec![
                            RangeAlgo::Sum,
                            RangeAlgo::Sum,
                            RangeAlgo::Sum,
                            RangeAlgo::Sum,
                            RangeAlgo::Sum,
                            RangeAlgo::Sum,
                            RangeAlgo::Sum,
                            RangeAlgo::Sum,
                            RangeAlgo::Sum,
                            RangeAlgo::Sum,
                            RangeAlgo::Sum,
                            RangeAlgo::Sum,
                        ],
                        fill_strategy: FillStrategy::ForwardFill,
                    }),
                    FeatureConfig::TwoValue(TwoValueConfig {
                        instrument_selector_1: InstrumentSelector {
                            synthetic: Some(true),
                            instrument_type: vec![InstrumentType::Spot],
                            venue: vec![VenueName::Index],
                            ..Default::default()
                        },
                        instrument_selector_2: InstrumentSelector {
                            synthetic: Some(true),
                            instrument_type: vec![InstrumentType::Perpetual],
                            venue: vec![VenueName::Index],
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: false,
                            instrument_type: false, // Group by type
                            venue: false,           // Global synthetics
                            ..Default::default()
                        },
                        input_1: vec![
                            "total_notional_05m".to_string(),
                            "total_notional_15m".to_string(),
                            "total_notional_60m".to_string(),
                        ],
                        input_2: vec![
                            "total_notional_05m".to_string(),
                            "total_notional_15m".to_string(),
                            "total_notional_60m".to_string(),
                        ],
                        output: vec![
                            "notional_imbalance_05m".to_string(),
                            "notional_imbalance_15m".to_string(),
                            "notional_imbalance_60m".to_string(),
                        ],
                        method: vec![TwoValueAlgo::Imbalance, TwoValueAlgo::Imbalance, TwoValueAlgo::Imbalance],
                        fill_strategy: FillStrategy::ForwardFill,
                    }),
                    FeatureConfig::Lag(LagConfig {
                        instrument_selector: InstrumentSelector {
                            synthetic: Some(true),
                            venue: vec![VenueName::Index],
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: false,
                            instrument_type: false, // Group by type
                            venue: false,           // Global synthetics
                            ..Default::default()
                        },
                        input: vec![
                            "notional_imbalance_05m".to_string(),
                            "notional_imbalance_15m".to_string(),
                            "notional_imbalance_60m".to_string(),
                        ],
                        output: vec![
                            "notional_imbalance_05m_pct_change".to_string(),
                            "notional_imbalance_15m_pct_change".to_string(),
                            "notional_imbalance_60m_pct_change".to_string(),
                        ],
                        lag: vec![1, 1, 1],
                        method: vec![LagAlgo::PercentChange, LagAlgo::PercentChange, LagAlgo::PercentChange],
                        fill_strategy: FillStrategy::ForwardFill,
                    }),
                    FeatureConfig::Lag(LagConfig {
                        instrument_selector: InstrumentSelector {
                            synthetic: Some(true),
                            venue: vec![VenueName::Index],
                            ..Default::default()
                        },
                        group_by: GroupBy {
                            base_asset: false,
                            instrument_type: false, // Group by type
                            venue: false,           // Global synthetics
                            ..Default::default()
                        },
                        input: vec![
                            "notional_imbalance_05m_pct_change".to_string(),
                            "notional_imbalance_15m_pct_change".to_string(),
                            "notional_imbalance_60m_pct_change".to_string(),
                        ],
                        output: vec![
                            "notional_imbalance_05m_acceleration".to_string(),
                            "notional_imbalance_15m_acceleration".to_string(),
                            "notional_imbalance_60m_acceleration".to_string(),
                        ],
                        lag: vec![1, 1, 1],
                        method: vec![LagAlgo::PercentChange, LagAlgo::PercentChange, LagAlgo::PercentChange],
                        fill_strategy: FillStrategy::ForwardFill,
                    }),
                ],
            },
        },
    }
}
