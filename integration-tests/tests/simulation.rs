use std::{str::FromStr, sync::Arc, time::Duration};

use arkin_accounting::Accounting;
use arkin_audit::Audit;
use arkin_core::prelude::*;
use arkin_exec_strat_taker::ExecutionStrategy;
use arkin_execution_sim::Executor;
use arkin_ingestor_binance::SimBinanceIngestor;
use arkin_insights::{prelude::InsightsConfig, Insights};
use arkin_persistence::{Persistence, PersistenceConfig};
use arkin_strat_crossover::CrossoverStrategy;
use rust_decimal_macros::dec;
use tracing::info;
use uuid::Uuid;

#[tokio::test]
#[test_log::test]
async fn test_simulation() {
    info!("Starting simulation test...");
    // Init mock time
    let time = MockTime::new();

    // Start and end time
    let start_time = time.now().await;
    let end_time = start_time + Duration::from_secs(86400);

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

    // Init accounting
    let accounting = Arc::new(
        Accounting::builder()
            .time(time.to_owned())
            .publisher(pubsub.publisher())
            .build(),
    );
    let accounting_service = Service::new(
        accounting.to_owned(),
        Some(pubsub.subscribe(EventFilter::Events(vec![EventType::VenueOrderFill]))),
    );

    // Init audit
    let audit = Audit::new("audit");
    let audit_service = Service::new(audit.to_owned(), Some(pubsub.subscribe(EventFilter::All)));

    // Init sim ingestor
    let binance_ingestor = Arc::new(
        SimBinanceIngestor::builder()
            .identifier("sim-binance-ingestor".to_owned())
            ._time(time.to_owned())
            .start(start_time)
            .end(end_time + Duration::from_secs(1))
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
            EventType::TradeUpdate,
            EventType::TickUpdate,
            EventType::InsightsTick,
        ]))),
    );

    // Crossover strategy
    let strategy = Strategy::builder()
        .id(Uuid::from_str("1fce35ce-1583-4334-a410-bc0f71c7469b").expect("Invalid UUID"))
        .name("crossover_strategy".into())
        .description(Some("This strategy is only for testing".into()))
        .build();
    let strategy_name = Arc::new(strategy);
    let crossover_strategy = Arc::new(
        CrossoverStrategy::builder()
            .identifier("crossover_strategy".into())
            .publisher(pubsub.publisher())
            .time(time.to_owned())
            .strategy(strategy_name)
            .allocation_limit_per_instrument(dec!(10000))
            .fast_ma(FeatureId::new("vwap_price_ema_10".into()))
            .slow_ma(FeatureId::new("vwap_price_ema_60".into()))
            .build(),
    );
    let strategy_service = Service::new(
        crossover_strategy,
        Some(pubsub.subscribe(EventFilter::Events(vec![EventType::InsightsUpdate]))),
    );

    // Exec Strategy
    let execution_order_book = ExecutionOrderBook::new(true);
    let venue_order_book = VenueOrderBook::new(true);
    let exec_strategy = Arc::new(
        ExecutionStrategy::builder()
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
            EventType::NewMakerExecutionOrder,
            EventType::CancelMakerExecutionOrder,
            EventType::CancelAllMakerExecutionOrders,
            EventType::VenueOrderInflight,
            EventType::VenueOrderPlaced,
            EventType::VenueOrderRejected,
            EventType::VenueOrderFill,
            EventType::VenueOrderCancelled,
            EventType::VenueOrderExpired,
        ]))),
    );

    // Executor
    let execution = Executor::new("exec-sim", time.clone(), pubsub.publisher());
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

    // Publish some demo events
    // for _ in 0..5 {
    //     demo_publisher
    //         .publish(Event::Finished(time.now().await + Duration::from_secs(1)))
    //         .await;
    //     tokio::time::sleep(Duration::from_secs(1)).await;
    //     // time.advance_time_by(Duration::from_secs(1)).await;
    // }

    while time.now().await < end_time {
        info!(target: "integration-test", "Current time: {} end time: {} diff: {}", time.now().await, end_time, end_time - time.now().await);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    info!(target: "integration-test", " --- LOG REVIEW ---");
    let event_log = audit.event_log();
    info!(target: "integration-test", "received {} events", event_log.len());
    // for event in event_log {
    //     info!(target: "integration-test", " - {} Event: {}",event.timestamp(), event.event_type());
    // }

    engine.stop().await;
}
