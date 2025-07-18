use std::{collections::HashMap, sync::Arc};

use async_stream::try_stream;
use futures::{stream, Stream, StreamExt};
use time::UtcDateTime;
use tracing::info;

use arkin_core::prelude::*;

use crate::{context::PersistenceContext, repos::ch::trade_repo, stores::instrument_store, PersistenceError};

pub async fn insert(ctx: &PersistenceContext, trade: Arc<Trade>) -> Result<(), PersistenceError> {
    trade_repo::insert(ctx, trade.into()).await
}

pub async fn insert_vec(ctx: &PersistenceContext, trades: &[Arc<Trade>]) -> Result<(), PersistenceError> {
    let ticks = trades.into_iter().cloned().map(|t| t.into()).collect::<Vec<_>>();
    trade_repo::insert_batch(ctx, &ticks).await
}

pub async fn read_range(
    ctx: &PersistenceContext,
    instruments: &[Arc<Instrument>],
    from: UtcDateTime,
    to: UtcDateTime,
) -> Result<Vec<Arc<Trade>>, PersistenceError> {
    let ids = instruments.iter().map(|i| i.id).collect::<Vec<_>>();
    let dto = trade_repo::read_range(ctx, &ids, from, to).await?;

    let mut trades = Vec::with_capacity(dto.len());
    for trade in &dto {
        let instrument = instrument_store::read_by_id(ctx, &trade.instrument_id).await?;
        let trade = Trade::builder()
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
) -> Result<impl Stream<Item = Result<Arc<Trade>, PersistenceError>> + 'static, PersistenceError> {
    // We do not `async` here, because returning `impl Stream` + `'a` from an `async fn`
    // is not yet stable. Instead, we return a non-async function that constructs the stream.

    // Collect the IDs.
    let ids = instruments.iter().map(|i| i.id).collect::<Vec<_>>();
    let mut cursor = trade_repo::stream_range(ctx, &ids, from, to).await?;

    // Build a "try_stream" that yields trades.
    let ctx_clone = ctx.clone();
    let stream = try_stream! {
        // Get the async cursor from the repository.

        // Loop over rows in the cursor.
        while let Some(row) = cursor.next().await? {
            // For each row, do your transformations.
            let instrument = instrument_store::read_by_id(&ctx_clone, &row.instrument_id).await?;
            let trade = Trade::builder()
                .event_time(row.event_time.to_utc())
                .instrument(instrument)
                .trade_id(row.trade_id as u64)
                .side(row.side.into())
                .price(row.price)
                .quantity(row.quantity)
                .build();

            // Yield the constructed trade to the stream.
            yield Arc::new(trade);
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
) -> impl Stream<Item = Arc<Trade>> + 'static {
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
            info!("Fetching trades for batch: {} - {}", start_batch, end_batch);

            // Fetch with retries
            let res = retry(
                || trade_repo::fetch_batch(&ctx_clone, &instrument_ids, start_batch, end_batch),
                5, // Max retries
            )
            .await;

            let batch = res.expect("Failed to fetch batch, abort mission");
            let mut trades = Vec::with_capacity(batch.len());
            for dto in batch {
                let instrument = local_instrument_lookup.get(&dto.instrument_id).cloned().unwrap();
                let trade = Trade::builder()
                    .event_time(dto.event_time.to_utc())
                    .instrument(instrument)
                    .trade_id(dto.trade_id as u64)
                    .side(dto.side.into())
                    .price(dto.price)
                    .quantity(dto.quantity)
                    .build();
                trades.push(Arc::new(trade));
            }
            trades
        }
    });
    fetch_stream.buffered(buffer_size).flat_map(|x| stream::iter(x))
}
