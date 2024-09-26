use std::{env, sync::Once};

use rstest::fixture;
use rust_decimal::prelude::*;
use uuid::Uuid;

use crate::{
    logging::init_test_tracing,
    models::{InstrumentStatus, InstrumentType, Venue},
    Instrument,
};

static INIT: Once = Once::new();

/// Setup function that is only run once, even if called multiple times.
pub fn test_setup() {
    INIT.call_once(|| {
        env::set_var("RUN_MODE", "test");
        init_test_tracing();
    });
}

#[fixture]
pub fn binance_venue() -> Venue {
    Venue {
        id: Uuid::parse_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Invalid UUID"),
        name: "Binance".into(),
        venue_type: "exchange".into(),
    }
}

#[fixture]
pub fn binance_btc_usdt_perp(binance_venue: Venue) -> Instrument {
    Instrument {
        id: Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID"),
        venue: binance_venue,
        symbol: "perp-btc-usdt@binance".into(),
        venue_symbol: "BTCUSDT".into(),
        contract_type: InstrumentType::Perpetual,
        base_asset: "btc".into(),
        quote_asset: "usdt".into(),
        maturity: None,
        strike: None,
        option_type: None,
        contract_size: Decimal::from_f64(1.0).expect("Invalid decimal"),
        price_precision: 2,
        quantity_precision: 3,
        base_precision: 8,
        quote_precision: 8,
        tick_size: Decimal::from_f64(0.10).expect("Invalid decimal"),
        lot_size: Decimal::from_f64(0.001).expect("Invalid decimal"),
        status: InstrumentStatus::Trading,
    }
}

#[fixture]
pub fn binance_eth_usdt_perp(binance_venue: Venue) -> Instrument {
    Instrument {
        id: Uuid::from_str("0a6400f4-abb5-4ff3-8720-cf2eeebef26e").expect("Invalid UUID"),
        venue: binance_venue,
        symbol: "perp-eth-usdt@binance".into(),
        venue_symbol: "ETHUSDT".into(),
        contract_type: InstrumentType::Perpetual,
        base_asset: "eth".into(),
        quote_asset: "usdt".into(),
        maturity: None,
        strike: None,
        option_type: None,
        contract_size: Decimal::from_f64(1.0).expect("Invalid decimal"),
        price_precision: 2,
        quantity_precision: 3,
        base_precision: 8,
        quote_precision: 8,
        tick_size: Decimal::from_f64(0.01).expect("Invalid decimal"),
        lot_size: Decimal::from_f64(0.001).expect("Invalid decimal"),
        status: InstrumentStatus::Trading,
    }
}
