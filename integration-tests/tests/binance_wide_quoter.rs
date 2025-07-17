use std::{sync::Arc, time::Duration};

use arkin_binance::BinanceExecution;
use arkin_exec_sim::SimulationExecutor;
use arkin_ingestor_binance::SimBinanceIngestor;
use arkin_persistence::{Persistence, PersistenceConfig};
use rust_decimal::prelude::*;
use tracing::info;

use arkin_audit::Audit;
use arkin_core::prelude::*;
use arkin_exec_strat_wide::WideQuoterExecutionStrategy;
use uuid::Uuid;

#[tokio::test]
#[test_log::test]
async fn test_binance_wide_quoting() {
    info!("Starting binance wide quoting test...");
    // Init mock time
    let time = LiveSystemTime::new();

    // Init pubsub
    let pubsub = PubSub::new(time.clone(), true);
    let pubsub_service = Service::new(pubsub.clone(), None);

    // Init audit
    let audit = Audit::new("audit");
    let audit_service = Service::new(audit.to_owned(), Some(pubsub.subscribe(EventFilter::All)));

    // Init wide quoter strategy
    let execution_order_book = ExecutionOrderBook::new(true);
    let venue_order_book = VenueOrderBook::new(true);
    let exec_strat = WideQuoterExecutionStrategy::new(
        "wide-quoter",
        time.clone(),
        pubsub.publisher(),
        execution_order_book,
        venue_order_book,
        dec!(0.001),
        dec!(0.0003),
    );
    let exec_strat_service = Service::new(
        exec_strat,
        Some(pubsub.subscribe(EventFilter::Events(vec![
            EventType::NewWideQuoterExecutionOrder,
            EventType::CancelWideQuoterExecutionOrder,
            EventType::CancelAllWideQuoterExecutionOrders,
            EventType::VenueOrderInflight,
            EventType::VenueOrderPlaced,
            EventType::VenueOrderRejected,
            EventType::VenueOrderFill,
            EventType::VenueOrderCancelled,
            EventType::VenueOrderExpired,
            EventType::TickUpdate,
        ]))),
    );

    // Init binance ingestor

    // Executor
    let execution = BinanceExecution::new(time.clone(), pubsub.publisher());
    let execution_service = Service::new(
        execution,
        Some(pubsub.subscribe(EventFilter::Events(vec![
            EventType::NewVenueOrder,
            EventType::CancelVenueOrder,
            // EventType::CancelAllVenueOrders,
            // EventType::TickUpdate,
        ]))),
    );

    // Create Buy exec order
    let publisher = pubsub.publisher();
    let buy_exec_id = Uuid::new_v4();
    let buy_exec = ExecutionOrder::builder()
        .id(buy_exec_id)
        .strategy(Some(test_strategy_1()))
        .instrument(test_inst_binance_btc_usdt_perp())
        .exec_strategy_type(ExecutionStrategyType::WideQuoter)
        .side(MarketSide::Buy)
        .set_price(dec!(0))
        .set_quantity(dec!(0.001))
        .status(ExecutionOrderStatus::New)
        .created_at(time.now().await)
        .updated_at(time.now().await)
        .build();

    publisher
        .publish(Event::NewWideQuoterExecutionOrder(buy_exec.clone().into()))
        .await;

    // Create Sell exec order
    let sell_exec_id = Uuid::new_v4();
    let sell_exec = ExecutionOrder::builder()
        .id(sell_exec_id)
        .strategy(Some(test_strategy_1()))
        .instrument(test_inst_binance_btc_usdt_perp())
        .exec_strategy_type(ExecutionStrategyType::WideQuoter)
        .side(MarketSide::Sell)
        .set_price(dec!(0))
        .set_quantity(dec!(0.001))
        .status(ExecutionOrderStatus::New)
        .created_at(time.now().await)
        .updated_at(time.now().await)
        .build();

    publisher
        .publish(Event::NewWideQuoterExecutionOrder(sell_exec.clone().into()))
        .await;

    // Setup engine
    let engine = Engine::new();
    engine.register(pubsub_service, 0, 10).await;
    engine.register(audit_service, 0, 10).await;
    engine.register(exec_strat_service, 1, 1).await;
    engine.register(execution_service, 1, 2).await;

    engine.start().await;
    tokio::time::sleep(Duration::from_secs(600)).await;
    engine.stop().await;

    info!(target: "integration-test", " --- LOG REVIEW ---");
    let event_log = audit.event_log();
    info!(target: "integration-test", "received {} events", event_log.len());
}

#[tokio::test]
#[test_log::test]
async fn test_sim_wide_quoting() {
    info!("Starting binance wide quoting test...");
    // Init mock time
    let time = MockTime::new();

    // Start and end time
    let start_time = time.now().await;
    let end_time = start_time + Duration::from_secs(10800);

    // Init pubsub
    let pubsub = PubSub::new(time.clone(), true);
    let pubsub_service = Service::new(pubsub.clone(), None);

    // Init persistence
    let config = load::<PersistenceConfig>();
    let instance = Instance::builder()
        .id(Uuid::from_str("04432ac5-483d-46a3-811b-6be79d6ef7c1").unwrap())
        .name("integration-test".to_owned())
        .instance_type(InstanceType::Test)
        .build();
    let persistence = Persistence::new(&config, instance, false, false, true).await;
    let persistence_service = Service::new(persistence.to_owned(), Some(pubsub.subscribe(EventFilter::All)));

    // Init audit
    let audit = Audit::new("audit");
    let audit_service = Service::new(
        audit.to_owned(),
        Some(pubsub.subscribe(EventFilter::Events(vec![
            EventType::NewWideQuoterExecutionOrder,
            EventType::CancelWideQuoterExecutionOrder,
            EventType::CancelAllWideQuoterExecutionOrders,
            EventType::VenueOrderInflight,
            EventType::VenueOrderPlaced,
            EventType::VenueOrderRejected,
            EventType::VenueOrderFill,
            EventType::VenueOrderCancelled,
            EventType::VenueOrderExpired,
            EventType::NewVenueOrder,
            EventType::CancelVenueOrder,
            EventType::CancelAllVenueOrders,
        ]))),
    );

    // Init sim ingestor
    let binance_ingestor = Arc::new(
        SimBinanceIngestor::builder()
            .identifier("sim-binance-ingestor".to_owned())
            ._time(time.to_owned())
            .start(start_time)
            .end(end_time + Duration::from_secs(3600))
            .instruments(vec![test_inst_binance_btc_usdt_perp()])
            .persistence(persistence.to_owned())
            .publisher(pubsub.publisher())
            .build(),
    );
    let binance_ingestor_service = Service::new(binance_ingestor, None);

    // Init wide quoter strategy
    let execution_order_book = ExecutionOrderBook::new(true);
    let venue_order_book = VenueOrderBook::new(true);
    let exec_strat = WideQuoterExecutionStrategy::new(
        "wide-quoter",
        time.clone(),
        pubsub.publisher(),
        execution_order_book,
        venue_order_book,
        dec!(0.01),
        dec!(0.0001),
    );
    let exec_strat_service = Service::new(
        exec_strat,
        Some(pubsub.subscribe(EventFilter::Events(vec![
            EventType::NewWideQuoterExecutionOrder,
            EventType::CancelWideQuoterExecutionOrder,
            EventType::CancelAllWideQuoterExecutionOrders,
            EventType::VenueOrderInflight,
            EventType::VenueOrderPlaced,
            EventType::VenueOrderRejected,
            EventType::VenueOrderFill,
            EventType::VenueOrderCancelled,
            EventType::VenueOrderExpired,
            EventType::TickUpdate,
        ]))),
    );

    // Init binance ingestor

    // Executor
    let execution = SimulationExecutor::new("exec-sim", time.clone(), pubsub.publisher());
    let execution_service = Service::new(
        execution,
        Some(pubsub.subscribe(EventFilter::Events(vec![
            EventType::NewVenueOrder,
            EventType::CancelVenueOrder,
            EventType::CancelAllVenueOrders,
            EventType::TickUpdate,
        ]))),
    );

    // Create Buy exec order
    let publisher = pubsub.publisher();
    let buy_exec_id = Uuid::new_v4();
    let buy_exec = ExecutionOrder::builder()
        .id(buy_exec_id)
        .strategy(Some(test_strategy_1()))
        .instrument(test_inst_binance_btc_usdt_perp())
        .exec_strategy_type(ExecutionStrategyType::WideQuoter)
        .side(MarketSide::Buy)
        .set_price(dec!(0))
        .set_quantity(dec!(0.001))
        .status(ExecutionOrderStatus::New)
        .created_at(time.now().await)
        .updated_at(time.now().await)
        .build();

    publisher
        .publish(Event::NewWideQuoterExecutionOrder(buy_exec.clone().into()))
        .await;

    // Create Sell exec order
    let sell_exec_id = Uuid::new_v4();
    let sell_exec = ExecutionOrder::builder()
        .id(sell_exec_id)
        .strategy(Some(test_strategy_1()))
        .instrument(test_inst_binance_btc_usdt_perp())
        .exec_strategy_type(ExecutionStrategyType::WideQuoter)
        .side(MarketSide::Sell)
        .set_price(dec!(0))
        .set_quantity(dec!(0.001))
        .status(ExecutionOrderStatus::New)
        .created_at(time.now().await)
        .updated_at(time.now().await)
        .build();

    publisher
        .publish(Event::NewWideQuoterExecutionOrder(sell_exec.clone().into()))
        .await;

    // Setup engine
    let engine = Engine::new();
    engine.register(persistence_service, 0, 10).await;
    engine.register(pubsub_service, 0, 10).await;
    engine.register(audit_service, 0, 10).await;
    engine.register(exec_strat_service, 1, 1).await;
    engine.register(execution_service, 1, 2).await;
    engine.register(binance_ingestor_service, 0, 10).await;

    engine.start().await;

    while time.now().await < end_time {
        info!(target: "integration-test", "Current time: {} end time: {} diff: {}", time.now().await, end_time, end_time - time.now().await);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    engine.stop().await;

    info!(target: "integration-test", " --- LOG REVIEW ---");
    let event_log = audit.event_log();
    info!(target: "integration-test", "received {} events", event_log.len());
}
