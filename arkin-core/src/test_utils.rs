use std::{str::FromStr, sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::Stream;
use rust_decimal::prelude::*;
use time::{macros::datetime, OffsetDateTime, UtcDateTime};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{
    utils::Frequency, AggTrade, Asset, AssetType, Event, Instance, InstanceType, Instrument, InstrumentStatus,
    InstrumentType, PersistenceReader, Pipeline, Price, PubSub, Publisher, Quantity, Strategy, SystemTime, Tick, Venue,
    VenueType,
};

// Define this in a test module or separate utils file for reuse
#[derive(Clone, Copy, Debug)]
struct MockTimeState {
    current: UtcDateTime,
    next_tick: UtcDateTime,
}

pub struct MockTime {
    state: RwLock<MockTimeState>,
    tick_frequency: Duration,
}

impl MockTime {
    pub fn new() -> Arc<Self> {
        let start_time = datetime!(2025-01-01 00:00:00 UTC).to_utc();
        let tick_frequency = Duration::from_secs(60);
        let next_tick = start_time + tick_frequency;
        Arc::new(Self {
            state: RwLock::new(MockTimeState {
                current: start_time,
                next_tick,
            }),
            tick_frequency,
        })
    }

    pub fn new_from(start_time: UtcDateTime, tick_freq: u64) -> Arc<Self> {
        let tick_frequency = Duration::from_secs(tick_freq);
        let next_tick = start_time + tick_frequency;
        Arc::new(Self {
            state: RwLock::new(MockTimeState {
                current: start_time,
                next_tick,
            }),
            tick_frequency,
        })
    }
}

#[async_trait]
impl SystemTime for MockTime {
    async fn now(&self) -> UtcDateTime {
        self.state.read().await.current
    }

    #[instrument(parent = None, skip_all, fields(service = "mock-time"))]
    async fn advance_time_to(&self, time: UtcDateTime) {
        let mut guard = self.state.write().await;
        guard.current = time;
        debug!(target: "mock-time", "advanced time to {}", time);
    }

    #[instrument(parent = None, skip_all, fields(service = "mock-time"))]
    async fn advance_time_by(&self, duration: Duration) {
        let mut guard = self.state.write().await;
        guard.current += duration;
        debug!(target: "mock-time", "advanced time by {:?}", duration);
    }

    async fn is_final_hour(&self) -> bool {
        false
    }

    async fn is_finished(&self) -> bool {
        false
    }

    async fn is_live(&self) -> bool {
        false
    }

    async fn check_interval(&self) -> Vec<UtcDateTime> {
        let mut guard = self.state.write().await;
        let mut ticks = Vec::new();
        while guard.current >= guard.next_tick {
            ticks.push(guard.next_tick);
            guard.next_tick += self.tick_frequency;
        }
        ticks
    }
}

pub struct MockPublisher {
    events: Arc<Mutex<Vec<Event>>>,
}

impl MockPublisher {
    pub fn new() -> Arc<Self> {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
        .into()
    }

    pub async fn get_events(&self) -> Vec<Event> {
        self.events.lock().await.clone()
    }
}

#[async_trait]
impl Publisher for MockPublisher {
    async fn publish(&self, event: Event) {
        self.events.lock().await.push(event);
    }
}

#[derive(Default)]
pub struct MockPersistence {
    // This struct can be expanded to include mock implementations of persistence methods
}

impl MockPersistence {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
}

#[async_trait]
impl PersistenceReader for MockPersistence {
    async fn get_instance_by_id(&self, _id: &Uuid) -> Arc<Instance> {
        test_instance()
    }

    async fn get_instance_by_name(&self, _name: &str) -> Arc<Instance> {
        test_instance()
    }
    async fn get_pipeline_by_id(&self, _id: &Uuid) -> Arc<Pipeline> {
        test_pipeline()
    }
    async fn get_pipeline_by_name(&self, _name: &str) -> Arc<Pipeline> {
        test_pipeline()
    }
    async fn get_venue_by_id(&self, _id: &Uuid) -> Arc<Venue> {
        test_binance_venue()
    }
    async fn get_venue_by_name(&self, _name: &str) -> Arc<Venue> {
        test_binance_venue()
    }
    async fn get_instrument_by_id(&self, _id: &Uuid) -> Arc<Instrument> {
        test_inst_binance_btc_usdt_perp()
    }
    async fn get_instrument_by_venue_symbol(&self, _symbol: &str) -> Arc<Instrument> {
        test_inst_binance_btc_usdt_perp()
    }
    async fn get_asset_by_id(&self, _id: &Uuid) -> Arc<Asset> {
        test_usdt_asset()
    }
    async fn get_asset_by_symbol(&self, _symbol: &str) -> Arc<Asset> {
        test_usdt_asset()
    }
    async fn tick_stream_range_buffered(
        &self,
        _instruments: &[Arc<Instrument>],
        _start: UtcDateTime,
        _end: UtcDateTime,
        _buffer_size: usize,
        _frequency: Frequency,
    ) -> Box<dyn Stream<Item = Arc<Tick>> + Send + Unpin> {
        todo!()
    }
    async fn trade_stream_range_buffered(
        &self,
        _instruments: &[Arc<Instrument>],
        _start: UtcDateTime,
        _end: UtcDateTime,
        _buffer_size: usize,
        _frequency: Frequency,
    ) -> Box<dyn Stream<Item = Arc<AggTrade>> + Send + Unpin> {
        todo!()
    }
}

pub fn test_pubsub() -> Arc<PubSub> {
    let pubsub = PubSub::new(true);
    pubsub
}

pub fn test_btc_asset() -> Arc<Asset> {
    let asset = Asset::builder()
        .id(Uuid::parse_str("894ff9df-e76e-4b2e-aaec-49988de26a84").expect("Invalid UUID"))
        .symbol("BTC".into())
        .name("Bitcoin".into())
        .asset_type(AssetType::Crypto)
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(asset)
}

pub fn test_eth_asset() -> Arc<Asset> {
    let asset = Asset::builder()
        .id(Uuid::parse_str("3091ac12-64a7-4824-9ea5-e1c27e10af6f").expect("Invalid UUID"))
        .symbol("ETH".into())
        .name("Ethereum".into())
        .asset_type(AssetType::Crypto)
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(asset)
}

pub fn test_usdt_asset() -> Arc<Asset> {
    let asset = Asset::builder()
        .id(Uuid::parse_str("5ba12a78-1f89-41b6-87c5-020afb7f680d").expect("Invalid UUID"))
        .symbol("USDT".into())
        .name("Tether".into())
        .asset_type(AssetType::Crypto)
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(asset)
}

pub fn test_bnb_asset() -> Arc<Asset> {
    let asset = Asset::builder()
        .id(Uuid::parse_str("91e61c74-9e4c-4226-b848-8b96e1ec4941").expect("Invalid UUID"))
        .symbol("BNB".into())
        .name("Binance Coin".into())
        .asset_type(AssetType::Crypto)
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(asset)
}

pub fn test_binance_venue() -> Arc<Venue> {
    let venue = Venue::builder()
        .id(Uuid::parse_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Invalid UUID"))
        .name("Binance".into())
        .venue_type(VenueType::Cex)
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(venue)
}

pub fn test_personal_venue() -> Arc<Venue> {
    let venue = Venue::builder()
        .id(Uuid::parse_str("b8b9dcf2-77ea-4d24-964e-8243bb7298ea").expect("Invalid UUID"))
        .name("Personal".into())
        .venue_type(VenueType::Otc)
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(venue)
}

pub fn test_inst_binance_btc_usdt_perp() -> Arc<Instrument> {
    let instrument = Instrument::builder()
        .id(Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID"))
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
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(instrument)
}

pub fn test_inst_binance_eth_usdt_perp() -> Arc<Instrument> {
    let instrument = Instrument::builder()
        .id(Uuid::from_str("0a6400f4-abb5-4ff3-8720-cf2eeebef26e").expect("Invalid UUID"))
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
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(instrument)
}

pub fn test_pipeline() -> Arc<Pipeline> {
    let pipeline = Pipeline::builder()
        .id(Uuid::from_str("df5305b0-3e9b-4b7c-8a13-1406e93f5cc9").expect("Invalid UUID"))
        .name("Test Pipeline".into())
        .description("This Pipeline is for testing purposes".into())
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
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
        .event_time(OffsetDateTime::now_utc().to_utc())
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
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(instance)
}

pub fn test_strategy_1() -> Arc<Strategy> {
    let strategy = Strategy::builder()
        .id(Uuid::from_str("1fce35ce-1583-4334-a410-bc0f71c7469b").expect("Invalid UUID"))
        .name("test_strategy_2".into())
        .description(Some("This strategy is only for testing".into()))
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(strategy)
}

pub fn test_strategy_2() -> Arc<Strategy> {
    let strategy = Strategy::builder()
        .id(Uuid::from_str("a2d0951e-9bc6-47a4-b803-e4e0bb4e98a3").expect("Invalid UUID"))
        .name("test_strategy_2".into())
        .description(Some("This strategy is only for testing".into()))
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(strategy)
}
