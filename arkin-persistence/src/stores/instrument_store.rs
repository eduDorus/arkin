use std::sync::Arc;

use tracing::debug;
use uuid::Uuid;

use arkin_core::{Instrument, InstrumentQuery, PersistenceError, Venue};

use crate::{
    context::PersistenceContext,
    repos::pg::instrument_repo::{self, InstrumentDTO},
    stores::{asset_store, venue_store},
};

async fn update_instrument_cache(ctx: &PersistenceContext, instrument: Arc<Instrument>) {
    ctx.cache.instrument_id.insert(instrument.id, instrument.clone()).await;
    ctx.cache
        .instrument_venue_symbol
        .insert(instrument.venue_symbol.clone(), instrument.clone())
        .await;
}

/// Convert InstrumentDTO to Arc<Instrument> by loading related entities
async fn dto_to_instrument(ctx: &PersistenceContext, dto: InstrumentDTO) -> Result<Arc<Instrument>, PersistenceError> {
    let venue = venue_store::read_by_id(ctx, &dto.venue_id).await?;
    let base_asset = asset_store::read_by_id(ctx, &dto.base_asset_id).await?;
    let quote_asset = asset_store::read_by_id(ctx, &dto.quote_asset_id).await?;
    let margin_asset = asset_store::read_by_id(ctx, &dto.margin_asset_id).await?;

    let instrument = Instrument {
        id: dto.id,
        symbol: dto.symbol,
        venue_symbol: dto.venue_symbol,
        venue,
        instrument_type: dto.instrument_type,
        synthetic: dto.synthetic,
        base_asset,
        quote_asset,
        margin_asset,
        maturity: dto.maturity.map(|m| m.to_utc()),
        strike: dto.strike,
        option_type: dto.option_type,
        contract_size: dto.contract_size,
        price_precision: dto.price_precision as u32,
        quantity_precision: dto.quantity_precision as u32,
        base_precision: dto.base_precision as u32,
        quote_precision: dto.quote_precision as u32,
        tick_size: dto.tick_size,
        lot_size: dto.lot_size,
        status: dto.status,
        created: dto.created.to_utc(),
        updated: dto.updated.to_utc(),
    };

    Ok(Arc::new(instrument))
}

/// Load all instruments from database into cache
/// This should be called once during persistence initialization
pub async fn load_instruments(ctx: &PersistenceContext) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
    let instrument_dtos = instrument_repo::list_all(ctx).await?;
    let mut instruments = Vec::with_capacity(instrument_dtos.len());

    for dto in instrument_dtos {
        let instrument = dto_to_instrument(ctx, dto).await?;
        update_instrument_cache(ctx, instrument.clone()).await;
        instruments.push(instrument);
    }

    Ok(instruments)
}

/// Query instruments with in-memory filtering from cache
/// Assumes cache is already populated via load_instruments()
pub async fn query(
    ctx: &PersistenceContext,
    query: &InstrumentQuery,
) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
    // Get all cached instruments by iterating over the cache
    let all_instruments: Vec<Arc<Instrument>> =
        ctx.cache.instrument_id.iter().map(|(_, instrument)| instrument).collect();

    // If query is empty, return all
    if query.is_empty() {
        return Ok(all_instruments);
    }

    // Filter in memory using the query's matches method
    let filtered: Vec<Arc<Instrument>> = all_instruments
        .into_iter()
        .filter(|instrument| query.matches(instrument))
        .collect();

    Ok(filtered)
}

async fn read_cache_by_id(ctx: &PersistenceContext, id: &Uuid) -> Option<Arc<Instrument>> {
    ctx.cache.instrument_id.get(id).await
}

async fn read_cache_by_venue_symbol(ctx: &PersistenceContext, venue_symbol: &str) -> Option<Arc<Instrument>> {
    ctx.cache.instrument_venue_symbol.get(venue_symbol).await
}

pub async fn insert(ctx: &PersistenceContext, instrument: Arc<Instrument>) -> Result<(), PersistenceError> {
    update_instrument_cache(ctx, instrument.clone()).await;
    instrument_repo::insert(ctx, instrument.into()).await
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<Arc<Instrument>, PersistenceError> {
    // Check cache
    match read_cache_by_id(ctx, id).await {
        Some(instrument) => Ok(instrument),
        None => {
            debug!("Instrument not found in cache");
            let instrument_dto = instrument_repo::read_by_id(ctx, id).await?;
            let instrument = dto_to_instrument(ctx, instrument_dto).await?;

            // Update cache
            update_instrument_cache(ctx, instrument.clone()).await;
            Ok(instrument)
        }
    }
}

pub async fn read_by_venue_symbol(
    ctx: &PersistenceContext,
    venue_symbol: &str,
    venue: &Arc<Venue>,
) -> Result<Arc<Instrument>, PersistenceError> {
    // Check cache
    match read_cache_by_venue_symbol(ctx, venue_symbol).await {
        Some(instrument) => Ok(instrument),
        None => {
            debug!("Instrument not found in cache");
            let instrument_dto = instrument_repo::read_by_venue_symbol(ctx, venue_symbol, venue).await?;
            let instrument = dto_to_instrument(ctx, instrument_dto).await?;

            // Update cache
            update_instrument_cache(ctx, instrument.clone()).await;
            Ok(instrument)
        }
    }
}

pub async fn list_by_venue(
    ctx: &PersistenceContext,
    venue: &Arc<Venue>,
) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
    let instrument_dtos = instrument_repo::list_by_venue(ctx, venue).await?;
    let mut instruments = Vec::with_capacity(instrument_dtos.len());
    for dto in instrument_dtos {
        let instrument = dto_to_instrument(ctx, dto).await?;
        instruments.push(instrument);
    }
    Ok(instruments)
}

pub async fn list_by_venue_and_type(
    ctx: &PersistenceContext,
    venue: &Arc<Venue>,
    instrument_type: arkin_core::InstrumentType,
) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
    let instrument_dtos = instrument_repo::list_by_venue_and_type(ctx, venue, instrument_type).await?;
    let mut instruments = Vec::with_capacity(instrument_dtos.len());
    for dto in instrument_dtos {
        let instrument = dto_to_instrument(ctx, dto).await?;
        instruments.push(instrument);
    }
    Ok(instruments)
}
