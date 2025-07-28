use std::{str::FromStr, sync::Arc, time::Duration};

use arkin_strat_agent::AgentStrategy;
use time::macros::utc_datetime;
use tracing::info;
use uuid::Uuid;

use arkin_accounting::Accounting;
use arkin_audit::Audit;
use arkin_core::prelude::*;
use arkin_exec_sim::SimulationExecutor;
use arkin_exec_strat_taker::TakerExecutionStrategy;
use arkin_ingestor_binance::SimBinanceIngestor;
use arkin_insights::{prelude::InsightsConfig, Insights};
use arkin_persistence::{Persistence, PersistenceConfig};

#[tokio::test]
#[test_log::test]
async fn test_simulation() {
    info!("Starting simulation test...");
    // Init mock time
    let time = MockTime::new_from(utc_datetime!(2025-03-01 00:00:00));

    // Start and end time
    let start_time = time.now().await;
    let end_time = start_time + Duration::from_secs(14 * 86400);

    // Init pubsub
    let pubsub = PubSub::new(time.clone(), true);
    let pubsub_service = Service::new(pubsub.clone(), None);

    // Init persistence
    let config = load::<PersistenceConfig>();
    let instance = Instance::builder()
        .id(Uuid::from_str("04432ac5-483d-46a3-811b-6be79d6ef7c1").unwrap())
        .name("integration-test".to_owned())
        .instance_type(InstanceType::Test)
        .created(time.now().await)
        .updated(time.now().await)
        .build();
    let persistence = Persistence::new(&config, instance, false, false, false);
    // let persistence_service = Service::new(persistence.to_owned(), None);
    let persistence_service = Service::new(persistence.to_owned(), Some(pubsub.subscribe(EventFilter::Persistable)));

    // Init accounting
    let accounting = Arc::new(
        Accounting::builder()
            .time(time.to_owned())
            .publisher(pubsub.publisher())
            .build(),
    );
    let accounting_service = Service::new(
        accounting.to_owned(),
        Some(pubsub.subscribe(EventFilter::Events(vec![
            EventType::InitialAccountUpdate,
            EventType::ReconcileAccountUpdate,
            EventType::VenueAccountUpdate,
            EventType::VenueTradeUpdate,
        ]))),
    );

    // Init audit
    let audit = Audit::new("audit");
    let audit_service = Service::new(audit.to_owned(), Some(pubsub.subscribe(EventFilter::AllWithoutMarket)));

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

    // Insights service
    let pipeline_config = load::<InsightsConfig>();
    let pipeline_info = Pipeline::builder()
        .id(Uuid::new_v4())
        .name("pipeline-test".to_owned())
        .description("Pipeline used for test purpuse".to_owned())
        .created(time.now().await)
        .updated(time.now().await)
        .build();
    let insights = Insights::new(
        pubsub.publisher(),
        pipeline_info.into(),
        &pipeline_config.insights_service.pipeline,
    )
    .await;
    let insights_service = Service::new(
        insights,
        Some(pubsub.subscribe(EventFilter::Events(vec![
            EventType::AggTradeUpdate,
            EventType::TickUpdate,
            EventType::InsightsTick,
        ]))),
    );

    // Crossover strategy
    // let strategy = Strategy::builder()
    //     .id(Uuid::from_str("9433328f-8f55-4357-a639-85350dec93d2").expect("Invalid UUID"))
    //     .name("crossover".into())
    //     .description(Some("This strategy is only for testing".into()))
    //     .created(time.now().await)
    //     .updated(time.now().await)
    //     .build();
    // let strategy_name = Arc::new(strategy);
    // let crossover_strategy = Arc::new(
    //     CrossoverStrategy::builder()
    //         .identifier("crossover_strategy".into())
    //         .publisher(pubsub.publisher())
    //         .time(time.to_owned())
    //         .strategy(strategy_name)
    //         .allocation_limit_per_instrument(dec!(10000))
    //         .fast_ma(FeatureId::new("vwap_price_ema_10".into()))
    //         .slow_ma(FeatureId::new("vwap_price_ema_60".into()))
    //         .build(),
    // );
    // let strategy_service = Service::new(
    //     crossover_strategy,
    //     Some(pubsub.subscribe(EventFilter::Events(vec![EventType::InsightsUpdate]))),
    // );

    let strategy_name = Strategy::builder()
        .id(Uuid::from_str("bf59f914-3304-4f57-89ea-c098b9af3f59").expect("Invalid UUID"))
        .name("agent".into())
        .description(Some("This strategy is only for testing".into()))
        .created(time.now().await)
        .updated(time.now().await)
        .build()
        .into();
    let strategy = AgentStrategy::new(time.clone(), pubsub.publisher(), strategy_name);
    let strategy_service = Service::new(
        strategy,
        Some(pubsub.subscribe(EventFilter::Events(vec![EventType::InsightsUpdate]))),
    );

    // Exec Strategy
    let execution_order_book = ExecutionOrderBook::new(pubsub.publisher(), true);
    let venue_order_book = VenueOrderBook::new(pubsub.publisher(), true);
    let exec_strategy = Arc::new(
        TakerExecutionStrategy::builder()
            .identifier("exec-strat-taker".to_string())
            .time(time.to_owned())
            .publisher(pubsub.publisher())
            .exec_order_book(execution_order_book.to_owned())
            .venue_order_book(venue_order_book.to_owned())
            .build(),
    );
    let exec_strategy_service = Service::new(
        exec_strategy,
        Some(pubsub.subscribe(EventFilter::Events(vec![
            EventType::NewTakerExecutionOrder,
            EventType::CancelTakerExecutionOrder,
            EventType::CancelAllTakerExecutionOrders,
            EventType::VenueOrderInflight,
            EventType::VenueOrderPlaced,
            EventType::VenueOrderRejected,
            EventType::VenueOrderFill,
            EventType::VenueOrderCancelled,
            EventType::VenueOrderExpired,
        ]))),
    );

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

    // Setup engine
    let engine = Engine::new();
    engine.register(persistence_service, 0, 10).await;
    engine.register(audit_service, 0, 10).await;
    engine.register(accounting_service, 0, 10).await;
    engine.register(binance_ingestor_service, 0, 10).await;
    engine.register(insights_service, 0, 10).await;
    engine.register(strategy_service, 0, 10).await;
    engine.register(exec_strategy_service, 0, 10).await;
    engine.register(execution_service, 0, 10).await;
    engine.register(pubsub_service, 0, 10).await;

    engine.start().await;

    while time.now().await < end_time {
        info!(target: "integration-test", "Current time: {} end time: {} diff: {}", time.now().await, end_time, end_time - time.now().await);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    engine.stop().await;

    info!(target: "integration-test", " --- LOG REVIEW ---");
    let event_log = audit.event_log();
    info!(target: "integration-test", "received {} events", event_log.len());
    // for event in event_log {
    //     info!(target: "integration-test", " - {} Event: {}",event.timestamp(), event.event_type());
    // }
}
