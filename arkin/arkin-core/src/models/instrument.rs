use std::fmt;

use rust_decimal::prelude::Decimal;
use strum::Display;

use crate::{Maturity, Price};

use super::Venue;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Instrument {
    pub id: u32,
    pub venue: Venue,
    pub symbol: String,
    pub venue_symbol: String,
    pub contract_type: ContractType,
    pub base_asset: String,
    pub quote_asset: String,
    pub strike: Option<Price>,
    pub maturity: Option<Maturity>,
    pub option_type: Option<OptionType>,
    pub contract_size: Decimal,
    pub price_precision: u32,
    pub quantity_precision: u32,
    pub base_precision: u32,
    pub quote_precision: u32,
    pub lot_size: Decimal,
    pub tick_size: Decimal,
    pub status: InstrumentStatus,
}

#[derive(Clone, Display, PartialEq, Eq, Hash)]
pub enum ContractType {
    Spot,
    Perpetual,
    Future,
    Option,
}

#[derive(Clone, Display, PartialEq, Eq, Hash)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Clone, Display, PartialEq, Eq, Hash)]
pub enum InstrumentStatus {
    PendingTrading,
    Trading,
    Halted,
    PreDelivering,
    Delivering,
    Delivered,
    PreSettle,
    Settling,
    Close,
}

impl fmt::Display for Instrument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}
