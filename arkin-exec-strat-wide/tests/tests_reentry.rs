
use arkin_core::prelude::*;
use arkin_core::test_utils::{MockPersistence, MockPublisher, MockTime};
use arkin_exec_strat_wide::WideQuoterExecutionStrategy;
use async_trait::async_trait;
use rust_decimal::dec;
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

struct MockSubscriber;

#[async_trait]
impl Subscriber for MockSubscriber {
    async fn recv(&self) -> Option<Event> {
        None
    }
    fn needs_ack(&self) -> bool {
        false
    }
    async fn send_ack(&self) {}
}

struct MockPubSub {
    publisher: Arc<MockPublisher>,
}

impl MockPubSub {
    fn new(publisher: Arc<MockPublisher>) -> Arc<Self> {
        Arc::new(Self { publisher })
    }
}

impl PubSubTrait for MockPubSub {
    fn publisher(&self) -> Arc<dyn Publisher> {
        self.publisher.clone()
    }
    fn subscribe(&self, _filter: EventFilter) -> Arc<dyn Subscriber> {
        Arc::new(MockSubscriber)
    }
}

fn test_inst_binance_btc_usdt_perp() -> Arc<Instrument> {
    Arc::new(
        Instrument::builder()
            .id(Uuid::new_v4())
            .symbol("BTC-USDT-PERP".to_string())
            .venue_symbol("BTCUSDT".to_string())
            .instrument_type(InstrumentType::Perpetual)
            .venue(Arc::new(
                Venue::builder()
                    .id(Uuid::new_v4())
                    .name(VenueName::Binance)
                    .venue_type(VenueType::Cex)
                    .created(OffsetDateTime::now_utc().into())
                    .updated(OffsetDateTime::now_utc().into())
                    .build(),
            ))
            .base_asset(Arc::new(
                Asset::builder()
                    .id(Uuid::new_v4())
                    .symbol("BTC".to_string())
                    .name("Bitcoin".to_string())
                    .asset_type(AssetType::Crypto)
                    .created(OffsetDateTime::now_utc().into())
                    .updated(OffsetDateTime::now_utc().into())
                    .build(),
            ))
            .quote_asset(Arc::new(
                Asset::builder()
                    .id(Uuid::new_v4())
                    .symbol("USDT".to_string())
                    .name("Tether".to_string())
                    .asset_type(AssetType::Crypto)
                    .created(OffsetDateTime::now_utc().into())
                    .updated(OffsetDateTime::now_utc().into())
                    .build(),
            ))
            .margin_asset(Arc::new(
                Asset::builder()
                    .id(Uuid::new_v4())
                    .symbol("USDT".to_string())
                    .name("Tether".to_string())
                    .asset_type(AssetType::Crypto)
                    .created(OffsetDateTime::now_utc().into())
                    .updated(OffsetDateTime::now_utc().into())
                    .build(),
            ))
            .synthetic(false)
            .contract_size(dec!(1))
            .price_precision(2)
            .quantity_precision(3)
            .base_precision(8)
            .quote_precision(8)
            .tick_size(dec!(0.01))
            .lot_size(dec!(0.001))
            .status(InstrumentStatus::Trading)
            .maturity(None)
            .strike(None)
            .option_type(None)
            .created(OffsetDateTime::now_utc().into())
            .updated(OffsetDateTime::now_utc().into())
            .build(),
    )
}

fn test_strategy_1() -> Arc<Strategy> {
    Arc::new(
        Strategy::builder()
            .id(Uuid::new_v4())
            .name("test_strategy_1".to_string())
            .description(Some("Test Strategy".to_string()))
            .created(OffsetDateTime::now_utc().into())
            .updated(OffsetDateTime::now_utc().into())
            .build(),
    )
}

#[tokio::test]
#[test_log::test]
async fn test_wide_quoter_reentry() {
    // Setup
    let time = MockTime::new();
    let publisher = MockPublisher::new();
    let pubsub = MockPubSub::new(publisher.clone());
    let persistence = MockPersistence::new();
    let core_ctx = Arc::new(CoreCtx::new(time.clone(), pubsub, persistence));

    let execution_order_book = ExecutionOrderBook::new(publisher.clone(), false);
    let venue_order_book = VenueOrderBook::new(publisher.clone(), false);
    let exec_strategy = WideQuoterExecutionStrategy::new(
        execution_order_book.clone(),
        venue_order_book.clone(),
        dec!(0.01),
        dec!(0.002),
    );

    let instrument = test_inst_binance_btc_usdt_perp();

    // 1. Create first exec order
    let exec_order_id_1 = Uuid::new_v4();
    let exec_order_1 = ExecutionOrder::builder()
        .id(exec_order_id_1)
        .strategy(Some(test_strategy_1()))
        .instrument(instrument.clone())
        .exec_strategy_type(ExecutionStrategyType::WideQuoter)
        .side(MarketSide::Buy)
        .set_price(dec!(0))
        .set_quantity(dec!(1))
        .status(ExecutionOrderStatus::New)
        .created(time.now().await)
        .updated(time.now().await)
        .build();

    exec_strategy
        .handle_event(core_ctx.clone(), Event::NewExecutionOrder(exec_order_1.clone().into()))
        .await;

    // 2. Tick update -> places venue order
    let tick1 = Tick::builder()
        .event_time(time.now().await)
        .instrument(instrument.clone())
        .tick_id(0u64)
        .ask_price(dec!(50000.0))
        .ask_quantity(dec!(0.3))
        .bid_price(dec!(49000.0))
        .bid_quantity(dec!(0.7))
        .build();
    exec_strategy
        .handle_event(core_ctx.clone(), Event::TickUpdate(tick1.clone().into()))
        .await;

    let venue_orders_1 = venue_order_book.list_orders_by_exec_id(exec_order_id_1);
    assert_eq!(venue_orders_1.len(), 1);
    let venue_order_1 = venue_orders_1[0].clone();
    assert_eq!(venue_order_1.price, dec!(49005)); // 49500 * 0.99

    // 3. Cancel first execution order
    exec_strategy
        .handle_event(core_ctx.clone(), Event::CancelExecutionOrder(exec_order_1.clone().into()))
        .await;

    // Simulate venue order cancellation
    let venue_order_update = VenueOrderUpdate::builder()
        .id(venue_order_1.id)
        .event_time(time.now().await)
        .status(VenueOrderStatus::Cancelled)
        .build();
    exec_strategy
        .handle_event(core_ctx.clone(), Event::VenueOrderUpdate(venue_order_update.into()))
        .await;

    assert_eq!(
        execution_order_book.get(exec_order_id_1).unwrap().status,
        ExecutionOrderStatus::Cancelled
    );

    // 4. Create second exec order (same instrument)
    let exec_order_id_2 = Uuid::new_v4();
    let exec_order_2 = ExecutionOrder::builder()
        .id(exec_order_id_2)
        .strategy(Some(test_strategy_1()))
        .instrument(instrument.clone())
        .exec_strategy_type(ExecutionStrategyType::WideQuoter)
        .side(MarketSide::Buy)
        .set_price(dec!(0))
        .set_quantity(dec!(1))
        .status(ExecutionOrderStatus::New)
        .created(time.now().await)
        .updated(time.now().await)
        .build();

    exec_strategy
        .handle_event(core_ctx.clone(), Event::NewExecutionOrder(exec_order_2.clone().into()))
        .await;

    // 5. Tick update (same price as before) -> SHOULD place new venue order
    // If we were still keying by Instrument, this would fail to quote because the price hasn't changed
    // relative to the last quote for this instrument.
    exec_strategy
        .handle_event(core_ctx.clone(), Event::TickUpdate(tick1.clone().into()))
        .await;

    let venue_orders_2 = venue_order_book.list_orders_by_exec_id(exec_order_id_2);
    assert_eq!(
        venue_orders_2.len(),
        1,
        "Should have placed a new venue order for the new execution order"
    );
    assert_eq!(venue_orders_2[0].price, dec!(49005));
}
