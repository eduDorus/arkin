use std::{fmt, sync::Arc};

use rust_decimal::prelude::Decimal;
use sqlx::Type;
use strum::Display;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{Maturity, Price};

use super::{Asset, Venue};

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "instrument_type", rename_all = "snake_case")]
pub enum InstrumentType {
    Spot,
    Perpetual,
    Future,
    Option,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "instrument_option_type", rename_all = "snake_case")]
pub enum InstrumentOptionType {
    Call,
    Put,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "instrument_status", rename_all = "snake_case")]
pub enum InstrumentStatus {
    Trading,
    Halted,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypedBuilder)]
pub struct Instrument {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub venue: Arc<Venue>,
    pub symbol: String,
    pub venue_symbol: String,
    pub instrument_type: InstrumentType,
    pub base_asset: Arc<Asset>,
    pub quote_asset: Arc<Asset>,
    pub margin_asset: Arc<Asset>,
    pub maturity: Option<Maturity>,
    pub strike: Option<Price>,
    pub option_type: Option<InstrumentOptionType>,
    pub contract_size: Decimal,
    pub price_precision: u32,
    pub quantity_precision: u32,
    pub base_precision: u32,
    pub quote_precision: u32,
    pub tick_size: Decimal,
    pub lot_size: Decimal,
    pub status: InstrumentStatus,
}

impl fmt::Display for Instrument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}
