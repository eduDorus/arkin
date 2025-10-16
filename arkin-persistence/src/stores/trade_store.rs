use std::{collections::HashMap, sync::Arc};

use async_stream::stream;
use async_stream::try_stream;
use futures::FutureExt;
use futures::{stream, Stream, StreamExt};
use time::UtcDateTime;
use tracing::error;
use tracing::info;

use arkin_core::prelude::*;

use crate::{context::PersistenceContext, repos::ch::trade_repo, stores::instrument_store};

pub async fn create_table(ctx: &PersistenceContext) -> Result<(), PersistenceError> {
    trade_repo::create_table(ctx).await
}

pub async fn insert(ctx: &PersistenceContext, trade: Arc<AggTrade>) -> Result<(), PersistenceError> {
    trade_repo::insert(ctx, trade.into()).await
}

pub async fn insert_vec(ctx: &PersistenceContext, trades: &[Arc<AggTrade>]) -> Result<(), PersistenceError> {
    let ticks = trades.into_iter().cloned().map(|t| t.into()).collect::<Vec<_>>();
    trade_repo::insert_batch(ctx, &ticks).await
}

pub async fn read_range(
    ctx: &PersistenceContext,
    instruments: &[Arc<Instrument>],
    from: UtcDateTime,
    to: UtcDateTime,
) -> Result<Vec<Arc<AggTrade>>, PersistenceError> {
    let ids = instruments.iter().map(|i| i.id).collect::<Vec<_>>();
    let dto = trade_repo::read_range(ctx, &ids, from, to).await?;

    let mut trades = Vec::with_capacity(dto.len());
    for trade in &dto {
        let instrument = instrument_store::read_by_id(ctx, &trade.instrument_id).await?;
        let trade = AggTrade::builder()
            .event_time(trade.event_time.to_utc())
            .instrument(instrument)
            .trade_id(trade.trade_id as u64)
            .side(trade.side.into())
            .price(trade.price)
            .quantity(trade.quantity)
            .build();
        trades.push(Arc::new(trade));
    }
    Ok(trades)
}

pub async fn stream_range(
    ctx: &PersistenceContext,
    instruments: &[Arc<Instrument>],
    from: UtcDateTime,
    to: UtcDateTime,
) -> Result<impl Stream<Item = Result<Arc<AggTrade>, PersistenceError>> + 'static, PersistenceError> {
    // We do not `async` here, because returning `impl Stream` + `'a` from an `async fn`
    // is not yet stable. Instead, we return a non-async function that constructs the stream.

    // Collect the IDs.
    let ids = instruments.iter().map(|i| i.id).collect::<Vec<_>>();
    let mut cursor = trade_repo::stream_range(ctx, &ids, from, to).await?;

    // Build a stream that yields trades.
    let ctx_clone = ctx.clone();
    let stream = stream! {
        loop {
            match cursor.next().await {
                Ok(Some(row)) => {
                    // For each row, do your transformations.
                    match instrument_store::read_by_id(&ctx_clone, &row.instrument_id).await {
                        Ok(instrument) => {
                            let trade = AggTrade::builder()
                                .event_time(row.event_time.to_utc())
                                .instrument(instrument)
                                .trade_id(row.trade_id as u64)
                                .side(row.side.into())
                                .price(row.price)
                                .quantity(row.quantity)
                                .build();

                            // Yield the constructed trade to the stream.
                            yield Ok(Arc::new(trade));
                        }
                        Err(e) => yield Err(e),
                    }
                }
                Ok(None) => break,
                Err(e) => yield Err(PersistenceError::ClickhouseError(e)),
            }
        }
    };
    Ok(stream)
}

pub async fn stream_range_buffered(
    ctx: &PersistenceContext,
    instruments: &[Arc<Instrument>],
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
                info!(target: "persistence", "Streaming trades for batch: {} - {}", start_batch, end_batch);

                // Get the cursor for this time chunk
                let mut cursor =
                    match trade_repo::stream_range(&ctx_clone, &instrument_ids, start_batch, end_batch).await {
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
                                    let trade = AggTrade::builder()
                                        .event_time(dto.event_time.to_utc())
                                        .instrument(Arc::clone(instrument))
                                        .trade_id(dto.trade_id as u64)
                                        .side(dto.side.into())
                                        .price(dto.price)
                                        .quantity(dto.quantity)
                                        .build();

                                    yield Event::AggTradeUpdate(Arc::new(trade));
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
