use std::{str::FromStr, sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::Stream;
use rust_decimal::prelude::*;
use time::{macros::datetime, OffsetDateTime, UtcDateTime};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{
    utils::Frequency, Account, AccountListQuery, AccountOwner, AccountQuery, AccountType, AggTrade, Asset,
    AssetListQuery, AssetQuery, AssetType, Event, FeatureId, FeatureListQuery, FeatureQuery, InmemoryPubSub, Instance,
    InstanceListQuery, InstanceQuery, InstanceType, Instrument, InstrumentListQuery, InstrumentQuery, InstrumentStatus,
    InstrumentType, MetricType, PersistenceError, PersistenceReader, Pipeline, PipelineListQuery, PipelineQuery, Price,
    Publisher, Quantity, Strategy, StrategyListQuery, StrategyQuery, SystemTime, Tick, Venue, VenueListQuery,
    VenueName, VenueQuery, VenueType,
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
        let start_time = datetime!(2025-01-01 00:00:00).as_utc();
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

    async fn advance_time_to(&self, time: UtcDateTime) {
        // We can only move forward in time
        let current_time = self.state.read().await.current;
        match (current_time, time) {
            (current, new) if current < new => {
                self.state.write().await.current = new;
                debug!(target: "time", "advanced time to {}", new);
            }
            (current, new) if current == new => {
                // No-op
            }
            (current, new) => {
                warn!(target: "time", "attempted to move time backwards from {} to {}", current, new);
            }
        }
    }

    async fn advance_time_by(&self, duration: Duration) {
        self.state.write().await.current += duration;
        debug!(target: "time", "advanced time by {:?}", duration);
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
    async fn refresh(&self) -> Result<(), PersistenceError> {
        Ok(())
    }

    async fn list_instances(&self, _query: &InstanceListQuery) -> Result<Vec<Arc<Instance>>, PersistenceError> {
        Ok(vec![test_instance()])
    }

    async fn list_pipelines(&self, _query: &PipelineListQuery) -> Result<Vec<Arc<Pipeline>>, PersistenceError> {
        Ok(vec![test_pipeline()])
    }

    async fn list_venues(&self, _query: &VenueListQuery) -> Result<Vec<Arc<Venue>>, PersistenceError> {
        Ok(vec![test_binance_venue()])
    }

    async fn list_instruments(&self, _query: &InstrumentListQuery) -> Result<Vec<Arc<Instrument>>, PersistenceError> {
        Ok(vec![test_inst_binance_btc_usdt_perp()])
    }

    async fn list_assets(&self, _query: &AssetListQuery) -> Result<Vec<Arc<Asset>>, PersistenceError> {
        Ok(vec![test_usdt_asset()])
    }

    async fn get_feature(&self, query: &FeatureQuery) -> FeatureId {
        FeatureId::new(query.id.clone())
    }

    async fn list_features(&self, _query: &FeatureListQuery) -> Result<Vec<FeatureId>, PersistenceError> {
        Ok(vec![])
    }

    async fn get_asset(&self, query: &AssetQuery) -> Result<Arc<Asset>, PersistenceError> {
        // Convert single query to list query
        let list_query = AssetListQuery {
            symbols: query.symbol.as_ref().map(|s: &String| vec![s.clone()]).unwrap_or_default(),
            asset_types: query.asset_type.as_ref().map(|t| vec![t.clone()]).unwrap_or_default(),
        };
        let results = self.list_assets(&list_query).await?;
        results.into_iter().next().ok_or(PersistenceError::NotFound)
    }

    async fn get_instance(&self, query: &InstanceQuery) -> Result<Arc<Instance>, PersistenceError> {
        let list_query = InstanceListQuery {
            names: query.name.as_ref().map(|n: &String| vec![n.clone()]).unwrap_or_default(),
            instance_types: query.instance_type.map(|t| vec![t]).unwrap_or_default(),
        };
        let results = self.list_instances(&list_query).await?;
        results.into_iter().next().ok_or(PersistenceError::NotFound)
    }

    async fn get_pipeline(&self, query: &PipelineQuery) -> Result<Arc<Pipeline>, PersistenceError> {
        let list_query = PipelineListQuery {
            names: query.name.as_ref().map(|n: &String| vec![n.clone()]).unwrap_or_default(),
        };
        let results = self.list_pipelines(&list_query).await?;
        results.into_iter().next().ok_or(PersistenceError::NotFound)
    }

    async fn get_venue(&self, query: &VenueQuery) -> Result<Arc<Venue>, PersistenceError> {
        let list_query = VenueListQuery {
            names: query.name.map(|n| vec![n]).unwrap_or_default(),
            venue_types: query.venue_type.as_ref().map(|t| vec![t.clone()]).unwrap_or_default(),
        };
        let results = self.list_venues(&list_query).await?;
        results.into_iter().next().ok_or(PersistenceError::NotFound)
    }

    async fn list_trades(
        &self,
        _instruments: &[Arc<Instrument>],
        _start: UtcDateTime,
        _end: UtcDateTime,
    ) -> Result<Vec<Arc<AggTrade>>, PersistenceError> {
        todo!()
    }

    async fn get_last_tick(&self, _instrument: &Arc<Instrument>) -> Result<Option<Arc<Tick>>, PersistenceError> {
        Ok(Some(test_tick(
            test_inst_binance_btc_usdt_perp(),
            dec!(30000.00),
            dec!(0.5),
            dec!(30010.00),
            dec!(0.3),
        )))
    }

    async fn agg_trade_stream_range_buffered(
        &self,
        _instruments: &[Arc<Instrument>],
        _start: UtcDateTime,
        _end: UtcDateTime,
        _buffer_size: usize,
        _frequency: Frequency,
    ) -> Result<Box<dyn Stream<Item = Event> + Send + Unpin>, PersistenceError> {
        todo!()
    }

    async fn tick_stream_range_buffered(
        &self,
        _instruments: &[Arc<Instrument>],
        _start: UtcDateTime,
        _end: UtcDateTime,
        _buffer_size: usize,
        _frequency: Frequency,
    ) -> Result<Box<dyn Stream<Item = Event> + Send + Unpin>, PersistenceError> {
        todo!()
    }

    async fn metric_stream_range_buffered(
        &self,
        _instruments: &[Arc<Instrument>],
        _metric_type: MetricType,
        _start: UtcDateTime,
        _end: UtcDateTime,
        _buffer_size: usize,
        _frequency: Frequency,
    ) -> Result<Box<dyn Stream<Item = Event> + Send + Unpin>, PersistenceError> {
        todo!()
    }

    async fn get_instrument(&self, query: &InstrumentQuery) -> Result<Arc<Instrument>, PersistenceError> {
        // Convert to list query for simplicity in mock
        let venues = query.venue.map(|v| vec![v]).unwrap_or_default();
        let base_asset_symbols = query
            .base_asset_symbol
            .as_ref()
            .map(|s: &String| vec![s.clone()])
            .unwrap_or_default();
        let quote_asset_symbols = query
            .quote_asset_symbol
            .as_ref()
            .map(|s: &String| vec![s.clone()])
            .unwrap_or_default();
        let instrument_types = query.instrument_type.map(|t| vec![t]).unwrap_or_default();

        let list_query = InstrumentListQuery {
            ids: Some(query.id.map(|id| vec![id]).unwrap_or_default()),
            venues,
            base_assets: query.base_asset.as_ref().map(|a| vec![Arc::clone(a)]),
            base_asset_symbols,
            quote_assets: query.quote_asset.as_ref().map(|a| vec![Arc::clone(a)]),
            quote_asset_symbols,
            margin_assets: query.margin_asset.as_ref().map(|a| vec![Arc::clone(a)]),
            margin_asset_symbols: query
                .margin_asset_symbol
                .as_ref()
                .map(|s: &String| vec![s.clone()])
                .unwrap_or_default(),
            instrument_types,
            synthetic: query.synthetic,
            status: query.status,
            venue_symbol: query.venue_symbol.clone(),
        };

        let results = self.list_instruments(&list_query).await?;
        results.into_iter().next().ok_or(PersistenceError::NotFound)
    }

    async fn list_accounts(&self, _query: &AccountListQuery) -> Result<Vec<Arc<Account>>, PersistenceError> {
        Ok(vec![test_account()])
    }

    async fn list_strategies(&self, _query: &StrategyListQuery) -> Result<Vec<Arc<Strategy>>, PersistenceError> {
        Ok(vec![test_strategy_1(), test_strategy_2()])
    }

    async fn get_account(&self, query: &AccountQuery) -> Result<Arc<Account>, PersistenceError> {
        let list_query = AccountListQuery {
            ids: query.id.map(|id| vec![id]).unwrap_or_default(),
        };
        let results = self.list_accounts(&list_query).await?;
        results.into_iter().next().ok_or(PersistenceError::NotFound)
    }

    async fn get_strategy(&self, query: &StrategyQuery) -> Result<Arc<Strategy>, PersistenceError> {
        let list_query = StrategyListQuery {
            ids: query.id.map(|id| vec![id]).unwrap_or_default(),
            names: query.name.as_ref().map(|n: &String| vec![n.clone()]).unwrap_or_default(),
        };
        let results = self.list_strategies(&list_query).await?;
        results.into_iter().next().ok_or(PersistenceError::NotFound)
    }
}

pub fn test_pubsub() -> Arc<InmemoryPubSub> {
    InmemoryPubSub::new(true)
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
        .name(VenueName::Binance)
        .venue_type(VenueType::Cex)
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(venue)
}

pub fn test_personal_venue() -> Arc<Venue> {
    let venue = Venue::builder()
        .id(Uuid::parse_str("b8b9dcf2-77ea-4d24-964e-8243bb7298ea").expect("Invalid UUID"))
        .name(VenueName::Personal)
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
        .synthetic(false)
        .base_asset(test_btc_asset())
        .quote_asset(test_usdt_asset())
        .margin_asset(test_usdt_asset())
        .maturity(None)
        .strike(None)
        .option_type(None)
        .contract_size(dec!(1.0))
        .price_precision(2_u32)
        .quantity_precision(3_u32)
        .base_precision(8_u32)
        .quote_precision(8_u32)
        .tick_size(dec!(0.10))
        .lot_size(dec!(0.001))
        .status(InstrumentStatus::Trading)
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(instrument)
}

pub fn test_inst_binance_btc_usdt_spot() -> Arc<Instrument> {
    let instrument = Instrument::builder()
        .id(Uuid::from_str("55dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID"))
        .venue(test_binance_venue())
        .symbol("spot-btc-usdt@binance".into())
        .venue_symbol("BTCUSDT".into())
        .instrument_type(InstrumentType::Spot)
        .synthetic(false)
        .base_asset(test_btc_asset())
        .quote_asset(test_usdt_asset())
        .margin_asset(test_usdt_asset())
        .maturity(None)
        .strike(None)
        .option_type(None)
        .contract_size(dec!(1.0))
        .price_precision(2_u32)
        .quantity_precision(3_u32)
        .base_precision(8_u32)
        .quote_precision(8_u32)
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
        .synthetic(false)
        .base_asset(test_eth_asset())
        .quote_asset(test_usdt_asset())
        .margin_asset(test_usdt_asset())
        .maturity(None)
        .strike(None)
        .option_type(None)
        .contract_size(dec!(1.0))
        .price_precision(2_u32)
        .quantity_precision(3_u32)
        .base_precision(8_u32)
        .quote_precision(8_u32)
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
        .tick_id(0_u64)
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

pub fn test_account() -> Arc<Account> {
    let account = Account::builder()
        .id(Uuid::from_str("41c79d6c-8dce-44a5-a5c8-c02578671afb").expect("Invalid UUID"))
        .venue(test_binance_venue())
        .owner(AccountOwner::User)
        .account_type(AccountType::Spot)
        .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
        .build();
    Arc::new(account)
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
