use arkin_core::{prelude::test_inst_binance_btc_usdt_perp, *};
use arkin_insights::prelude::InsightsState;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use std::{hint::black_box, sync::Arc, time::Duration};
use time::UtcDateTime;

fn create_test_instrument() -> Arc<Instrument> {
    test_inst_binance_btc_usdt_perp()
}

fn create_insight(
    instrument: Arc<Instrument>,
    feature_id: FeatureId,
    timestamp: UtcDateTime,
    value: f64,
) -> Arc<Insight> {
    Arc::new(
        Insight::builder()
            .event_time(timestamp)
            .instrument(instrument)
            .feature_id(feature_id)
            .value(value)
            .insight_type(InsightType::Raw)
            .build(),
    )
}

fn bench_single_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_insert");

    for batch_size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));

        let instrument = create_test_instrument();
        let feature_id = FeatureId::new("test_feature".into());

        group.bench_with_input(BenchmarkId::from_parameter(batch_size), batch_size, |b, &size| {
            b.iter_batched(
                || {
                    // Setup: create fresh state and insights for each iteration
                    let state = InsightsState::builder().build();
                    let base_timestamp = UtcDateTime::now();
                    let insights: Vec<_> = (0..size)
                        .map(|i| {
                            create_insight(
                                instrument.clone(),
                                feature_id.clone(),
                                base_timestamp + time::Duration::seconds(i as i64),
                                100.0 + i as f64,
                            )
                        })
                        .collect();
                    (state, insights)
                },
                |(state, insights)| {
                    // Benchmark: only measure the inserts
                    for insight in insights {
                        state.insert(black_box(insight));
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_batch_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_insert");

    for batch_size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));

        let instrument = create_test_instrument();
        let feature_id = FeatureId::new("test_feature".into());

        group.bench_with_input(BenchmarkId::from_parameter(batch_size), batch_size, |b, &size| {
            b.iter_batched(
                || {
                    // Setup: create fresh state and insights for each iteration
                    let state = InsightsState::builder().build();
                    let base_timestamp = UtcDateTime::now();
                    let insights: Vec<_> = (0..size)
                        .map(|i| {
                            create_insight(
                                instrument.clone(),
                                feature_id.clone(),
                                base_timestamp + time::Duration::seconds(i as i64),
                                100.0 + i as f64,
                            )
                        })
                        .collect();
                    (state, insights)
                },
                |(state, insights)| {
                    // Benchmark: only measure the batch insert
                    state.insert_batch(black_box(&insights));
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_insert_buffered(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_buffered");

    for batch_size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));

        let instrument = create_test_instrument();
        let feature_id = FeatureId::new("test_feature".into());

        group.bench_with_input(BenchmarkId::from_parameter(batch_size), batch_size, |b, &size| {
            b.to_async(tokio::runtime::Runtime::new().unwrap()).iter_batched(
                || {
                    // Setup: create fresh state and insights for each iteration
                    let state = Arc::new(InsightsState::builder().build());
                    let base_timestamp = UtcDateTime::now();
                    let insights: Vec<_> = (0..size)
                        .map(|i| {
                            create_insight(
                                instrument.clone(),
                                feature_id.clone(),
                                base_timestamp + time::Duration::seconds(i as i64),
                                100.0 + i as f64,
                            )
                        })
                        .collect();
                    let commit_time = base_timestamp + time::Duration::seconds(size as i64);
                    (state, insights, commit_time)
                },
                |(state, insights, commit_time)| async move {
                    // Benchmark: measure individual buffered inserts + commit
                    for insight in insights {
                        state.insert_buffered(black_box(insight)).await;
                    }
                    state.commit(black_box(commit_time)).await;
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_insert_batch_buffered(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_batch_buffered");

    for batch_size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));

        let instrument = create_test_instrument();
        let feature_id = FeatureId::new("test_feature".into());

        group.bench_with_input(BenchmarkId::from_parameter(batch_size), batch_size, |b, &size| {
            b.to_async(tokio::runtime::Runtime::new().unwrap()).iter_batched(
                || {
                    // Setup: create fresh state and insights for each iteration
                    let state = Arc::new(InsightsState::builder().build());
                    let base_timestamp = UtcDateTime::now();
                    let insights: Vec<_> = (0..size)
                        .map(|i| {
                            create_insight(
                                instrument.clone(),
                                feature_id.clone(),
                                base_timestamp + time::Duration::seconds(i as i64),
                                100.0 + i as f64,
                            )
                        })
                        .collect();
                    let commit_time = base_timestamp + time::Duration::seconds(size as i64);
                    (state, insights, commit_time)
                },
                |(state, insights, commit_time)| async move {
                    // Benchmark: only measure the insert + commit
                    state.insert_batch_buffered(black_box(&insights)).await;
                    state.commit(black_box(commit_time)).await;
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_point_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("point_queries");
    group.throughput(Throughput::Elements(1));

    let state = InsightsState::builder().build();
    let instrument = create_test_instrument();
    let feature_id = FeatureId::new("test_feature".into());
    let base_timestamp = UtcDateTime::now();

    // Populate with 1000 points
    for i in 0..1000 {
        let insight = create_insight(
            instrument.clone(),
            feature_id.clone(),
            base_timestamp + time::Duration::seconds(i),
            100.0 + i as f64,
        );
        state.insert(insight);
    }

    let query_time = base_timestamp + time::Duration::seconds(999);

    group.bench_function("last", |b| {
        b.iter(|| state.last(&instrument, black_box(&feature_id), black_box(query_time)));
    });

    group.bench_function("lag_10", |b| {
        b.iter(|| state.lag(&instrument, black_box(&feature_id), black_box(query_time), black_box(10)));
    });

    group.bench_function("lag_100", |b| {
        b.iter(|| state.lag(&instrument, black_box(&feature_id), black_box(query_time), black_box(100)));
    });

    group.finish();
}

fn bench_range_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_queries");

    let state = InsightsState::builder().build();
    let instrument = create_test_instrument();
    let feature_id = FeatureId::new("test_feature".into());
    let base_timestamp = UtcDateTime::now();

    // Populate with 10000 points
    for i in 0..10000 {
        let insight = create_insight(
            instrument.clone(),
            feature_id.clone(),
            base_timestamp + time::Duration::seconds(i),
            100.0 + i as f64,
        );
        state.insert(insight);
    }

    for window_size in [10, 100, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*window_size as u64));

        let start = base_timestamp;
        let window_duration = Duration::from_secs(*window_size as u64);

        group.bench_with_input(BenchmarkId::new("window", window_size), window_size, |b, _| {
            b.iter(|| {
                state.window(
                    &instrument,
                    black_box(&feature_id),
                    black_box(start),
                    black_box(window_duration),
                )
            });
        });
    }

    group.finish();
}

fn bench_last_n(c: &mut Criterion) {
    let mut group = c.benchmark_group("last_n");

    let state = InsightsState::builder().build();
    let instrument = create_test_instrument();
    let feature_id = FeatureId::new("test_feature".into());
    let base_timestamp = UtcDateTime::now();

    // Populate with 5000 points
    for i in 0..5000 {
        let insight = create_insight(
            instrument.clone(),
            feature_id.clone(),
            base_timestamp + time::Duration::seconds(i),
            100.0 + i as f64,
        );
        state.insert(insight);
    }

    let query_time = base_timestamp + time::Duration::seconds(4999);

    for n in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*n as u64));

        group.bench_with_input(BenchmarkId::from_parameter(n), n, |b, &n| {
            b.iter(|| state.last_n(&instrument, black_box(&feature_id), black_box(query_time), black_box(n)));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_insert,
    bench_batch_insert,
    bench_insert_buffered,
    bench_insert_batch_buffered,
    bench_point_queries,
    bench_range_queries,
    bench_last_n
);
criterion_main!(benches);
