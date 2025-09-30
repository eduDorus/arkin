use std::{collections::HashMap, sync::Arc};

use async_stream::stream;
use async_stream::try_stream;
use futures::{stream, Stream, StreamExt};
use time::UtcDateTime;
use tracing::info;
use uuid::Uuid;

use arkin_core::prelude::*;

use arkin_core::PersistenceError;

use crate::{context::PersistenceContext, repos::ch::tick_repo, stores::instrument_store};

pub async fn insert(ctx: &PersistenceContext, tick: Arc<Tick>) -> Result<(), PersistenceError> {
    tick_repo::insert(ctx, tick.into()).await?;
    Ok(())
}

pub async fn insert_vec(ctx: &PersistenceContext, ticks: &[Arc<Tick>]) -> Result<(), PersistenceError> {
    let ticks = ticks.into_iter().cloned().map(|t| t.into()).collect::<Vec<_>>();
    tick_repo::insert_batch(ctx, &ticks).await
}

pub async fn read_last(
    ctx: &PersistenceContext,
    instrument: &Arc<Instrument>,
) -> Result<Option<Arc<Tick>>, PersistenceError> {
    match tick_repo::read_last(ctx, &instrument.id).await? {
        Some(dto) => {
            let tick = Tick::builder()
                .event_time(dto.event_time.to_utc())
                .instrument(Arc::clone(instrument))
                .tick_id(dto.tick_id as u64)
                .bid_price(dto.bid_price)
                .bid_quantity(dto.bid_quantity)
                .ask_price(dto.ask_price)
                .ask_quantity(dto.ask_quantity)
                .build();
            Ok(Some(Arc::new(tick)))
        }
        None => Ok(None),
    }
}

pub async fn read_range(
    ctx: &PersistenceContext,
    instrument_ids: &[Uuid],
    from: UtcDateTime,
    to: UtcDateTime,
) -> Result<Vec<Arc<Tick>>, PersistenceError> {
    let db_ticks = tick_repo::read_range(ctx, &instrument_ids, from, to).await?;
    let mut ticks = Vec::with_capacity(db_ticks.len());
    for dto in &db_ticks {
        let instrument = instrument_store::read_by_id(ctx, &dto.instrument_id).await?;
        let tick = Tick::builder()
            .event_time(dto.event_time.to_utc())
            .instrument(instrument)
            .tick_id(dto.tick_id as u64)
            .bid_price(dto.bid_price)
            .bid_quantity(dto.bid_quantity)
            .ask_price(dto.ask_price)
            .ask_quantity(dto.ask_quantity)
            .build();
        ticks.push(Arc::new(tick));
    }
    Ok(ticks)
}

pub async fn stream_range(
    ctx: &PersistenceContext,
    instruments: &[Arc<Instrument>],
    from: UtcDateTime,
    to: UtcDateTime,
) -> Result<impl Stream<Item = Result<Arc<Tick>, PersistenceError>> + 'static, PersistenceError> {
    let ids = instruments.iter().map(|i| i.id).collect::<Vec<_>>();
    let mut cursor = tick_repo::stream_range(ctx, &ids, from, to).await?;

    // Build a stream that yields ticks.
    let ctx_clone = ctx.clone();
    let stream = stream! {
        loop {
            match cursor.next().await {
                Ok(Some(row)) => {
                    // For each row, do your transformations.
                    match instrument_store::read_by_id(&ctx_clone, &row.instrument_id).await {
                        Ok(instrument) => {
                            let tick = Tick::builder()
                                .event_time(row.event_time.to_utc())
                                .instrument(instrument)
                                .tick_id(row.tick_id as u64)
                                .bid_price(row.bid_price)
                                .bid_quantity(row.bid_quantity)
                                .ask_price(row.ask_price)
                                .ask_quantity(row.ask_quantity)
                                .build();
                            let tick_arc = Arc::new(tick);
                            yield Ok(tick_arc);
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
) -> Box<dyn Stream<Item = Arc<Tick>> + Send + Unpin> {
    // Split the range into daily chunks
    let time_chunks = datetime_chunks(start, end, frequency).unwrap();
    let instrument_ids = Arc::new(instruments.iter().map(|i| i.id).collect::<Vec<_>>());
    let local_instrument_lookup =
        Arc::new(instruments.iter().map(|i| (i.id, Arc::clone(i))).collect::<HashMap<_, _>>());

    // Create a stream of futures for each daily chunk
    let ctx_clone = ctx.clone();
    let fetch_stream = stream::iter(time_chunks).map(move |(start_batch, end_batch)| {
        let ctx_clone = ctx_clone.clone();
        let instrument_ids = instrument_ids.clone();
        let local_instrument_lookup = local_instrument_lookup.clone();

        async move {
            info!("Fetching ticks for batch: {} - {}", start_batch, end_batch);

            // Fetch with retries
            let res = retry(
                || tick_repo::fetch_batch(&ctx_clone, &instrument_ids, start_batch, end_batch),
                5, // Max retries
            )
            .await;

            let batch = res.expect("Failed to fetch batch, abort mission");
            let mut ticks = Vec::with_capacity(batch.len());
            for dto in batch {
                let instrument = local_instrument_lookup.get(&dto.instrument_id).cloned().unwrap();
                let tick = Tick::builder()
                    .event_time(dto.event_time.to_utc())
                    .instrument(instrument)
                    .tick_id(dto.tick_id as u64)
                    .bid_price(dto.bid_price)
                    .bid_quantity(dto.bid_quantity)
                    .ask_price(dto.ask_price)
                    .ask_quantity(dto.ask_quantity)
                    .build();
                ticks.push(Arc::new(tick));
            }
            ticks
        }
    });
    Box::new(fetch_stream.buffered(buffer_size).flat_map(|x| stream::iter(x)))
}
