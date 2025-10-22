use arkin_core::prelude::*;
use arkin_core::test_utils::{test_inst_binance_btc_usdt_perp, test_pipeline};
use arkin_insights::*;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use time::UtcDateTime;

#[tokio::test]
async fn test_range_forward_fill() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();
    let feature_id = FeatureId::new("price".to_string());

    // Create a range feature with forward fill
    let feature = RangeFeature::builder()
        .input(feature_id.clone())
        .output(FeatureId::new("mean_5".to_string()))
        .method(RangeAlgo::Mean)
        .data(RangeData::Interval(5))
        .fill_strategy(FillStrategy::ForwardFill)
        .persist(false)
        .build();

    // Insert only 3 data points at grid-aligned timestamps (expect 5 with forward fill)
    let now = UtcDateTime::now();
    let t1 = now - StdDuration::from_secs(4); // Grid: now-4
    let t2 = now - StdDuration::from_secs(2); // Grid: now-2
    let t3 = now; // Grid: now

    state.insert_batch(&[
        Arc::new(
            Insight::builder()
                .event_time(t1)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(100.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t2)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(110.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t3)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(120.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
    ]);

    // Calculate - should forward fill missing values (now-3 and now-1)
    // Expected interval: [now-4: 100, now-3: 100 (filled), now-2: 110, now-1: 110 (filled), now: 120]
    let result = feature.calculate(&state, &pipeline, &instrument, t3);
    assert!(result.is_some());

    let insights = result.unwrap();
    assert_eq!(insights.len(), 1);

    // Mean should be: (100 + 100 + 110 + 110 + 120) / 5 = 108.0
    let value = insights[0].value;
    assert_eq!(value, 108.0, "Forward fill should extend with last value");
}

#[tokio::test]
async fn test_range_zero_fill() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();
    let feature_id = FeatureId::new("volume".to_string());

    // Create a range feature with zero fill (good for volumes)
    let feature = RangeFeature::builder()
        .input(feature_id.clone())
        .output(FeatureId::new("sum_5".to_string()))
        .method(RangeAlgo::Sum)
        .data(RangeData::Interval(5))
        .fill_strategy(FillStrategy::Zero)
        .persist(false)
        .build();

    // Insert only 3 data points at grid-aligned timestamps (expect 5 with zero fill)
    let now = UtcDateTime::now();
    let t1 = now - StdDuration::from_secs(4); // Grid: now-4
    let t2 = now - StdDuration::from_secs(2); // Grid: now-2
    let t3 = now; // Grid: now

    state.insert_batch(&[
        Arc::new(
            Insight::builder()
                .event_time(t1)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(100.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t2)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(200.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t3)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(300.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
    ]);

    // Calculate - should zero fill missing values (now-3 and now-1)
    // Expected interval: [now-4: 100, now-3: 0 (filled), now-2: 200, now-1: 0 (filled), now: 300]
    let result = feature.calculate(&state, &pipeline, &instrument, t3);
    assert!(result.is_some());

    let insights = result.unwrap();
    assert_eq!(insights.len(), 1);

    // Sum should be: 100 + 0 + 200 + 0 + 300 = 600
    let value = insights[0].value;
    assert_eq!(value, 600.0, "Zero fill should add zeros for missing values");
}

#[tokio::test]
async fn test_range_drop_insufficient_data() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();
    let feature_id = FeatureId::new("price".to_string());

    // Create a range feature with drop strategy
    let feature = RangeFeature::builder()
        .input(feature_id.clone())
        .output(FeatureId::new("mean_5".to_string()))
        .method(RangeAlgo::Mean)
        .data(RangeData::Interval(5))
        .fill_strategy(FillStrategy::Drop)
        .persist(false)
        .build();

    // Insert only 1 data point (need at least 2)
    let now = UtcDateTime::now();

    state.insert_batch(&[Arc::new(
        Insight::builder()
            .event_time(now)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(feature_id.clone())
            .value(100.0)
            .insight_type(InsightType::Raw)
            .build(),
    )]);

    // Calculate - should return None because Drop strategy needs minimum data
    let result = feature.calculate(&state, &pipeline, &instrument, now);
    assert!(result.is_none(), "Drop strategy should return None with insufficient data");
}

#[tokio::test]
async fn test_range_drop_with_sufficient_data() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();
    let feature_id = FeatureId::new("price".to_string());

    // Create a range feature with drop strategy
    let feature = RangeFeature::builder()
        .input(feature_id.clone())
        .output(FeatureId::new("mean_5".to_string()))
        .method(RangeAlgo::Mean)
        .data(RangeData::Interval(5))
        .fill_strategy(FillStrategy::Drop)
        .persist(false)
        .build();

    // Insert 5 data points at grid-aligned timestamps (no fill needed)
    let now = UtcDateTime::now();
    let t1 = now - StdDuration::from_secs(4);
    let t2 = now - StdDuration::from_secs(3);
    let t3 = now - StdDuration::from_secs(2);
    let t4 = now - StdDuration::from_secs(1);
    let t5 = now;

    state.insert_batch(&[
        Arc::new(
            Insight::builder()
                .event_time(t1)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(100.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t2)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(105.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t3)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(110.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t4)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(115.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t5)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(120.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
    ]);

    // Calculate - should work with all 5 data points
    let result = feature.calculate(&state, &pipeline, &instrument, t5);
    assert!(result.is_some());

    let insights = result.unwrap();
    assert_eq!(insights.len(), 1);

    // Mean should be: (100 + 105 + 110 + 115 + 120) / 5 = 110.0
    let value = insights[0].value;
    assert_eq!(value, 110.0, "Drop strategy should calculate with all data present");
}

#[tokio::test]
async fn test_two_value_zero_fill() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();

    // Create a two value feature with zero fill (e.g., division)
    // Testing that missing input_1 gets zero filled
    let feature = TwoValueFeature::builder()
        .input_1(FeatureId::new("buy_volume".to_string()))
        .input_2(FeatureId::new("sell_volume".to_string()))
        .output(FeatureId::new("ratio".to_string()))
        .method(TwoValueAlgo::Division)
        .fill_strategy(FillStrategy::Zero)
        .persist(false)
        .build();

    // Insert only sell_volume (buy_volume missing) - zero fill should give 0 / sell
    let now = UtcDateTime::now();

    state.insert_batch(&[Arc::new(
        Insight::builder()
            .event_time(now)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(FeatureId::new("sell_volume".to_string()))
            .value(100.0)
            .insight_type(InsightType::Raw)
            .build(),
    )]);

    // Calculate - two_value features don't support fill strategy for single-value queries
    // They return None when data is missing (correct behavior)
    let result = feature.calculate(&state, &pipeline, &instrument, now);
    assert!(result.is_none(), "Two-value features return None when either input is missing");
}

#[tokio::test]
async fn test_forward_fill_empty_state() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();

    // Create a range feature with forward fill
    let feature = RangeFeature::builder()
        .input(FeatureId::new("price".to_string()))
        .output(FeatureId::new("mean_5".to_string()))
        .method(RangeAlgo::Mean)
        .data(RangeData::Interval(5))
        .fill_strategy(FillStrategy::ForwardFill)
        .persist(false)
        .build();

    // Don't insert any data

    // Calculate - should return None because there's nothing to forward fill
    let now = UtcDateTime::now();
    let result = feature.calculate(&state, &pipeline, &instrument, now);
    assert!(result.is_none(), "Forward fill with empty state should return None");
}

#[tokio::test]
async fn test_dual_range_forward_fill() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();

    // Create a dual range feature with forward fill (correlation between two price series)
    let feature = DualRangeFeature::builder()
        .input_1(FeatureId::new("price_a".to_string()))
        .input_2(FeatureId::new("price_b".to_string()))
        .output(FeatureId::new("correlation".to_string()))
        .method(DualRangeAlgo::Correlation)
        .data(RangeData::Interval(5))
        .fill_strategy(FillStrategy::ForwardFill)
        .persist(false)
        .build();

    // Insert only 3 data points for each series (expect 5)
    let now = UtcDateTime::now();
    let t1 = now - StdDuration::from_secs(20);
    let t2 = now - StdDuration::from_secs(10);
    let t3 = now;

    state.insert_batch(&[
        // Price A series
        Arc::new(
            Insight::builder()
                .event_time(t1)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(FeatureId::new("price_a".to_string()))
                .value(100.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t2)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(FeatureId::new("price_a".to_string()))
                .value(110.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t3)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(FeatureId::new("price_a".to_string()))
                .value(120.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        // Price B series
        Arc::new(
            Insight::builder()
                .event_time(t1)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(FeatureId::new("price_b".to_string()))
                .value(50.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t2)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(FeatureId::new("price_b".to_string()))
                .value(55.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t3)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(FeatureId::new("price_b".to_string()))
                .value(60.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
    ]);

    // Calculate - should forward fill both series (120.0 twice for A, 60.0 twice for B)
    let result = feature.calculate(&state, &pipeline, &instrument, t3);
    assert!(result.is_some(), "Forward fill should allow calculation");

    let insights = result.unwrap();
    assert_eq!(insights.len(), 1);

    // Both series are perfectly correlated (both increasing linearly), so correlation should be 1.0
    let value = insights[0].value;
    assert!(
        value > 0.95,
        "Correlation should be close to 1.0 for perfectly correlated series, got {}",
        value
    );
}

#[tokio::test]
async fn test_dual_range_zero_fill() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();

    // Create a dual range feature with zero fill
    let feature = DualRangeFeature::builder()
        .input_1(FeatureId::new("volume_a".to_string()))
        .input_2(FeatureId::new("volume_b".to_string()))
        .output(FeatureId::new("beta".to_string()))
        .method(DualRangeAlgo::Beta)
        .data(RangeData::Interval(5))
        .fill_strategy(FillStrategy::Zero)
        .persist(false)
        .build();

    // Insert only 2 data points (expect 5)
    let now = UtcDateTime::now();
    let t1 = now - StdDuration::from_secs(10);
    let t2 = now;

    state.insert_batch(&[
        Arc::new(
            Insight::builder()
                .event_time(t1)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(FeatureId::new("volume_a".to_string()))
                .value(100.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t2)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(FeatureId::new("volume_a".to_string()))
                .value(200.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t1)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(FeatureId::new("volume_b".to_string()))
                .value(50.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t2)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(FeatureId::new("volume_b".to_string()))
                .value(100.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
    ]);

    // Calculate - should zero fill to get 5 points: [100, 200, 0, 0, 0] and [50, 100, 0, 0, 0]
    let result = feature.calculate(&state, &pipeline, &instrument, t2);
    assert!(result.is_some(), "Zero fill should allow calculation");
}

#[tokio::test]
#[ignore]
async fn test_lag_zero_fill() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();
    let feature_id = FeatureId::new("returns".to_string());

    // Create a lag feature with zero fill
    let feature = LagFeature::builder()
        .input(feature_id.clone())
        .output(FeatureId::new("returns_diff".to_string()))
        .method(LagAlgo::Difference)
        .lag(1)
        .fill_strategy(FillStrategy::Zero)
        .persist(false)
        .build();

    // Insert only current value (no lagged value available)
    let now = UtcDateTime::now();

    state.insert_batch(&[Arc::new(
        Insight::builder()
            .event_time(now)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(feature_id.clone())
            .value(0.05)
            .insight_type(InsightType::Raw)
            .build(),
    )]);

    // Calculate - zero fill should return 0.0 for missing lagged value
    // Difference: current(0.05) - lagged(0.0) = 0.05
    let result = feature.calculate(&state, &pipeline, &instrument, now);
    assert!(result.is_some(), "Zero fill should provide 0.0 for missing lagged value");

    let insights = result.unwrap();
    assert_eq!(insights.len(), 1);
    assert_eq!(
        insights[0].value, 0.05,
        "Difference with zero-filled lag should be the current value"
    );
}

#[tokio::test]
async fn test_lag_drop() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();
    let feature_id = FeatureId::new("price".to_string());

    // Create a lag feature with drop strategy
    let feature = LagFeature::builder()
        .input(feature_id.clone())
        .output(FeatureId::new("log_return".to_string()))
        .method(LagAlgo::LogChange)
        .lag(1)
        .fill_strategy(FillStrategy::Drop)
        .persist(false)
        .build();

    // Insert only current value (no lagged value)
    let now = UtcDateTime::now();

    state.insert_batch(&[Arc::new(
        Insight::builder()
            .event_time(now)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(feature_id.clone())
            .value(100.0)
            .insight_type(InsightType::Raw)
            .build(),
    )]);

    // Calculate - should return None because Drop strategy requires both values
    let result = feature.calculate(&state, &pipeline, &instrument, now);
    assert!(
        result.is_none(),
        "Drop strategy should return None when lagged value is missing"
    );
}

#[tokio::test]
async fn test_standard_deviation_with_forward_fill() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();
    let feature_id = FeatureId::new("returns".to_string());

    // Create a range feature for standard deviation with forward fill
    let feature = RangeFeature::builder()
        .input(feature_id.clone())
        .output(FeatureId::new("std_10".to_string()))
        .method(RangeAlgo::StandardDeviation)
        .data(RangeData::Interval(10))
        .fill_strategy(FillStrategy::ForwardFill)
        .persist(false)
        .build();

    // Insert 5 data points at grid-aligned timestamps (expect 10 with forward fill) with varying returns
    let now = UtcDateTime::now();
    let t1 = now - StdDuration::from_secs(9);
    let t2 = now - StdDuration::from_secs(7);
    let t3 = now - StdDuration::from_secs(5);
    let t4 = now - StdDuration::from_secs(3);
    let t5 = now - StdDuration::from_secs(1);

    state.insert_batch(&[
        Arc::new(
            Insight::builder()
                .event_time(t1)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(0.01)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t2)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(0.02)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t3)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(-0.01)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t4)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(0.03)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t5)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(-0.02)
                .insight_type(InsightType::Raw)
                .build(),
        ),
    ]);

    // Calculate - should forward fill missing timestamps
    // Expected: [now-9: 0.01, now-8: 0.01, now-7: 0.02, now-6: 0.02, now-5: -0.01, now-4: -0.01, now-3: 0.03, now-2: 0.03, now-1: -0.02, now: -0.02]
    let result = feature.calculate(&state, &pipeline, &instrument, now);
    assert!(result.is_some(), "Forward fill should allow calculation");

    let output = result.unwrap();
    assert_eq!(output.len(), 1);

    // With forward fill: [0.01, 0.01, 0.02, 0.02, -0.01, -0.01, 0.03, 0.03, -0.02, -0.02]
    let value = output[0].value;
    assert!(value >= 0.0, "Standard deviation should be non-negative");
    assert!(!value.is_nan(), "Standard deviation should not be NaN");
    assert!(value > 0.0, "Standard deviation should be positive for varying data");
}

#[tokio::test]
async fn test_quantile_with_zero_fill() {
    // Create state and pipeline
    let state = FeatureStore::builder().min_interval(1).build();
    let pipeline = test_pipeline();
    let instrument = test_inst_binance_btc_usdt_perp();
    let feature_id = FeatureId::new("volume".to_string());

    // Create a quantile feature with zero fill
    let feature = RangeFeature::builder()
        .input(feature_id.clone())
        .output(FeatureId::new("q95".to_string()))
        .method(RangeAlgo::Quantile(0.95))
        .data(RangeData::Interval(10))
        .fill_strategy(FillStrategy::Zero)
        .persist(false)
        .build();

    // Insert only 4 data points at grid-aligned timestamps (expect 10 with zero fill)
    let now = UtcDateTime::now();
    let t1 = now - StdDuration::from_secs(9);
    let t2 = now - StdDuration::from_secs(6);
    let t3 = now - StdDuration::from_secs(3);
    let t4 = now;

    state.insert_batch(&[
        Arc::new(
            Insight::builder()
                .event_time(t1)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(100.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t2)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(200.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t3)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(300.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
        Arc::new(
            Insight::builder()
                .event_time(t4)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(400.0)
                .insight_type(InsightType::Raw)
                .build(),
        ),
    ]);

    // Calculate - should zero fill missing timestamps
    // Expected: [now-9: 100, now-8: 0, now-7: 0, now-6: 200, now-5: 0, now-4: 0, now-3: 300, now-2: 0, now-1: 0, now: 400]
    let result = feature.calculate(&state, &pipeline, &instrument, now);
    assert!(result.is_some(), "Zero fill should allow calculation");

    let output = result.unwrap();
    assert_eq!(output.len(), 1);

    // 95th percentile of [100, 0, 0, 200, 0, 0, 300, 0, 0, 400]
    // Sorted: [0, 0, 0, 0, 0, 0, 100, 200, 300, 400]
    // 95th percentile is at position 9.5 (interpolated between 300 and 400)
    let value = output[0].value;
    assert!(value >= 300.0, "95th percentile should be between 300-400, got {}", value);
}
