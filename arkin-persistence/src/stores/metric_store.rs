use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use async_stream::stream;
use futures::{channel, stream, Stream, StreamExt};
use time::UtcDateTime;
use tracing::{error, info};

use arkin_core::prelude::*;

use crate::repos::ch::metric_repo;
use crate::{context::PersistenceContext, repos::ch::insight_repo};

pub async fn create_table(ctx: &PersistenceContext) -> Result<(), PersistenceError> {
    metric_repo::create_table(ctx).await
}

pub async fn insert_metric(ctx: &PersistenceContext, metric: Arc<Metric>) -> Result<(), PersistenceError> {
    metric_repo::insert(ctx, metric.into()).await?;
    Ok(())
}

pub async fn batch_insert_metric(ctx: &PersistenceContext, metrics: &[Arc<Metric>]) -> Result<(), PersistenceError> {
    let metrics = metrics.iter().map(|i| i.clone().into()).collect::<Vec<_>>();
    metric_repo::insert_batch(ctx, &metrics).await
}

pub async fn stream_range_buffered(
    ctx: &PersistenceContext,
    instruments: &[Arc<Instrument>],
    metric_type: MetricType,
    start: UtcDateTime,
    end: UtcDateTime,
    buffer_size: usize,
    frequency: Frequency,
) -> Result<Box<dyn Stream<Item = Event> + Send + Unpin>, PersistenceError> {
    // Split the range into chunks (hourly/daily)
    let time_chunks = datetime_chunks(start, end, frequency)
        .map_err(|e| PersistenceError::Other(format!("Failed to chunk datetime range {} - {}: {}", start, end, e)))?;
    let instrument_ids = Arc::new(instruments.iter().map(|i| i.id).collect::<Vec<_>>());

    // Build local instrument lookup for fast mapping
    let local_instrument_lookup =
        Arc::new(instruments.iter().map(|i| (i.id, Arc::clone(i))).collect::<HashMap<_, _>>());

    let ctx_clone = ctx.clone();

    // Create a stream that processes chunks concurrently with buffering
    let stream = stream::iter(time_chunks)
        .map(move |(start_batch, end_batch)| {
            let ctx_clone = ctx_clone.clone();
            let instrument_ids = instrument_ids.clone();
            let local_instrument_lookup = local_instrument_lookup.clone();

            async move {
                info!(target: "persistence", "Streaming metrics for batch: {} - {}", start_batch, end_batch);

                // Get the cursor for this time chunk
                let mut cursor =
                    match metric_repo::stream_range(&ctx_clone, &instrument_ids, metric_type, start_batch, end_batch).await {
                        Ok(c) => c,
                        Err(e) => {
                            error!(target: "persistence", "Failed to create cursor for batch {} - {}: {}", start_batch, end_batch, e);
                            return stream::iter(vec![]).boxed();
                        }
                    };

                // Stream rows from this cursor, parsing each row immediately
                let row_stream = stream! {
                    loop {
                        match cursor.next().await {
                            Ok(Some(dto)) => {
                                // Parse immediately as we receive each row
                                if let Some(instrument) = local_instrument_lookup.get(&dto.instrument_id) {
                                    let metric = Metric::builder()
                                        .event_time(dto.event_time.to_utc())
                                        .instrument(instrument.clone())
                                        .metric_type(MetricType::from_str(&dto.metric_type).unwrap())
                                        .value(dto.value)
                                        .build();

                                    yield Event::MetricUpdate(Arc::new(metric));
                                } else {
                                    error!(target: "persistence", "Instrument {} not found in lookup", dto.instrument_id);
                                }
                            }
                            Ok(None) => {
                                info!(target: "persistence", "Finished streaming batch: {} - {}", start_batch, end_batch);
                                break;
                            }
                            Err(e) => {
                                error!(target: "persistence", "Error streaming row: {}", e);
                                break;
                            }
                        }
                    }
                };

                row_stream.boxed()
            }
        })
        // Buffer N chunk cursors concurrently
        .buffered(buffer_size)
        // Flatten all the row streams into a single stream
        .flatten();

    Ok(Box::new(Box::pin(stream)))
}

// pub async fn stream_range_buffered(
//     ctx: &PersistenceContext,
//     instruments: &[Arc<Instrument>],
//     metric_type: MetricType,
//     start: UtcDateTime,
//     end: UtcDateTime,
//     buffer_size: usize,
//     frequency: Frequency,
// ) -> Box<dyn Stream<Item = Event> + Send + Unpin> {
//     // Split the range into daily chunks
//     let time_chunks = datetime_chunks(start, end, frequency).unwrap();
//     let instrument_ids = Arc::new(instruments.iter().map(|i| i.id).collect::<Vec<_>>());
//     let local_instrument_lookup =
//         Arc::new(instruments.iter().map(|i| (i.id, Arc::clone(i))).collect::<HashMap<_, _>>());

//     // Create a stream of futures for each daily chunk
//     let ctx_clone = ctx.clone();
//     let fetch_stream = stream::iter(time_chunks).map(move |(start_batch, end_batch)| {
//         let ctx_clone = ctx_clone.clone();
//         let instrument_ids = instrument_ids.clone();
//         let local_instrument_lookup = local_instrument_lookup.clone();

//         async move {
//             info!("Fetching metrics for batch: {} - {}", start_batch, end_batch);

//             // Fetch with retries
//             let res = retry(
//                 || metric_repo::fetch_batch(&ctx_clone, &instrument_ids, metric_type, start_batch, end_batch),
//                 5, // Max retries
//             )
//             .await;

//             let batch = res.expect("Failed to fetch batch, abort mission");
//             let mut metrics = Vec::with_capacity(batch.len());
//             for dto in batch {
//                 let instrument = local_instrument_lookup.get(&dto.instrument_id).cloned().unwrap();
//                 let metric = Metric::builder()
//                     .event_time(dto.event_time.to_utc())
//                     .instrument(instrument)
//                     .metric_type(MetricType::from_str(&dto.metric_type).unwrap())
//                     .value(dto.value)
//                     .build();
//                 metrics.push(Event::MetricUpdate(Arc::new(metric)));
//             }
//             metrics
//         }
//     });
//     Box::new(fetch_stream.buffered(buffer_size).flat_map(|x| stream::iter(x)))
// }
