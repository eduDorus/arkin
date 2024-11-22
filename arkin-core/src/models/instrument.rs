use std::fmt;

use derive_builder::Builder;
use rust_decimal::prelude::Decimal;
use strum::Display;
use uuid::Uuid;

use crate::{prelude::INSTRUMENT_TIMESTAMP_FORMAT, types::AssetId, Maturity, Price};

use super::Venue;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Builder)]
#[builder(setter(into))]
pub struct Instrument {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub venue: Venue,
    pub symbol: String,
    pub venue_symbol: String,
    pub instrument_type: InstrumentType,
    pub base_asset: AssetId,
    pub quote_asset: AssetId,
    pub maturity: Option<Maturity>,
    pub strike: Option<Price>,
    pub option_type: Option<OptionType>,
    pub contract_size: Decimal,
    pub price_precision: u32,
    pub quantity_precision: u32,
    pub base_precision: u32,
    pub quote_precision: u32,
    pub tick_size: Decimal,
    pub lot_size: Decimal,
    pub status: InstrumentStatus,
}

impl Instrument {
    pub fn new_spot(
        venue: Venue,
        venue_symbol: String,
        base_asset: AssetId,
        quote_asset: AssetId,
        price_precision: u32,
        quantity_precision: u32,
        base_precision: u32,
        quote_precision: u32,
        tick_size: Decimal,
        lot_size: Decimal,
    ) -> Self {
        Instrument {
            id: Uuid::new_v4(),
            symbol: format!("SPOT-{}-{}@{}", base_asset, quote_asset, venue),
            venue,
            venue_symbol,
            instrument_type: InstrumentType::Spot,
            base_asset,
            quote_asset,
            maturity: None,
            strike: None,
            option_type: None,
            contract_size: Decimal::ZERO,
            price_precision,
            quantity_precision,
            base_precision,
            quote_precision,
            tick_size,
            lot_size,
            status: InstrumentStatus::Trading,
        }
    }

    pub fn new_perpetual(
        venue: Venue,
        venue_symbol: String,
        base_asset: AssetId,
        quote_asset: AssetId,
        price_precision: u32,
        quantity_precision: u32,
        base_precision: u32,
        quote_precision: u32,
        tick_size: Decimal,
        lot_size: Decimal,
    ) -> Self {
        Instrument {
            id: Uuid::new_v4(),
            symbol: format!("PERP-{}-{}@{}", base_asset, quote_asset, venue),
            venue,
            venue_symbol,
            instrument_type: InstrumentType::Perpetual,
            base_asset,
            quote_asset,
            maturity: None,
            strike: None,
            option_type: None,
            contract_size: Decimal::ZERO,
            price_precision,
            quantity_precision,
            base_precision,
            quote_precision,
            tick_size,
            lot_size,
            status: InstrumentStatus::Trading,
        }
    }

    pub fn new_future(
        venue: Venue,
        venue_symbol: String,
        base_asset: AssetId,
        quote_asset: AssetId,
        maturity: Maturity,
        price_precision: u32,
        quantity_precision: u32,
        base_precision: u32,
        quote_precision: u32,
        tick_size: Decimal,
        lot_size: Decimal,
    ) -> Self {
        Instrument {
            id: Uuid::new_v4(),
            symbol: format!(
                "FUTURE-{}-{}-{}@{}",
                base_asset,
                quote_asset,
                maturity.format(INSTRUMENT_TIMESTAMP_FORMAT).expect("Failed to format time"),
                venue
            ),
            venue,
            venue_symbol,
            instrument_type: InstrumentType::Future,
            base_asset,
            quote_asset,
            maturity: Some(maturity),
            strike: None,
            option_type: None,
            contract_size: Decimal::ZERO,
            price_precision,
            quantity_precision,
            base_precision,
            quote_precision,
            tick_size,
            lot_size,
            status: InstrumentStatus::Trading,
        }
    }

    pub fn new_option(
        venue: Venue,
        venue_symbol: String,
        base_asset: AssetId,
        quote_asset: AssetId,
        maturity: Maturity,
        strike: Price,
        option_type: OptionType,
        price_precision: u32,
        quantity_precision: u32,
        base_precision: u32,
        quote_precision: u32,
        tick_size: Decimal,
        lot_size: Decimal,
    ) -> Self {
        Instrument {
            id: Uuid::new_v4(),
            symbol: format!(
                "OPTION-{}-{}-{}-{}-{}@{}",
                base_asset,
                quote_asset,
                maturity.format(INSTRUMENT_TIMESTAMP_FORMAT).expect("Failed to format time"),
                strike,
                option_type,
                venue
            ),
            venue,
            venue_symbol,
            instrument_type: InstrumentType::Option,
            base_asset,
            quote_asset,
            maturity: Some(maturity),
            strike: Some(strike),
            option_type: Some(option_type),
            contract_size: Decimal::ZERO,
            price_precision,
            quantity_precision,
            base_precision,
            quote_precision,
            tick_size,
            lot_size,
            status: InstrumentStatus::Trading,
        }
    }
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash)]
pub enum InstrumentType {
    Spot,
    Perpetual,
    Future,
    Option,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, Hash)]
pub enum InstrumentStatus {
    Trading,
    Halted,
}

impl fmt::Display for Instrument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}
