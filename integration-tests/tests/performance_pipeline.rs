use std::{time::Duration, vec};

use anyhow::Result;
use futures::StreamExt;
use integration_tests::feature_graph::build_pipeline_v2;
use rust_decimal::prelude::*;
use time::macros::utc_datetime;
use tracing::info;

use arkin_core::prelude::*;
use arkin_insights::prelude::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 16)]
#[test_log::test]
async fn test_feature_pipeline_v2() -> Result<()> {
    let start = utc_datetime!(2025-01-01 00:00:00);
    let end = utc_datetime!(2025-01-01 01:00:00); // Extended to 3 hours to see calculations after warmup

    let persistence = integration_tests::init_test_persistence().await;
    persistence.refresh().await?;

    // Build pipeline
    let config = build_pipeline_v2();
    let pipeline = FeaturePipeline::new(&persistence, &config.insights_service.pipeline).await;
    pipeline
        .graph()
        .export_svg("./pipeline_v2.svg")
        .expect("Failed to export pipeline graph");

    // Print the features in the pipeline
    pipeline.graph().print_summary();
    // pipeline.graph().print_tree();

    // Get synthetic instruments from the pipeline
    let instruments = pipeline.real_instruments();
    let syn_instruments = pipeline.synthetic_instruments();

    info!("Found {} real instruments from pipeline", instruments.len());
    for inst in &instruments {
        info!(" - {}", inst);
    }
    info!("Found {} synthetic instruments from pipeline", syn_instruments.len());
    for inst in &syn_instruments {
        info!(" - {}", inst);
    }

    // let stream = persistence
    //     .agg_trade_stream_range_buffered(instruments.as_slice(), start, end, 3, Frequency::Daily)
    //     .await?;

    // tokio::pin!(stream);

    // let interval = Duration::from_secs(60);
    // let mut next_insights_tick = start + interval;

    // // Pre-fetch feature IDs
    // let trade_price_feature = persistence.get_feature_id("trade_price").await;
    // let trade_quantity_feature = persistence.get_feature_id("trade_quantity").await;
    // let trade_notional_feature = persistence.get_feature_id("trade_notional").await;

    // let mut total_trades: u64 = 0;
    // let mut total_calculated_insights: u64 = 0;

    // let mut total_batch_insert_duration = std::time::Duration::from_secs(0);
    // let mut total_compute_duration = std::time::Duration::from_secs(0);

    // let total_time_start = std::time::Instant::now();
    // while let Some(event) = stream.next().await {
    //     let trade = match event {
    //         Event::AggTradeUpdate(t) => t,
    //         _ => continue,
    //     };

    //     total_trades += 1;

    //     // Insert trade data
    //     let insights = vec![
    //         Insight::builder()
    //             .event_time(trade.event_time)
    //             .instrument(trade.instrument.clone())
    //             .feature_id(trade_price_feature.clone())
    //             .value(trade.price.to_f64().unwrap_or(f64::NAN))
    //             .insight_type(InsightType::Raw)
    //             .build()
    //             .into(),
    //         Insight::builder()
    //             .event_time(trade.event_time)
    //             .instrument(trade.instrument.clone())
    //             .feature_id(trade_quantity_feature.clone())
    //             .value(trade.quantity.to_f64().unwrap_or(f64::NAN) * f64::from(trade.side))
    //             .insight_type(InsightType::Raw)
    //             .build()
    //             .into(),
    //         Insight::builder()
    //             .event_time(trade.event_time)
    //             .instrument(trade.instrument.clone())
    //             .feature_id(trade_notional_feature.clone())
    //             .value((trade.price * trade.quantity).to_f64().unwrap_or(f64::NAN) * f64::from(trade.side))
    //             .insight_type(InsightType::Raw)
    //             .build()
    //             .into(),
    //     ];
    //     let batch_insert_start = std::time::Instant::now();
    //     pipeline.insert_batch(insights);
    //     total_batch_insert_duration += batch_insert_start.elapsed();

    //     // Check if we should calculate
    //     if trade.event_time > next_insights_tick {
    //         // Calculate insights
    //         let commit_start = std::time::Instant::now();
    //         let calculated_insights = pipeline.calculate(next_insights_tick).await;
    //         total_compute_duration += commit_start.elapsed();
    //         total_calculated_insights += calculated_insights.len() as u64;

    //         info!(
    //             "Processing tick at {} - total trades so far: {} batch insert duration: {:?} total commit duration: {:?}",
    //             next_insights_tick, total_trades, total_batch_insert_duration, total_compute_duration
    //         );

    //         next_insights_tick += interval;
    //     }
    // }

    // info!(
    //     "✅ Pipeline execution complete - processed {} trades ({:?} trades/s) and calculated {} insights ({:?} insights/s) total",
    //     total_trades, total_trades as f64 / total_time_start.elapsed().as_secs_f64(), total_calculated_insights, total_calculated_insights as f64 / total_time_start.elapsed().as_secs_f64()
    // );

    Ok(())
}
