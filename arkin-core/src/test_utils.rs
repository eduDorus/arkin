use std::{str::FromStr, sync::Arc};

use rust_decimal::prelude::*;
use uuid::Uuid;

use crate::{Instrument, InstrumentBuilder, InstrumentStatus, InstrumentType, Venue, VenueBuilder};

pub fn binance_venue() -> Venue {
    VenueBuilder::default()
        .id(Uuid::parse_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Invalid UUID"))
        .name("Binance")
        .venue_type("exchange")
        .build()
        .expect("Failed to build Venue in test utils")
}

pub fn binance_btc_usdt_perp() -> Arc<Instrument> {
    let venue = binance_venue();
    let instrument = InstrumentBuilder::default()
        .id(Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID"))
        .venue(venue)
        .symbol("perp-btc-usdt@binance")
        .venue_symbol("BTCUSDT")
        .instrument_type(InstrumentType::Perpetual)
        .base_asset("btc")
        .quote_asset("usdt")
        .maturity(None)
        .strike(None)
        .option_type(None)
        .contract_size(Decimal::from_f64(1.0).expect("Invalid decimal"))
        .price_precision(2 as u32)
        .quantity_precision(3 as u32)
        .base_precision(8 as u32)
        .quote_precision(8 as u32)
        .tick_size(Decimal::from_f64(0.10).expect("Invalid decimal"))
        .lot_size(Decimal::from_f64(0.001).expect("Invalid decimal"))
        .status(InstrumentStatus::Trading)
        .build()
        .expect("Failed to build Instrument in test utils");
    Arc::new(instrument)
}

pub fn binance_eth_usdt_perp() -> Arc<Instrument> {
    let venue = binance_venue();
    let instrument = InstrumentBuilder::default()
        .id(Uuid::from_str("0a6400f4-abb5-4ff3-8720-cf2eeebef26e").expect("Invalid UUID"))
        .venue(venue)
        .symbol("perp-eth-usdt@binance")
        .venue_symbol("ETHUSDT")
        .instrument_type(InstrumentType::Perpetual)
        .base_asset("eth")
        .quote_asset("usdt")
        .maturity(None)
        .strike(None)
        .option_type(None)
        .contract_size(Decimal::from_f64(1.0).expect("Invalid decimal"))
        .price_precision(2 as u32)
        .quantity_precision(3 as u32)
        .base_precision(8 as u32)
        .quote_precision(8 as u32)
        .tick_size(Decimal::from_f64(0.01).expect("Invalid decimal"))
        .lot_size(Decimal::from_f64(0.001).expect("Invalid decimal"))
        .status(InstrumentStatus::Trading)
        .build()
        .expect("Failed to build Instrument in test utils");
    Arc::new(instrument)
}
