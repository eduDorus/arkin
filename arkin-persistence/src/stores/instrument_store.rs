use std::sync::Arc;

use tracing::debug;
use uuid::Uuid;

use arkin_core::{Instrument, PersistenceError, Venue};

use crate::{
    context::PersistenceContext,
    repos::pg::instrument_repo,
    stores::{asset_store, venue_store},
};

async fn update_instrument_cache(ctx: &PersistenceContext, instrument: Arc<Instrument>) {
    ctx.cache.instrument_id.insert(instrument.id, instrument.clone()).await;
    ctx.cache
        .instrument_venue_symbol
        .insert(instrument.venue_symbol.clone(), instrument.clone())
        .await;
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
            let venue = venue_store::read_by_id(ctx, &instrument_dto.venue_id).await?;

            let base_asset = asset_store::read_by_id(ctx, &instrument_dto.base_asset_id).await?;
            let quote_asset = asset_store::read_by_id(ctx, &instrument_dto.quote_asset_id).await?;
            let margin_asset = asset_store::read_by_id(ctx, &instrument_dto.margin_asset_id).await?;

            let instrument = Instrument {
                id: instrument_dto.id,
                symbol: instrument_dto.symbol,
                venue_symbol: instrument_dto.venue_symbol,
                venue,
                instrument_type: instrument_dto.instrument_type,
                synthetic: instrument_dto.synthetic,
                base_asset,
                quote_asset,
                margin_asset,
                maturity: instrument_dto.maturity.map(|m| m.to_utc()),
                strike: instrument_dto.strike,
                option_type: instrument_dto.option_type.map(|v| v),
                contract_size: instrument_dto.contract_size,
                price_precision: instrument_dto.price_precision as u32,
                quantity_precision: instrument_dto.quantity_precision as u32,
                base_precision: instrument_dto.base_precision as u32,
                quote_precision: instrument_dto.quote_precision as u32,
                tick_size: instrument_dto.tick_size,
                lot_size: instrument_dto.lot_size,
                status: instrument_dto.status,
                created: instrument_dto.created.to_utc(),
                updated: instrument_dto.updated.to_utc(),
            };

            // Update cache
            let instrument = Arc::new(instrument);
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
            let venue = venue_store::read_by_id(ctx, &instrument_dto.venue_id).await?;

            let base_asset = asset_store::read_by_id(ctx, &instrument_dto.base_asset_id).await?;
            let quote_asset = asset_store::read_by_id(ctx, &instrument_dto.quote_asset_id).await?;
            let margin_asset = asset_store::read_by_id(ctx, &instrument_dto.margin_asset_id).await?;

            let instrument = Instrument {
                id: instrument_dto.id,
                symbol: instrument_dto.symbol,
                venue_symbol: instrument_dto.venue_symbol,
                venue,
                instrument_type: instrument_dto.instrument_type,
                synthetic: instrument_dto.synthetic,
                base_asset,
                quote_asset,
                margin_asset,
                maturity: instrument_dto.maturity.map(|m| m.to_utc()),
                strike: instrument_dto.strike,
                option_type: instrument_dto.option_type.map(|v| v),
                contract_size: instrument_dto.contract_size,
                price_precision: instrument_dto.price_precision as u32,
                quantity_precision: instrument_dto.quantity_precision as u32,
                base_precision: instrument_dto.base_precision as u32,
                quote_precision: instrument_dto.quote_precision as u32,
                tick_size: instrument_dto.tick_size,
                lot_size: instrument_dto.lot_size,
                status: instrument_dto.status,
                created: instrument_dto.created.to_utc(),
                updated: instrument_dto.updated.to_utc(),
            };

            // Update cache
            let instrument = Arc::new(instrument);
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
    for dto in &instrument_dtos {
        let instrument = read_by_id(ctx, &dto.id).await?;
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
    for dto in &instrument_dtos {
        let instrument = read_by_id(ctx, &dto.id).await?;
        instruments.push(instrument);
    }
    Ok(instruments)
}
