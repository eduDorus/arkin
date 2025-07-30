use std::{str::FromStr, sync::Arc, time::Duration};

use arkin_accounting::Accounting;
use arkin_binance::BinanceExecution;
use arkin_cli::{Cli, Commands};
use arkin_exec_sim::SimulationExecutor;
use arkin_exec_strat_taker::TakerExecutionStrategy;
use arkin_exec_strat_wide::WideQuoterExecutionStrategy;
use arkin_ingestor_binance::SimBinanceIngestor;
use arkin_ingestor_tardis::{TardisConfig, TardisIngestor};
use arkin_insights::{prelude::InsightsConfig, Insights};
use arkin_persistence::{Persistence, PersistenceConfig};
use arkin_strat_crossover::CrossoverStrategy;
use clap::Parser;
use rust_decimal::dec;
use tokio_rustls::rustls::crypto::{ring, CryptoProvider};
use tracing::{error, info};

use arkin_core::prelude::*;
use uuid::Uuid;

// #[tokio::main(flavor = "current_thread")]
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();

    // Install the default CryptoProvider
    CryptoProvider::install_default(ring::default_provider()).expect("Failed to install default CryptoProvider");

    let args = Cli::parse();
    info!("args: {:?}", args);

    match args.command {
        Commands::Download(a) => {
            info!("Starting arkin downloader 🚀");
            let time = LiveSystemTime::new();

            let pubsub = PubSub::new(time.clone(), true);
            let pubsub_service = Service::new(pubsub.clone(), None);

            let config = load::<PersistenceConfig>();
            let instance = Instance::builder()
                .id(Uuid::from_str("fcdad148-4ecf-4989-89d9-89c21d50f9b1").unwrap())
                .name("downloader".to_owned())
                .instance_type(InstanceType::Utility)
                .created(time.now().await)
                .updated(time.now().await)
                .build();
            let persistence = Persistence::new(&config, instance, false, false, a.dry_run);
            // let persistence_service = Service::new(persistence.to_owned(), None);
            let persistence_service =
                Service::new(persistence.to_owned(), Some(pubsub.subscribe(EventFilter::Persistable)));

            let cfg = load::<TardisConfig>();
            let ingestor = Arc::new(
                TardisIngestor::builder()
                    .publisher(pubsub.publisher())
                    .persistence(persistence)
                    .venue(a.venue.clone())
                    .channel(a.channel.clone())
                    .start(a.start)
                    .end(a.end)
                    .instruments(a.instruments.clone())
                    .max_concurrent_requests(cfg.tardis.max_concurrent_requests)
                    .base_url(cfg.tardis.http_url)
                    .api_secret(cfg.tardis.api_secret)
                    .build(),
            );
            let download_service = Service::new(ingestor, None);

            let engine = Engine::new();
            engine.register(pubsub_service, 0, 10).await;
            engine.register(persistence_service, 0, 10).await;
            engine.register(download_service, 0, 10).await;
            engine.start().await;
            engine.wait_for_shutdown().await;
        }
        Commands::Ingestor(a) => {
            info!("Starting arkin ingestor 🚀");
            let time = LiveSystemTime::new();

            let pubsub = PubSub::new(time.clone(), true);
            let pubsub_service = Service::new(pubsub.clone(), None);

            let config = load::<PersistenceConfig>();
            let instance = Instance::builder()
                .id(Uuid::from_str("fcdad148-4ecf-4989-89d9-89c21d50f9b1").unwrap())
                .name("downloader".to_owned())
                .instance_type(InstanceType::Utility)
                .created(time.now().await)
                .updated(time.now().await)
                .build();
            let persistence = Persistence::new(&config, instance, false, false, a.dry_run);
            // let persistence_service = Service::new(persistence.to_owned(), None);
            let persistence_service =
                Service::new(persistence.to_owned(), Some(pubsub.subscribe(EventFilter::Persistable)));

            let engine = Engine::new();
            engine.register(pubsub_service, 0, 10).await;
            engine.register(persistence_service, 0, 10).await;
            // engine.register(download_service, 0, 10).await;
            engine.start().await;
            engine.wait_for_shutdown().await;
        }
        Commands::Scaler(a) => {
            info!("Starting arkin scaler init 🚀");

            let start = a.start;
            let end = a.end;

            let time = LiveSystemTime::new();

            let config = load::<PersistenceConfig>();
            let instance = Instance::builder()
                .id(Uuid::from_str("b787c86a-aff3-4495-b898-008f0fde633c").unwrap())
                .name("insights".to_owned())
                .instance_type(InstanceType::Insights)
                .created(time.now().await)
                .updated(time.now().await)
                .build();
            let persistence = Persistence::new(&config, instance, false, false, a.dry_run);

            let pipeline = persistence.get_pipeline_by_name(&a.pipeline).await.unwrap();
            let instruments = persistence.list_instruments_by_venue_symbol(&a.instruments).await.unwrap();

            // Calculate quantiles
            let levels: Vec<f64> = (1..a.n_quantiles).map(|i| i as f64 / a.n_quantiles as f64).collect();

            let mut quantiles = Vec::new();
            for inst in instruments {
                let data = persistence
                    .get_scaler_data(&pipeline, &inst, start, end, &levels)
                    .await
                    .unwrap();
                quantiles.extend(data);
            }

            let data = Quantiles {
                pipeline_id: pipeline.id,
                levels,
                data: quantiles,
            };

            // Save to json file:
            if !a.dry_run {
                let pipeline_name = pipeline.name.clone();
                let file = std::fs::File::create(format!("./scalers/{pipeline_name}.json")).unwrap();
                serde_json::to_writer(file, &data).unwrap();
            }
        }
        Commands::Insights(a) => {
            info!("Starting arkin insights 🚀");

            // Start and end time
            let start_time = a.start;
            let end_time = a.end;

            let time = MockTime::new_from(start_time, a.tick_frequency);

            // Init pubsub
            let pubsub = PubSub::new(time.clone(), true);
            let pubsub_service = Service::new(pubsub.clone(), None);

            // Init persistence
            let config = load::<PersistenceConfig>();
            let instance = Instance::builder()
                .id(Uuid::from_str("b787c86a-aff3-4495-b898-008f0fde633c").unwrap())
                .name("insights".to_owned())
                .instance_type(InstanceType::Insights)
                .created(time.now().await)
                .updated(time.now().await)
                .build();
            let persistence = Persistence::new(&config, instance, a.only_normalized, a.only_predictions, a.dry_run);
            let persistence_service =
                Service::new(persistence.to_owned(), Some(pubsub.subscribe(EventFilter::Insights)));

            let mut instruments = Vec::new();
            for inst in a.instruments {
                match persistence.get_instrument_by_venue_symbol(&inst).await {
                    Ok(i) => instruments.push(i),
                    Err(e) => panic!("{}", e),
                }
            }

            // Init sim ingestor
            let binance_ingestor = Arc::new(
                SimBinanceIngestor::builder()
                    .identifier("sim-binance-ingestor".to_owned())
                    ._time(time.to_owned())
                    .start(start_time)
                    .end(end_time + Duration::from_secs(3600))
                    .instruments(instruments.clone())
                    .persistence(persistence.to_owned())
                    .publisher(pubsub.publisher())
                    .build(),
            );
            let binance_ingestor_service = Service::new(binance_ingestor, None);

            // Insights service
            let pipeline_config = load::<InsightsConfig>();
            let pipeline_info: Arc<Pipeline> = Pipeline::builder()
                .id(Uuid::from_str("f031d4e2-2ada-4651-83fa-aef515accb29").unwrap())
                .name(a.pipeline)
                .description("Pipeline version v1.6.0 (Multi Asset)".to_owned())
                .created(time.now().await)
                .updated(time.now().await)
                .build()
                .into();
            if let Err(e) = persistence.insert_pipeline(pipeline_info.clone()).await {
                error!("{}", e);
            }
            let insights = Insights::new(
                pubsub.publisher(),
                pipeline_info,
                &pipeline_config.insights_service.pipeline,
                instruments,
                a.warmup,
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

            // Setup engine
            let engine = Engine::new();
            engine.register(pubsub_service, 0, 10).await;
            engine.register(persistence_service, 0, 9).await;
            engine.register(insights_service, 0, 8).await;
            engine.register(binance_ingestor_service, 1, 7).await;

            engine.start().await;
            engine.wait_for_shutdown().await;
        }
        Commands::Simulation(a) => {
            info!("Starting arkin Simulation 🚀");

            // Start and end time
            let start_time = a.start;
            let end_time = a.end;

            let time = MockTime::new_from(start_time, a.tick_frequency);

            // Init pubsub
            let pubsub = PubSub::new(time.clone(), true);
            let pubsub_service = Service::new(pubsub.clone(), None);

            // Init persistence
            let config = load::<PersistenceConfig>();
            let instance = Instance::builder()
                .id(Uuid::new_v4())
                .name(a.instance_name.to_owned())
                .instance_type(InstanceType::Simulation)
                .created(time.now().await)
                .updated(time.now().await)
                .build();
            let persistence = Persistence::new(&config, instance, false, false, a.dry_run);
            let persistence_service =
                Service::new(persistence.to_owned(), Some(pubsub.subscribe(EventFilter::PersistableNoMarket)));

            let mut instruments = Vec::new();
            for inst in a.instruments {
                match persistence.get_instrument_by_venue_symbol(&inst).await {
                    Ok(i) => instruments.push(i),
                    Err(e) => panic!("{}", e),
                }
            }

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
            // let audit = Audit::new("audit");
            // let audit_service = Service::new(audit.to_owned(), Some(pubsub.subscribe(EventFilter::AllWithoutMarket)));

            // Init sim ingestor
            let binance_sim_ingestor = Arc::new(
                SimBinanceIngestor::builder()
                    .identifier("sim-binance-ingestor".to_owned())
                    ._time(time.to_owned())
                    .start(start_time)
                    .end(end_time + Duration::from_secs(3600))
                    .instruments(instruments.clone())
                    .persistence(persistence.to_owned())
                    .publisher(pubsub.publisher())
                    .build(),
            );
            let binance_ingestor_service = Service::new(binance_sim_ingestor, None);

            // Insights service
            let pipeline_config = load::<InsightsConfig>();
            let pipeline_info = Pipeline::builder()
                .id(Uuid::new_v4())
                .name(a.pipeline.to_owned())
                .description("Pipeline used for test purpuse".to_owned())
                .created(time.now().await)
                .updated(time.now().await)
                .build();
            let insights = Insights::new(
                pubsub.publisher(),
                pipeline_info.into(),
                &pipeline_config.insights_service.pipeline,
                instruments,
                a.warmup,
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
            let strategy = Strategy::builder()
                .id(Uuid::from_str("9433328f-8f55-4357-a639-85350dec93d2").expect("Invalid UUID"))
                .name("crossover".into())
                .description(Some("This strategy is only for testing".into()))
                .created(time.now().await)
                .updated(time.now().await)
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

            // let strategy_name = Strategy::builder()
            //     .id(Uuid::from_str("bf59f914-3304-4f57-89ea-c098b9af3f59").expect("Invalid UUID"))
            //     .name("agent".into())
            //     .description(Some("This strategy is only for testing".into()))
            //     .created(time.now().await)
            //     .updated(time.now().await)
            //     .build()
            //     .into();
            // let strategy = AgentStrategy::new(time.clone(), pubsub.publisher(), strategy_name);
            // let strategy_service = Service::new(
            //     strategy,
            //     Some(pubsub.subscribe(EventFilter::Events(vec![EventType::InsightsUpdate]))),
            // );

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
            engine.register(accounting_service, 0, 10).await;
            engine.register(binance_ingestor_service, 0, 10).await;
            engine.register(insights_service, 0, 10).await;
            engine.register(strategy_service, 0, 10).await;
            engine.register(exec_strategy_service, 0, 10).await;
            engine.register(execution_service, 0, 10).await;
            engine.register(pubsub_service, 0, 10).await;

            engine.start().await;
            engine.wait_for_shutdown().await;
        }
        Commands::Live(a) => {
            info!("Starting arkin Live Trading 🚀");
            // Init mock time
            let time = LiveSystemTime::new();

            // Init pubsub
            let pubsub = PubSub::new(time.clone(), true);
            let pubsub_service = Service::new(pubsub.clone(), None);

            // Init audit
            // let audit = Audit::new("audit");
            // let audit_service = Service::new(audit.to_owned(), Some(pubsub.subscribe(EventFilter::All)));

            // Init persistence
            let config = load::<PersistenceConfig>();
            let instance = Instance::builder()
                .id(Uuid::from_str("e2d051a3-c7fe-4cb8-b844-9545eda0a8ae").unwrap())
                .name("wide-quoter".to_owned())
                .instance_type(InstanceType::Live)
                .created(time.now().await)
                .updated(time.now().await)
                .build();
            let persistence = Persistence::new(&config, instance, false, false, false);
            let persistence_service =
                Service::new(persistence.to_owned(), Some(pubsub.subscribe(EventFilter::Persistable)));

            let instruments = persistence.list_instruments_by_venue_symbol(&a.instruments).await.unwrap();

            // Init wide quoter strategy
            let execution_order_book = ExecutionOrderBook::new(pubsub.publisher(), true);
            let venue_order_book = VenueOrderBook::new(pubsub.publisher(), true);
            let exec_strat = WideQuoterExecutionStrategy::new(
                "wide-quoter",
                time.clone(),
                pubsub.publisher(),
                execution_order_book,
                venue_order_book,
                dec!(0.002),
                dec!(0.0005),
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
            let execution = BinanceExecution::new(time.clone(), pubsub.publisher(), instruments.clone());
            let execution_service = Service::new(
                execution,
                Some(pubsub.subscribe(EventFilter::Events(vec![
                    EventType::NewVenueOrder,
                    EventType::CancelVenueOrder,
                    // EventType::CancelAllVenueOrders,
                    // EventType::TickUpdate,
                ]))),
            );

            // Setup engine
            let engine = Engine::new();
            engine.register(pubsub_service, 0, 10).await;
            engine.register(persistence_service, 1, 9).await;
            // engine.register(audit_service, 1, 9).await;
            engine.register(exec_strat_service, 2, 1).await;
            engine.register(execution_service, 2, 2).await;
            engine.start().await;

            for inst in instruments {
                info!("Sending orders for {}", inst);

                let lot_size = if inst.venue_symbol != "BTCUSDT" {
                    inst.lot_size * dec!(10)
                } else {
                    inst.lot_size
                };

                // Create Buy exec order
                let publisher = pubsub.publisher();
                let buy_exec_id = Uuid::new_v4();
                let buy_exec = ExecutionOrder::builder()
                    .id(buy_exec_id)
                    .strategy(Some(test_strategy_1()))
                    .instrument(inst.clone())
                    .exec_strategy_type(ExecutionStrategyType::WideQuoter)
                    .side(MarketSide::Buy)
                    .set_price(dec!(0))
                    .set_quantity(lot_size)
                    .status(ExecutionOrderStatus::New)
                    .created(time.now().await)
                    .updated(time.now().await)
                    .build();

                // Create Sell exec order
                let sell_exec_id = Uuid::new_v4();
                let sell_exec = ExecutionOrder::builder()
                    .id(sell_exec_id)
                    .strategy(Some(test_strategy_1()))
                    .instrument(inst.clone())
                    .exec_strategy_type(ExecutionStrategyType::WideQuoter)
                    .side(MarketSide::Sell)
                    .set_price(dec!(0))
                    .set_quantity(lot_size)
                    .status(ExecutionOrderStatus::New)
                    .created(time.now().await)
                    .updated(time.now().await)
                    .build();

                publisher
                    .publish(Event::NewWideQuoterExecutionOrder(buy_exec.clone().into()))
                    .await;

                publisher
                    .publish(Event::NewWideQuoterExecutionOrder(sell_exec.clone().into()))
                    .await;
            }

            engine.wait_for_shutdown().await;

            // info!(target: "integration-test", " --- LOG REVIEW ---");
            // let event_log = audit.event_log();
            // info!(target: "integration-test", "received {} events", event_log.len());
        }
    }
}
