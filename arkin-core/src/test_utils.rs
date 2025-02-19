use std::{str::FromStr, sync::Arc};

use rust_decimal_macros::dec;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    Asset, AssetType, ExecutionOrder, ExecutionOrderStatus, ExecutionOrderType, Instance, InstanceType, Instrument,
    InstrumentStatus, InstrumentType, MarketSide, Pipeline, Portfolio, Price, Quantity, Strategy, Tick, Venue,
    VenueType,
};

pub fn test_btc_asset() -> Arc<Asset> {
    let asset = Asset::builder()
        .id(Uuid::parse_str("894ff9df-e76e-4b2e-aaec-49988de26a84").expect("Invalid UUID"))
        .symbol("BTC".into())
        .name("Bitcoin".into())
        .asset_type(AssetType::Crypto)
        .build();
    Arc::new(asset)
}

pub fn test_eth_asset() -> Arc<Asset> {
    let asset = Asset::builder()
        .id(Uuid::parse_str("3091ac12-64a7-4824-9ea5-e1c27e10af6f").expect("Invalid UUID"))
        .symbol("ETH".into())
        .name("Ethereum".into())
        .asset_type(AssetType::Crypto)
        .build();
    Arc::new(asset)
}

pub fn test_usdt_asset() -> Arc<Asset> {
    let asset = Asset::builder()
        .id(Uuid::parse_str("5ba12a78-1f89-41b6-87c5-020afb7f680d").expect("Invalid UUID"))
        .symbol("USDT".into())
        .name("Tether".into())
        .asset_type(AssetType::Crypto)
        .build();
    Arc::new(asset)
}

pub fn test_bnb_asset() -> Arc<Asset> {
    let asset = Asset::builder()
        .id(Uuid::parse_str("91e61c74-9e4c-4226-b848-8b96e1ec4941").expect("Invalid UUID"))
        .symbol("BNB".into())
        .name("Binance Coin".into())
        .asset_type(AssetType::Crypto)
        .build();
    Arc::new(asset)
}

pub fn test_binance_venue() -> Arc<Venue> {
    let venue = Venue::builder()
        .id(Uuid::parse_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Invalid UUID"))
        .name("Binance".into())
        .venue_type(VenueType::Cex)
        .build();
    Arc::new(venue)
}

pub fn test_personal_venue() -> Arc<Venue> {
    let venue = Venue::builder()
        .id(Uuid::parse_str("b8b9dcf2-77ea-4d24-964e-8243bb7298ea").expect("Invalid UUID"))
        .name("Personal".into())
        .venue_type(VenueType::Otc)
        .build();
    Arc::new(venue)
}

pub fn test_inst_binance_btc_usdt_perp() -> Arc<Instrument> {
    let instrument = Instrument::builder()
        .id(Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID"))
        .secondary_id(1)
        .venue(test_binance_venue())
        .symbol("perp-btc-usdt@binance".into())
        .venue_symbol("BTCUSDT".into())
        .instrument_type(InstrumentType::Perpetual)
        .base_asset(test_btc_asset())
        .quote_asset(test_usdt_asset())
        .margin_asset(test_usdt_asset())
        .maturity(None)
        .strike(None)
        .option_type(None)
        .contract_size(dec!(1.0))
        .price_precision(2 as u32)
        .quantity_precision(3 as u32)
        .base_precision(8 as u32)
        .quote_precision(8 as u32)
        .tick_size(dec!(0.10))
        .lot_size(dec!(0.001))
        .status(InstrumentStatus::Trading)
        .build();
    Arc::new(instrument)
}

pub fn test_inst_binance_eth_usdt_perp() -> Arc<Instrument> {
    let instrument = Instrument::builder()
        .id(Uuid::from_str("0a6400f4-abb5-4ff3-8720-cf2eeebef26e").expect("Invalid UUID"))
        .secondary_id(2)
        .venue(test_binance_venue())
        .symbol("perp-eth-usdt@binance".into())
        .venue_symbol("ETHUSDT".into())
        .instrument_type(InstrumentType::Perpetual)
        .base_asset(test_eth_asset())
        .quote_asset(test_usdt_asset())
        .margin_asset(test_usdt_asset())
        .maturity(None)
        .strike(None)
        .option_type(None)
        .contract_size(dec!(1.0))
        .price_precision(2 as u32)
        .quantity_precision(3 as u32)
        .base_precision(8 as u32)
        .quote_precision(8 as u32)
        .tick_size(dec!(0.01))
        .lot_size(dec!(0.001))
        .status(InstrumentStatus::Trading)
        .build();
    Arc::new(instrument)
}

pub fn test_pipeline() -> Arc<Pipeline> {
    let pipeline = Pipeline::builder()
        .id(Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID"))
        .name("Test Pipeline".into())
        .description("This Pipeline is for testing purposes".into())
        .created_at(OffsetDateTime::now_utc())
        .updated_at(OffsetDateTime::now_utc())
        .build();
    Arc::new(pipeline)
}

pub fn test_tick(
    instrument: Arc<Instrument>,
    bid_price: Price,
    bid_quantity: Quantity,
    ask_price: Price,
    ask_quantity: Quantity,
) -> Arc<Tick> {
    let tick = Tick::builder()
        .instrument(instrument)
        .tick_id(0 as u64)
        .bid_price(bid_price)
        .bid_quantity(bid_quantity)
        .ask_price(ask_price)
        .ask_quantity(ask_quantity)
        .build();
    Arc::new(tick)
}

pub fn test_instance() -> Arc<Instance> {
    let instance = Instance::builder()
        .id(Uuid::from_str("31c79d6c-8dce-44a5-a5c8-c02578671afb").expect("Invalid UUID"))
        .name("Test Instance".into())
        .instance_type(InstanceType::Live)
        .build();
    Arc::new(instance)
}

pub fn test_portfolio() -> Arc<Portfolio> {
    let portfolio = Portfolio::builder()
        .id(Uuid::from_str("fcb8b709-325c-4cc8-8778-4ca4a7f3616b").expect("Invalid UUID"))
        .name("Test Portfolio".into())
        .description("This Portfolio is for testing purposes".into())
        .created_at(OffsetDateTime::now_utc())
        .updated_at(OffsetDateTime::now_utc())
        .build();
    Arc::new(portfolio)
}

pub fn test_strategy() -> Arc<Strategy> {
    let strategy = Strategy::builder()
        .id(Uuid::from_str("a2d0951e-9bc6-47a4-b803-e4e0bb4e98a3").expect("Invalid UUID"))
        .name("Test Strategy".into())
        .description(Some("Test Description".into()))
        .build();
    Arc::new(strategy)
}

pub fn test_strategy_crossover() -> Arc<Strategy> {
    let strategy = Strategy::builder()
        .id(Uuid::from_str("1fce35ce-1583-4334-a410-bc0f71c7469b").expect("Invalid UUID"))
        .name("Test Crossover Strategy".into())
        .description(Some("Test Description".into()))
        .build();
    Arc::new(strategy)
}

// pub fn test_venue_order() -> Arc<VenueOrder> {
//     let order = VenueOrder::builder()
//         .id(Uuid::from_str("452883de-70fa-4620-8c56-5e00e54dbb0a").expect("Invalid UUID"))
//         .portfolio(test_portfolio())
//         .instrument(test_inst_binance_btc_usdt_perp())
//         .order_type(VenueOrderType::Market)
//         .time_in_force(VenueOrderTimeInForce::Gtc)
//         .side(MarketSide::Buy)
//         .price(dec!(100))
//         .quantity(dec!(1))
//         .status(VenueOrderStatus::Placed)
//         .build();
//     Arc::new(order)
// }

pub fn test_execution_order_new() -> Arc<ExecutionOrder> {
    let order = ExecutionOrder::builder()
        .id(Uuid::from_str("452883de-70fa-4620-8c56-5e00e54dbb0a").expect("Invalid UUID"))
        .strategy(Some(test_strategy()))
        .instrument(test_inst_binance_btc_usdt_perp())
        .order_type(ExecutionOrderType::Maker)
        .side(MarketSide::Buy)
        .price(dec!(0))
        .quantity(dec!(1))
        .status(ExecutionOrderStatus::New)
        .created_at(OffsetDateTime::now_utc())
        .updated_at(OffsetDateTime::now_utc())
        .build();
    Arc::new(order)
}

pub fn test_execution_order_filled() -> Arc<ExecutionOrder> {
    let order = ExecutionOrder::builder()
        .id(Uuid::from_str("452883de-70fa-4620-8c56-5e00e54dbb0a").expect("Invalid UUID"))
        .strategy(Some(test_strategy()))
        .instrument(test_inst_binance_btc_usdt_perp())
        .order_type(ExecutionOrderType::Maker)
        .side(MarketSide::Buy)
        .price(dec!(0).into())
        .quantity(dec!(1))
        .fill_price(dec!(110))
        .filled_quantity(dec!(1))
        .total_commission(dec!(0.2))
        .status(ExecutionOrderStatus::Filled)
        .created_at(OffsetDateTime::now_utc())
        .updated_at(OffsetDateTime::now_utc())
        .build();
    Arc::new(order)
}
