use anyhow::Result;
use arkin_core::prelude::*;
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "instrument_type", rename_all = "snake_case")]
pub enum DBInstrumentType {
    Spot,
    Perpetual,
    Future,
    Option,
}

impl From<InstrumentType> for DBInstrumentType {
    fn from(v: InstrumentType) -> Self {
        match v {
            InstrumentType::Spot => Self::Spot,
            InstrumentType::Perpetual => Self::Perpetual,
            InstrumentType::Future => Self::Future,
            InstrumentType::Option => Self::Option,
        }
    }
}

impl From<DBInstrumentType> for InstrumentType {
    fn from(v: DBInstrumentType) -> Self {
        match v {
            DBInstrumentType::Spot => Self::Spot,
            DBInstrumentType::Perpetual => Self::Perpetual,
            DBInstrumentType::Future => Self::Future,
            DBInstrumentType::Option => Self::Option,
        }
    }
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "option_type", rename_all = "snake_case")]
pub enum DBOptionType {
    Call,
    Put,
}

impl From<OptionType> for DBOptionType {
    fn from(option_type: OptionType) -> Self {
        match option_type {
            OptionType::Call => Self::Call,
            OptionType::Put => Self::Put,
        }
    }
}

impl From<DBOptionType> for OptionType {
    fn from(option_type: DBOptionType) -> Self {
        match option_type {
            DBOptionType::Call => Self::Call,
            DBOptionType::Put => Self::Put,
        }
    }
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "instrument_status", rename_all = "snake_case")]
pub enum DBInstrumentStatus {
    Trading,
    Halted,
}

impl From<InstrumentStatus> for DBInstrumentStatus {
    fn from(status: InstrumentStatus) -> Self {
        match status {
            InstrumentStatus::Trading => Self::Trading,
            InstrumentStatus::Halted => Self::Halted,
        }
    }
}

impl From<DBInstrumentStatus> for InstrumentStatus {
    fn from(status: DBInstrumentStatus) -> Self {
        match status {
            DBInstrumentStatus::Trading => Self::Trading,
            DBInstrumentStatus::Halted => Self::Halted,
        }
    }
}

#[derive(FromRow)]
pub struct DBInstrument {
    pub id: Uuid,
    pub venue_id: Uuid,
    pub symbol: String,
    pub venue_symbol: String,
    pub instrument_type: DBInstrumentType,
    pub base_asset: String,
    pub quote_asset: String,
    pub strike: Option<Price>,
    pub maturity: Option<OffsetDateTime>,
    pub option_type: Option<DBOptionType>,
    pub contract_size: Quantity,
    pub price_precision: i32,
    pub quantity_precision: i32,
    pub base_precision: i32,
    pub quote_precision: i32,
    pub lot_size: Quantity,
    pub tick_size: Price,
    pub status: DBInstrumentStatus,
}

impl From<Instrument> for DBInstrument {
    fn from(instrument: Instrument) -> Self {
        Self {
            id: instrument.id,
            venue_id: instrument.venue.id,
            symbol: instrument.symbol,
            venue_symbol: instrument.venue_symbol,
            instrument_type: instrument.instrument_type.into(),
            base_asset: instrument.base_asset,
            quote_asset: instrument.quote_asset,
            strike: instrument.strike,
            maturity: instrument.maturity,
            option_type: instrument.option_type.map(|v| v.into()),
            contract_size: instrument.contract_size,
            price_precision: instrument.price_precision as i32,
            quantity_precision: instrument.quantity_precision as i32,
            base_precision: instrument.base_precision as i32,
            quote_precision: instrument.quote_precision as i32,
            lot_size: instrument.lot_size,
            tick_size: instrument.tick_size,
            status: instrument.status.into(),
        }
    }
}

pub struct InstrumentRepo {
    pool: PgPool,
}

impl InstrumentRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, instrument: Instrument) -> Result<()> {
        let instrument = DBInstrument::from(instrument);
        sqlx::query!(
            r#"
            INSERT INTO instruments (
                id, venue_id, symbol, venue_symbol, instrument_type, base_asset, quote_asset, strike, maturity, option_type,
                contract_size, price_precision, quantity_precision, base_precision, quote_precision, lot_size, tick_size, status
            ) VALUES (
                $1, $2, $3, $4, $5::instrument_type, $6, $7, $8, $9, $10::option_type,
                $11, $12, $13, $14, $15, $16, $17, $18::instrument_status
            )
            "#,
            instrument.id,
            instrument.venue_id,
            instrument.symbol,
            instrument.venue_symbol,
            instrument.instrument_type as DBInstrumentType,
            instrument.base_asset,
            instrument.quote_asset,
            instrument.strike,
            instrument.maturity,
            instrument.option_type as Option<DBOptionType>,
            instrument.contract_size,
            instrument.price_precision,
            instrument.quantity_precision,
            instrument.base_precision,
            instrument.quote_precision,
            instrument.lot_size,
            instrument.tick_size,
            instrument.status as DBInstrumentStatus
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Option<DBInstrument>> {
        let instrument = sqlx::query_as!(
            DBInstrument,
            r#"
            SELECT
                id,
                venue_id,
                symbol,
                venue_symbol,
                instrument_type AS "instrument_type:DBInstrumentType",
                base_asset,
                quote_asset,
                strike,
                maturity,
                option_type AS "option_type:DBOptionType",
                contract_size,
                price_precision,
                quantity_precision,
                base_precision,
                quote_precision,
                lot_size,
                tick_size,
                status AS "status:DBInstrumentStatus"
            FROM instruments
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(instrument)
    }
}
