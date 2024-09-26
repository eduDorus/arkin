use arkin_core::prelude::*;
use sqlx::prelude::*;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "contract_type", rename_all = "snake_case")]
pub enum DBContractType {
    Spot,
    Perpetual,
    Future,
    Option,
}

impl From<InstrumentType> for DBContractType {
    fn from(contract_type: InstrumentType) -> Self {
        match contract_type {
            InstrumentType::Spot => Self::Spot,
            InstrumentType::Perpetual => Self::Perpetual,
            InstrumentType::Future => Self::Future,
            InstrumentType::Option => Self::Option,
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

#[derive(FromRow)]
pub struct DBInstrument {
    pub id: Uuid,
    pub venue: Uuid,
    pub symbol: String,
    pub venue_symbol: String,
    pub contract_type: DBContractType,
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
            venue: instrument.venue.id,
            symbol: instrument.symbol,
            venue_symbol: instrument.venue_symbol,
            contract_type: instrument.contract_type.into(),
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
