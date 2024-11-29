use std::{str::FromStr, sync::Arc};

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    Instance, InstanceBuilder, InstanceStatus, InstanceType, Instrument, InstrumentBuilder, InstrumentStatus,
    InstrumentType, MarketSide, Price, Quantity, Strategy, StrategyBuilder, Tick, TickBuilder, Venue, VenueBuilder,
    VenueOrder, VenueOrderBuilder, VenueOrderStatus, VenueOrderTimeInForce, VenueOrderType,
};

pub fn binance_venue() -> Venue {
    VenueBuilder::default()
        .id(Uuid::parse_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Invalid UUID"))
        .name("Binance")
        .venue_type("exchange")
        .build()
        .expect("Failed to build Venue in test utils")
}

pub fn test_inst_binance_btc_usdt_perp() -> Arc<Instrument> {
    let venue = binance_venue();
    let instrument = InstrumentBuilder::default()
        .id(Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID"))
        .venue(venue)
        .symbol("perp-btc-usdt@binance")
        .venue_symbol("BTCUSDT")
        .instrument_type(InstrumentType::Perpetual)
        .base_asset(Arc::new("btc".to_string()))
        .quote_asset(Arc::new("usdt".to_string()))
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

pub fn test_inst_binance_eth_usdt_perp() -> Arc<Instrument> {
    let venue = binance_venue();
    let instrument = InstrumentBuilder::default()
        .id(Uuid::from_str("0a6400f4-abb5-4ff3-8720-cf2eeebef26e").expect("Invalid UUID"))
        .venue(venue)
        .symbol("perp-eth-usdt@binance")
        .venue_symbol("ETHUSDT")
        .instrument_type(InstrumentType::Perpetual)
        .base_asset(Arc::new("eth".to_string()))
        .quote_asset(Arc::new("usdt".to_string()))
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

pub fn test_tick(
    instrument: Arc<Instrument>,
    bid_price: Price,
    bid_quantity: Quantity,
    ask_price: Price,
    ask_quantity: Quantity,
) -> Tick {
    TickBuilder::default()
        .instrument(instrument)
        .tick_id(0 as u64)
        .bid_price(bid_price)
        .bid_quantity(bid_quantity)
        .ask_price(ask_price)
        .ask_quantity(ask_quantity)
        .build()
        .expect("Failed to build Tick in test utils")
}

pub fn test_instance() -> Arc<Instance> {
    let instance = InstanceBuilder::default()
        .id(Uuid::from_str("31c79d6c-8dce-44a5-a5c8-c02578671afb").expect("Invalid UUID"))
        .name("Test Instance")
        .start_time(OffsetDateTime::now_utc())
        .instance_type(InstanceType::Live)
        .status(InstanceStatus::Running)
        .build()
        .expect("Failed to build Instance in test utils");
    Arc::new(instance)
}

pub fn test_strategy() -> Arc<Strategy> {
    let strategy = StrategyBuilder::default()
        .id(Uuid::from_str("a2d0951e-9bc6-47a4-b803-e4e0bb4e98a3").expect("Invalid UUID"))
        .name("Test Strategy")
        .description(Some("Test Description".into()))
        .build()
        .expect("Failed to build Strategy in test utils");
    Arc::new(strategy)
}

pub fn test_venue_order() -> Arc<VenueOrder> {
    let order = VenueOrderBuilder::default()
        .id(Uuid::from_str("452883de-70fa-4620-8c56-5e00e54dbb0a").expect("Invalid UUID"))
        .instance(test_instance())
        .instrument(test_inst_binance_btc_usdt_perp())
        .order_type(VenueOrderType::Market)
        .time_in_force(VenueOrderTimeInForce::Gtc)
        .side(MarketSide::Buy)
        .price(None)
        .quantity(dec!(1))
        .status(VenueOrderStatus::Placed)
        .build()
        .expect("Failed to build VenueOrder in test utils");
    Arc::new(order)
}
