use std::sync::Arc;

use rust_decimal::Decimal;
use sqlx::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, info};
use uuid::Uuid;

use arkin_core::{Instrument, InstrumentOptionType, InstrumentStatus, InstrumentType, Price, Venue};

use arkin_core::PersistenceError;

use crate::context::PersistenceContext;

#[derive(FromRow)]
pub struct InstrumentDTO {
    pub id: Uuid,
    pub venue_id: Uuid,
    pub symbol: String,
    pub venue_symbol: String,
    pub instrument_type: InstrumentType,
    pub synthetic: bool,
    pub base_asset_id: Uuid,
    pub quote_asset_id: Uuid,
    pub margin_asset_id: Uuid,
    pub strike: Option<Decimal>,
    pub maturity: Option<OffsetDateTime>,
    pub option_type: Option<InstrumentOptionType>,
    pub contract_size: Decimal,
    pub price_precision: i32,
    pub quantity_precision: i32,
    pub base_precision: i32,
    pub quote_precision: i32,
    pub lot_size: Decimal,
    pub tick_size: Price,
    pub status: InstrumentStatus,
    pub created: OffsetDateTime,
    pub updated: OffsetDateTime,
}

impl From<Arc<Instrument>> for InstrumentDTO {
    fn from(instrument: Arc<Instrument>) -> Self {
        Self {
            id: instrument.id,
            venue_id: instrument.venue.id,
            symbol: instrument.symbol.clone(),
            venue_symbol: instrument.venue_symbol.clone(),
            instrument_type: instrument.instrument_type.clone(),
            synthetic: instrument.synthetic,
            base_asset_id: instrument.base_asset.id,
            quote_asset_id: instrument.quote_asset.id,
            margin_asset_id: instrument.margin_asset.id,
            strike: instrument.strike,
            maturity: instrument.maturity.map(|m| m.into()),
            option_type: instrument.option_type.as_ref().map(|v| v.clone()),
            contract_size: instrument.contract_size,
            price_precision: instrument.price_precision as i32,
            quantity_precision: instrument.quantity_precision as i32,
            base_precision: instrument.base_precision as i32,
            quote_precision: instrument.quote_precision as i32,
            lot_size: instrument.lot_size,
            tick_size: instrument.tick_size,
            status: instrument.status.clone(),
            created: instrument.created.into(),
            updated: instrument.updated.into(),
        }
    }
}

pub async fn insert(ctx: &PersistenceContext, instrument: InstrumentDTO) -> Result<(), PersistenceError> {
    let instrument = InstrumentDTO::from(instrument);
    sqlx::query!(
            r#"
            INSERT INTO instruments (
                id, venue_id, symbol, venue_symbol, instrument_type, synthetic, base_asset_id, quote_asset_id, margin_asset_id, strike, maturity, option_type,
                contract_size, price_precision, quantity_precision, base_precision, quote_precision, lot_size, tick_size, status
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,$11, $12, $13, $14, $15, $16, $17, $18, $19, $20
            )
            "#,
            instrument.id,
            instrument.venue_id,
            instrument.symbol,
            instrument.venue_symbol,
            instrument.instrument_type as InstrumentType,
            instrument.synthetic,
            instrument.base_asset_id,
            instrument.quote_asset_id,
            instrument.margin_asset_id,
            instrument.strike,
            instrument.maturity,
            instrument.option_type as Option<InstrumentOptionType>,
            instrument.contract_size,
            instrument.price_precision,
            instrument.quantity_precision,
            instrument.base_precision,
            instrument.quote_precision,
            instrument.lot_size,
            instrument.tick_size,
            instrument.status as InstrumentStatus
        )
        .execute(&ctx.pg_pool)
        .await?;
    Ok(())
}

pub async fn read_by_id(ctx: &PersistenceContext, id: &Uuid) -> Result<InstrumentDTO, PersistenceError> {
    let instrument = sqlx::query_as!(
        InstrumentDTO,
        r#"
            SELECT
                id,
                venue_id,
                symbol,
                venue_symbol,
                instrument_type AS "instrument_type:InstrumentType",
                synthetic,
                base_asset_id,
                quote_asset_id,
                margin_asset_id,
                strike,
                maturity,
                option_type AS "option_type:InstrumentOptionType",
                contract_size,
                price_precision,
                quantity_precision,
                base_precision,
                quote_precision,
                lot_size,
                tick_size,
                status AS "status:InstrumentStatus",
                created,
                updated
            FROM instruments
            WHERE id = $1
            "#,
        id,
    )
    .fetch_optional(&ctx.pg_pool)
    .await?;

    match instrument {
        Some(instrument) => Ok(instrument),
        None => Err(PersistenceError::NotFound),
    }
}

pub async fn read_by_venue_symbol(
    ctx: &PersistenceContext,
    symbol: &str,
    venue: &Arc<Venue>,
) -> Result<InstrumentDTO, PersistenceError> {
    info!("Instrument repo reading instrument by venue symbol: {}", symbol);
    let instrument = sqlx::query_as!(
        InstrumentDTO,
        r#"
            SELECT
                id,
                venue_id,
                symbol,
                venue_symbol,
                instrument_type AS "instrument_type:InstrumentType",
                synthetic,
                base_asset_id,
                quote_asset_id,
                margin_asset_id,
                strike,
                maturity,
                option_type AS "option_type:InstrumentOptionType",
                contract_size,
                price_precision,
                quantity_precision,
                base_precision,
                quote_precision,
                lot_size,
                tick_size,
                status AS "status:InstrumentStatus",
                created,
                updated
            FROM instruments
            WHERE venue_symbol = $1 AND venue_id = $2
            "#,
        symbol,
        venue.id as Uuid,
    )
    .fetch_optional(&ctx.pg_pool)
    .await?;

    match instrument {
        Some(instrument) => Ok(instrument),
        None => Err(PersistenceError::NotFound),
    }
}

pub async fn list_by_venue(
    ctx: &PersistenceContext,
    venue: &Arc<Venue>,
) -> Result<Vec<InstrumentDTO>, PersistenceError> {
    debug!("Instrument repo listing instruments by venue: {}", venue.name);
    let instruments = sqlx::query_as!(
        InstrumentDTO,
        r#"
            SELECT
                id,
                venue_id,
                symbol,
                venue_symbol,
                instrument_type AS "instrument_type:InstrumentType",
                synthetic,
                base_asset_id,
                quote_asset_id,
                margin_asset_id,
                strike,
                maturity,
                option_type AS "option_type:InstrumentOptionType",
                contract_size,
                price_precision,
                quantity_precision,
                base_precision,
                quote_precision,
                lot_size,
                tick_size,
                status AS "status:InstrumentStatus",
                created,
                updated
            FROM instruments
            WHERE venue_id = $1
            "#,
        venue.id as Uuid,
    )
    .fetch_all(&ctx.pg_pool)
    .await?;

    Ok(instruments)
}

pub async fn list_by_venue_and_type(
    ctx: &PersistenceContext,
    venue: &Arc<Venue>,
    instrument_type: InstrumentType,
) -> Result<Vec<InstrumentDTO>, PersistenceError> {
    debug!(
        "Instrument repo listing instruments by venue: {} and type: {:?}",
        venue.name, instrument_type
    );
    let instruments = sqlx::query_as!(
        InstrumentDTO,
        r#"
            SELECT
                id,
                venue_id,
                symbol,
                venue_symbol,
                instrument_type AS "instrument_type:InstrumentType",
                synthetic,
                base_asset_id,
                quote_asset_id,
                margin_asset_id,
                strike,
                maturity,
                option_type AS "option_type:InstrumentOptionType",
                contract_size,
                price_precision,
                quantity_precision,
                base_precision,
                quote_precision,
                lot_size,
                tick_size,
                status AS "status:InstrumentStatus",
                created,
                updated
            FROM instruments
            WHERE venue_id = $1 AND instrument_type = $2
            "#,
        venue.id as Uuid,
        instrument_type as InstrumentType,
    )
    .fetch_all(&ctx.pg_pool)
    .await?;

    Ok(instruments)
}
