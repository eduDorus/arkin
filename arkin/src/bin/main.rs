use std::{cmp::max, str::FromStr, sync::Arc};

use anyhow::Result;
use arkin_exec_binance::BinanceExecution;
use arkin_exec_strat_default::DefaultExecutionStrategy;
use arkin_exec_strat_wide::WideQuoterExecutionStrategy;
use clap::Parser;
use rust_decimal::dec;
use tokio::select;
use tokio_rustls::rustls::crypto::{ring, CryptoProvider};
use tracing::info;

use arkin_cli::{Cli, Commands};
use arkin_core::prelude::*;
use arkin_data_provider::DataProviderService;
use arkin_persistence::Persistence;

use mimalloc::MiMalloc;
use uuid::Uuid;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// #[tokio::main(flavor = "multi_thread")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_tracing();

    // Install the default CryptoProvider
    CryptoProvider::install_default(ring::default_provider()).expect("Failed to install default CryptoProvider");

    let args = Cli::parse();
    info!("args: {:?}", args);

    match args.command {
        Commands::DataProvider(a) => {
            info!("Starting arkin data provider 🚀");
            info!("DataProvider args: {:?}", a);

            // Init Persistence
            let persistence = Persistence::from_config_test();
            persistence.refresh().await?;

            // Init Redis PubSub
            let pubsub = RedisPubSub::new(persistence.clone())?;

            let mut engine =
                Engine::new(LiveSystemTime::new(), pubsub.clone(), persistence.clone(), InstanceType::Live);
            engine.register("pubsub", pubsub.clone(), 0, 10);

            let data_provider_service = DataProviderService::load_config();
            engine.register("data_provider", Arc::new(data_provider_service), 1, 1);

            engine.start().await;
            engine.wait_for_shutdown().await;
            engine.stop().await;
        }
        Commands::ExecutionStrategy(a) => {
            info!("Starting arkin execution strategy 🚀");
            info!("Execution Strategy args: {:?}", a);

            // Init Persistence
            let persistence = Persistence::from_config_test();
            persistence.refresh().await?;

            // Init Redis PubSub
            let pubsub = RedisPubSub::new(persistence.clone())?;

            // Init Execution Strategy
            let execution_orderbook = Arc::new(
                ExecutionOrderBook::builder()
                    .publisher(pubsub.publisher())
                    .autoclean(true)
                    .build(),
            );
            let venue_orderbook =
                Arc::new(VenueOrderBook::builder().publisher(pubsub.publisher()).autoclean(true).build());
            let strategy: Arc<dyn Runnable + Send + Sync> = match a.strategy {
                ExecutionStrategyType::WideQuoter => Arc::new(
                    WideQuoterExecutionStrategy::builder()
                        .exec_order_book(execution_orderbook.clone())
                        .venue_order_book(venue_orderbook.clone())
                        .pct_from_mid(dec!(0.005))
                        .requote_pct_change(dec!(0.0005))
                        .build(),
                ),
                ExecutionStrategyType::Maker | ExecutionStrategyType::Taker => Arc::new(
                    DefaultExecutionStrategy::builder()
                        .exec_order_book(execution_orderbook.clone())
                        .venue_order_book(venue_orderbook.clone())
                        .build(),
                ),
            };

            // Init Engine
            let mut engine =
                Engine::new(LiveSystemTime::new(), pubsub.clone(), persistence.clone(), InstanceType::Live);
            engine.register("pubsub", pubsub.clone(), 0, 10);
            engine.register("execution_strategy", strategy, 1, 1);

            engine.start().await;
            engine.wait_for_shutdown().await;
            engine.stop().await;
        }
        Commands::Execution(a) => {
            info!("Starting arkin execution service 🚀");
            info!("Execution args: {:?}", a);

            // Init Persistence
            let persistence = Persistence::from_config_test();
            persistence.refresh().await?;

            // Init Redis PubSub
            let pubsub = RedisPubSub::new(persistence.clone())?;

            let mut engine =
                Engine::new(LiveSystemTime::new(), pubsub.clone(), persistence.clone(), InstanceType::Live);
            engine.register("pubsub", pubsub.clone(), 0, 10);

            let execution = match a.venue {
                VenueName::Binance => BinanceExecution::from_config(),
                _ => unimplemented!("Execution service for venue {} is not implemented yet", a.venue),
            };

            engine.register("execution", execution, 1, 1);

            engine.start().await;
            engine.wait_for_shutdown().await;
            engine.stop().await;
        }
        Commands::Persistence(a) => {
            info!("Starting arkin persistence service 🚀");
            info!("Persistence args: {:?}", a);

            // Init Persistence
            let instance = Instance::builder()
                .id(Uuid::from_str("38fb7951-07d1-4a45-a999-59a5bf4ab0c1").expect("Failed to parse UUID"))
                .name("persistence".to_string())
                .instance_type(InstanceType::Live)
                .created(LiveSystemTime::new().now().await)
                .updated(LiveSystemTime::new().now().await)
                .build();
            let persistence = Persistence::from_config(instance, false, false, false);
            persistence.refresh().await?;

            // Init Redis PubSub
            let pubsub = RedisPubSub::new(persistence.clone())?;

            let mut engine =
                Engine::new(LiveSystemTime::new(), pubsub.clone(), persistence.clone(), InstanceType::Live);
            engine.register("pubsub", pubsub.clone(), 0, 10);
            engine.register("persistence", persistence.clone(), 0, 10);

            engine.start().await;
            engine.wait_for_shutdown().await;
            engine.stop().await;
        }
        Commands::Audit(a) => {
            info!("Starting arkin logger 🚀");
            info!("Logger args: {:?}", a);

            // Init Persistence
            let persistence = Persistence::from_config_test();
            persistence.refresh().await?;

            // Init Redis PubSub
            let pubsub = RedisPubSub::new(persistence.clone())?;
            let receiver = pubsub.subscribe(EventFilter::AllWithoutMarket);

            loop {
                select! {
                  Some(msg) = receiver.recv() => {
                      info!("{}", msg);
                  },
                  _ = tokio::signal::ctrl_c() => {
                      info!("Shutting down logger...");
                      break;
                  }
                }
            }
        }
        Commands::SendOrder(a) => {
            // Init time
            let time = LiveSystemTime::new();

            // Init Persistence
            let persistence = Persistence::from_config_test();
            persistence.refresh().await?;

            // Init Redis PubSub
            let pubsub = RedisPubSub::new(persistence.clone())?;

            let strategy = persistence
                .get_strategy(
                    &StrategyQuery::builder()
                        .id(Uuid::parse_str("41ba36fb-6171-4d5f-a4b4-25eb5415e426").expect("Invalid UUID"))
                        .build(),
                )
                .await?;

            // ETH USDT SPOT: "95b3e6e7-d39d-4cea-a70b-a4da23103a0a"
            // ETH USDT PERP: "0a6400f4-abb5-4ff3-8720-cf2eeebef26e"
            let inst = persistence
                .get_instrument(&InstrumentQuery::builder().id(a.instrument).build())
                .await?;

            info!("Sending orders for {}", inst);

            let last_price = persistence.get_last_tick(&inst).await.unwrap().unwrap().mid_price();

            let lot_size = max(a.amount / last_price, inst.lot_size);

            match a.strategy {
                ExecutionStrategyType::WideQuoter => {
                    // Create Buy exec order
                    let buy_exec_id = Uuid::new_v4();
                    let buy_exec = ExecutionOrder::builder()
                        .id(buy_exec_id)
                        .strategy(Some(strategy.clone()))
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
                        .strategy(Some(strategy.clone()))
                        .instrument(inst.clone())
                        .exec_strategy_type(ExecutionStrategyType::WideQuoter)
                        .side(MarketSide::Sell)
                        .set_price(dec!(0))
                        .set_quantity(lot_size)
                        .status(ExecutionOrderStatus::New)
                        .created(time.now().await)
                        .updated(time.now().await)
                        .build();

                    pubsub.publish(Event::NewExecutionOrder(buy_exec.clone().into())).await;
                    pubsub.publish(Event::NewExecutionOrder(sell_exec.clone().into())).await;
                }
                ExecutionStrategyType::Maker | ExecutionStrategyType::Taker => {
                    let side = a.side.expect("Side is required for Maker/Taker orders");
                    let price = if a.strategy == ExecutionStrategyType::Maker {
                        a.price.expect("Price is required for Maker orders")
                    } else {
                        dec!(0)
                    };

                    let exec_id = Uuid::new_v4();
                    let exec_order = ExecutionOrder::builder()
                        .id(exec_id)
                        .strategy(Some(strategy.clone()))
                        .instrument(inst.clone())
                        .exec_strategy_type(a.strategy)
                        .side(side)
                        .set_price(price)
                        .set_quantity(lot_size)
                        .status(ExecutionOrderStatus::New)
                        .created(time.now().await)
                        .updated(time.now().await)
                        .build();

                    pubsub.publish(Event::NewExecutionOrder(exec_order.clone().into())).await;
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
        Commands::CancelAllOrders => {
            // Init time
            let time = LiveSystemTime::new();

            // Init Persistence
            let persistence = Persistence::from_config_test();
            persistence.refresh().await?;

            // Init Redis PubSub
            let pubsub = RedisPubSub::new(persistence.clone())?;

            pubsub.publish(Event::CancelAllExecutionOrders(time.now().await)).await;

            info!("Publishing CancelAllOrders event");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
        _ => {
            unimplemented!()
        } // Commands::DownloadBinance(a) => {
          //     info!("Starting arkin downloader 🚀");
          //     let time = LiveSystemTime::new();
          //     let pubsub = PubSub::new(true);

          //     let config = load::<PersistenceConfig>();
          //     let instance = Instance::builder()
          //         .id(Uuid::from_str("fcdad148-4ecf-4989-89d9-89c21d50f9b1").unwrap())
          //         .name("downloader".to_owned())
          //         .instance_type(InstanceType::Utility)
          //         .created(time.now().await)
          //         .updated(time.now().await)
          //         .build();

          //     let persistence = Persistence::new(&config, instance, false, false, a.dry_run);

          //     let binance_historical_ingestor = Arc::new(
          //         BinanceHistoricalIngestor::builder()
          //             .venue(a.venue)
          //             .channel(a.channel)
          //             .start(a.start)
          //             .end(a.end)
          //             .build(),
          //     );

          //     // Create the engine
          //     let engine = Engine::new(time.clone(), pubsub.clone(), persistence.clone());
          //     // let engine = Engine::builder()
          //     //     .time(time.clone())
          //     //     .pubsub(pubsub.clone())
          //     //     .persistence(persistence.clone())
          //     //     .build();
          //     engine.register("pubsub", pubsub, 0, 10, None).await;
          //     engine
          //         .register("persistence", persistence, 0, 10, Some(EventFilter::Persistable))
          //         .await;
          //     engine.register("ingestor", binance_historical_ingestor, 1, 9, None).await;

          //     // Start the engine
          //     engine.start().await;
          //     engine.wait_for_shutdown().await;
          // }
          // Commands::DownloadTardis(a) => {
          //     info!("Starting arkin downloader 🚀");
          //     let time = LiveSystemTime::new();

          //     let pubsub = PubSub::new(true);

          //     let config = load::<PersistenceConfig>();
          //     let instance = Instance::builder()
          //         .id(Uuid::from_str("fcdad148-4ecf-4989-89d9-89c21d50f9b1").unwrap())
          //         .name("downloader".to_owned())
          //         .instance_type(InstanceType::Utility)
          //         .created(time.now().await)
          //         .updated(time.now().await)
          //         .build();
          //     let persistence = Persistence::new(&config, instance, false, false, a.dry_run);

          //     let venue = persistence.get_venue_by_name(&a.venue).await.unwrap();
          //     let instruments = persistence.get_instruments_by_venue(&venue).await.unwrap();
          //     let instrument_str = instruments.iter().map(|i| i.venue_symbol.clone()).collect::<Vec<_>>();
          //     // Lowercase if exchange is binance
          //     let instrument_str = if a.venue.to_string().to_lowercase().contains("binance") {
          //         instrument_str.into_iter().map(|s| s.to_lowercase()).collect::<Vec<_>>()
          //     } else {
          //         instrument_str
          //     };

          //     // Create the engine
          //     let engine = Engine::new(time.clone(), pubsub.clone(), persistence.clone());
          //     engine.register("pubsub", pubsub, 0, 10, None).await;
          //     engine
          //         .register("persistence", persistence, 0, 10, Some(EventFilter::Persistable))
          //         .await;

          //     // Chunk to max 50 instruments per ingestor
          //     let cfg = load::<TardisConfig>();
          //     for (i, chunk) in instrument_str.chunks(50).enumerate() {
          //         let ingestor = Arc::new(
          //             TardisIngestor::builder()
          //                 .venue(a.venue)
          //                 .channel(a.channel)
          //                 .start(a.start)
          //                 .end(a.end)
          //                 .instruments(chunk.to_vec())
          //                 .max_concurrent_requests(cfg.tardis.max_concurrent_requests)
          //                 .base_url(cfg.tardis.http_url.clone())
          //                 .api_secret(cfg.tardis.api_secret.clone())
          //                 .build(),
          //         );
          //         engine.register(&format!("downloader_{}", i), ingestor, 1, 9, None).await;
          //     }
          //     engine.start().await;
          //     engine.wait_for_shutdown().await;
          // }
          // Commands::Ingestor(a) => {
          //     info!("Starting arkin ingestor 🚀");
          //     let time = LiveSystemTime::new();
          //     let pubsub = PubSub::new(true);

          //     let config = load::<PersistenceConfig>();
          //     let instance = Instance::builder()
          //         .id(Uuid::from_str("0387efa2-2d8b-4d40-9244-a4377697556a").unwrap())
          //         .name("binance-ingestor".to_owned())
          //         .instance_type(InstanceType::Utility)
          //         .created(time.now().await)
          //         .updated(time.now().await)
          //         .build();
          //     let persistence = Persistence::new(&config, instance, false, false, a.dry_run);
          //     let venue = persistence.get_venue_by_name(&VenueName::BinanceUsdmFutures).await.unwrap();
          //     let instruments = persistence
          //         .list_instruments_by_venue_symbol(&a.instruments, &venue)
          //         .await
          //         .unwrap();

          //     let ingestor = Arc::new(BinanceIngestor::builder().instruments(instruments.clone()).venue(venue).build());

          //     let engine = Engine::new(time.clone(), pubsub.clone(), persistence.clone());
          //     engine.register("pubsub", pubsub, 0, 10, None).await;
          //     engine
          //         .register("persistence", persistence, 0, 10, Some(EventFilter::Persistable))
          //         .await;
          //     engine.register("ingestor", ingestor, 1, 9, None).await;
          //     engine.start().await;
          //     engine.wait_for_shutdown().await;
          // }
          // Commands::Scaler(a) => {
          //     info!("Starting arkin scaler init 🚀");

          //     let start = a.start;
          //     let end = a.end;

          //     let time = LiveSystemTime::new();

          //     let config = load::<PersistenceConfig>();
          //     let instance = Instance::builder()
          //         .id(Uuid::from_str("b787c86a-aff3-4495-b898-008f0fde633c").unwrap())
          //         .name("insights".to_owned())
          //         .instance_type(InstanceType::Insights)
          //         .created(time.now().await)
          //         .updated(time.now().await)
          //         .build();
          //     let persistence = Persistence::new(&config, instance, false, false, a.dry_run);

          //     let pipeline = persistence.get_pipeline_by_name(&a.pipeline).await.unwrap();
          //     let venue = persistence.get_venue_by_name(&VenueName::BinanceUsdmFutures).await.unwrap();
          //     let instruments = persistence
          //         .list_instruments_by_venue_symbol(&a.instruments, &venue)
          //         .await
          //         .unwrap();

          //     // Calculate quantiles
          //     let levels: Vec<f64> = (1..a.n_quantiles).map(|i| i as f64 / a.n_quantiles as f64).collect();

          //     let mut quantiles = Vec::new();
          //     for inst in instruments {
          //         let data = persistence
          //             .get_scaler_data(&pipeline, &inst, start, end, &levels)
          //             .await
          //             .unwrap();
          //         quantiles.extend(data);
          //     }

          //     let data = Quantiles {
          //         pipeline_id: pipeline.id,
          //         levels,
          //         data: quantiles,
          //     };

          //     // Save to json file:
          //     if !a.dry_run {
          //         let pipeline_name = pipeline.name.clone();
          //         let file = std::fs::File::create(format!("./scalers/{pipeline_name}.json")).unwrap();
          //         serde_json::to_writer(file, &data).unwrap();
          //     }
          // }
          // Commands::Insights(a) => {
          //     info!("Starting arkin insights 🚀");

          //     // Start and end time
          //     let start_time = a.start;
          //     let end_time = a.end;

          //     let time = MockTime::new_from(start_time, a.tick_frequency);

          //     // Init pubsub
          //     let pubsub = PubSub::new(false);

          //     // Init persistence
          //     let config = load::<PersistenceConfig>();
          //     let instance = Instance::builder()
          //         .id(Uuid::from_str("b787c86a-aff3-4495-b898-008f0fde633c").unwrap())
          //         .name("insights".to_owned())
          //         .instance_type(InstanceType::Insights)
          //         .created(time.now().await)
          //         .updated(time.now().await)
          //         .build();
          //     let persistence = Persistence::new(&config, instance, a.only_normalized, a.only_predictions, a.dry_run);

          //     let tasks = vec![
          //         ReplayTask::builder()
          //             .venue(VenueName::BinanceUsdmFutures)
          //             .channel(Channel::AggTrades)
          //             .build(),
          //         ReplayTask::builder()
          //             .venue(VenueName::BinanceSpot)
          //             .channel(Channel::AggTrades)
          //             .build(),
          //         ReplayTask::builder()
          //             .venue(VenueName::BinanceUsdmFutures)
          //             .channel(Channel::MarkPriceKlines)
          //             .build(),
          //         // ReplayTask::builder()
          //         //     .venue(VenueName::BinanceUsdmFutures)
          //         //     .channel(Channel::Ticker)
          //         //     .build(),
          //     ];

          //     let ingestor = Arc::new(
          //         SimIngestor::builder()
          //             .replay_tasks(tasks)
          //             .start(start_time)
          //             .end(end_time)
          //             .build(),
          //     );
          //     // let venue = persistence.get_venue_by_name(&VenueName::BinanceUsdmFutures).await.unwrap();
          //     // let instruments = persistence
          //     //     .list_instruments_by_venue_symbol(&a.instruments, &venue)
          //     //     .await
          //     //     .unwrap();

          //     // Insights service
          //     // let pipeline_config = load::<InsightsConfig>();
          //     // let pipeline_info = persistence
          //     //     .get_pipeline_by_name(&a.pipeline)
          //     //     .await
          //     //     .expect("Pipeline not found in database");
          //     // // if let Err(e) = persistence.insert_pipeline(pipeline_info.clone()).await {
          //     // //     error!("{}", e);
          //     // // }
          //     // let insights =
          //     //     Insights::new(pipeline_info, &pipeline_config.insights_service.pipeline, instruments, a.warmup);

          //     // Setup engine
          //     let engine = Engine::new(time.clone(), pubsub.clone(), persistence.clone());
          //     engine.register("pubsub", pubsub, 0, 10, None).await;
          //     engine
          //         .register("persistence", persistence, 0, 10, Some(EventFilter::Persistable))
          //         .await;
          //     engine.register("ingestor", ingestor, 1, 7, None).await;
          //     // engine
          //     //     .register(
          //     //         "insights",
          //     //         insights,
          //     //         0,
          //     //         8,
          //     //         Some(EventFilter::Events(vec![
          //     //             EventType::AggTradeUpdate,
          //     //             EventType::TickUpdate,
          //     //             EventType::MetricUpdate,
          //     //             EventType::InsightsTick,
          //     //         ])),
          //     //     )
          //     //     .await;

          //     engine.start().await;
          //     engine.wait_for_shutdown().await;
          // }
          // Commands::Simulation(a) => {
          //     info!("Starting arkin Simulation 🚀");

          //     // Start and end time
          //     let start_time = a.start;
          //     let end_time = a.end;

          //     let time = SimulationSystemTime::new(start_time, end_time, Duration::from_secs(a.tick_frequency));

          //     // Init pubsub
          //     let pubsub = PubSub::new(true);

          //     // Init persistence
          //     let config = load::<PersistenceConfig>();
          //     let instance = Instance::builder()
          //         .id(Uuid::new_v4())
          //         .name(a.instance_name.to_owned())
          //         .instance_type(InstanceType::Simulation)
          //         .created(time.now().await)
          //         .updated(time.now().await)
          //         .build();
          //     let persistence = Persistence::new(&config, instance, false, false, a.dry_run);

          //     // let venue = persistence.get_venue_by_name(&VenueName::BinanceUsdmFutures).await.unwrap();
          //     // let instruments = persistence
          //     //     .list_instruments_by_venue_symbol(&a.instruments, &venue)
          //     //     .await
          //     //     .unwrap();

          //     // Init accounting
          //     let ledger = Ledger::new(pubsub.publisher());
          //     let accounting = Arc::new(Accounting::new(ledger));

          //     // Init sim ingestor
          //     // let binance_sim_ingestor = Arc::new(
          //     //     SimBinanceIngestor::builder()
          //     //         .start(start_time)
          //     //         .end(end_time + Duration::from_secs(3600))
          //     //         .instruments(instruments.clone())
          //     //         .buffer_size(1)
          //     //         .build(),
          //     // );

          //     // Insights service
          //     let pipeline_config = load::<InsightsConfig>();
          //     let pipeline_meat = Pipeline::builder()
          //         .id(Uuid::new_v4())
          //         .name(a.pipeline.to_owned())
          //         .description("Pipeline used for test purpuse".to_owned())
          //         .created(time.now().await)
          //         .updated(time.now().await)
          //         .build();
          //     let insights = InsightService::new(
          //         persistence.clone(),
          //         pipeline_meat.into(),
          //         &pipeline_config.insights_service.pipeline,
          //     )
          //     .await;

          //     // Crossover strategy
          //     // let strategy_instance = Strategy::builder()
          //     //     .id(Uuid::from_str("9433328f-8f55-4357-a639-85350dec93d2").expect("Invalid UUID"))
          //     //     .name("crossover".into())
          //     //     .description(Some("This strategy is only for testing".into()))
          //     //     .created(time.now().await)
          //     //     .updated(time.now().await)
          //     //     .build();
          //     // let strategy_instance = Arc::new(strategy_instance);
          //     // let strategy = Arc::new(
          //     //     CrossoverStrategy::builder()
          //     //         .strategy(strategy_instance)
          //     //         .allocation_limit_per_instrument(dec!(10000))
          //     //         .fast_ma(FeatureId::new("vwap_price_ema_10".into()))
          //     //         .slow_ma(FeatureId::new("vwap_price_ema_60".into()))
          //     //         .build(),
          //     // );

          //     let strategy_instance = Strategy::builder()
          //         .id(Uuid::from_str("bf59f914-3304-4f57-89ea-c098b9af3f59").expect("Invalid UUID"))
          //         .name("agent".into())
          //         .description(Some("This strategy is only for testing".into()))
          //         .created(time.now().await)
          //         .updated(time.now().await)
          //         .build()
          //         .into();
          //     let agent_strat_cfg = load::<AgentStrategyConfig>().strategy_agent;
          //     let strat_cfg = agent_strat_cfg.strategy;
          //     let model_cfg = agent_strat_cfg.model;
          //     let strategy = AgentStrategy::new(
          //         strategy_instance,
          //         strat_cfg.capital_per_inst,
          //         strat_cfg.inference_interval,
          //         strat_cfg.input_feature_ids,
          //         strat_cfg.input_state_ids,
          //         strat_cfg.inference_host,
          //         strat_cfg.inference_port,
          //         strat_cfg.inference_type,
          //         model_cfg,
          //     );

          //     // Exec Strategy
          //     let execution_order_book = ExecutionOrderBook::new(pubsub.publisher(), true);
          //     let venue_order_book = VenueOrderBook::new(pubsub.publisher(), true);
          //     let exec_strategy = Arc::new(
          //         TakerExecutionStrategy::builder()
          //             .exec_order_book(execution_order_book.to_owned())
          //             .venue_order_book(venue_order_book.to_owned())
          //             .build(),
          //     );

          //     // Executor
          //     let sim_venue = persistence.get_venue_by_name(&VenueName::BinanceUsdmFutures).await.unwrap();
          //     let usdt_asset = persistence.get_asset_by_symbol("usdt").await.unwrap();
          //     let mut init_balance = HashMap::new();
          //     init_balance.insert(usdt_asset, dec!(100000));
          //     let execution = SimulationExecutor::new(sim_venue, init_balance);

          //     // Setup engine
          //     let engine = Engine::new(time.clone(), pubsub.clone(), persistence.clone());
          //     engine.register("pubsub", pubsub, 0, 10, None).await;
          //     engine
          //         .register("persistence", persistence, 1, 10, Some(EventFilter::PersistableSimulation))
          //         .await;
          //     engine
          //         .register(
          //             "accounting",
          //             accounting,
          //             2,
          //             10,
          //             Some(EventFilter::Events(vec![
          //                 EventType::InitialAccountUpdate,
          //                 EventType::ReconcileAccountUpdate,
          //                 EventType::VenueAccountUpdate,
          //                 EventType::VenueTradeUpdate,
          //             ])),
          //         )
          //         .await;
          //     engine
          //         .register(
          //             "insights",
          //             insights,
          //             3,
          //             10,
          //             Some(EventFilter::Events(vec![
          //                 EventType::AggTradeUpdate,
          //                 EventType::TickUpdate,
          //                 EventType::InsightsTick,
          //             ])),
          //         )
          //         .await;
          //     // engine.register("ingestor", binance_sim_ingestor, 4, 10, None).await;
          //     engine
          //         .register(
          //             "strat-agent",
          //             strategy,
          //             2,
          //             10,
          //             Some(EventFilter::Events(vec![
          //                 EventType::InsightsUpdate,
          //                 EventType::WarmupInsightsUpdate,
          //                 EventType::TickUpdate,
          //             ])),
          //         )
          //         .await;
          //     engine
          //         .register(
          //             "exec-strat-taker",
          //             exec_strategy,
          //             2,
          //             10,
          //             Some(EventFilter::Events(vec![
          //                 EventType::NewTakerExecutionOrder,
          //                 EventType::CancelTakerExecutionOrder,
          //                 EventType::CancelAllTakerExecutionOrders,
          //                 EventType::VenueOrderInflight,
          //                 EventType::VenueOrderPlaced,
          //                 EventType::VenueOrderRejected,
          //                 EventType::VenueOrderFill,
          //                 EventType::VenueOrderCancelled,
          //                 EventType::VenueOrderExpired,
          //             ])),
          //         )
          //         .await;
          //     engine
          //         .register(
          //             "exec",
          //             execution,
          //             2,
          //             10,
          //             Some(EventFilter::Events(vec![
          //                 EventType::NewVenueOrder,
          //                 EventType::CancelVenueOrder,
          //                 EventType::CancelAllVenueOrders,
          //                 EventType::TickUpdate,
          //             ])),
          //         )
          //         .await;

          //     engine.start().await;
          //     engine.wait_for_shutdown().await;
          // }
          // Commands::WideQuoter(a) => {
          //     info!("Starting arkin Live Trading 🚀");
          //     // Init core
          //     let time = LiveSystemTime::new();
          //     let pubsub = PubSub::new(false);

          //     // Init persistence
          //     let config = load::<PersistenceConfig>();
          //     let instance = Instance::builder()
          //         .id(Uuid::from_str("e2d051a3-c7fe-4cb8-b844-9545eda0a8ae").unwrap())
          //         .name("wide-quoter".to_owned())
          //         .instance_type(InstanceType::Live)
          //         .created(time.now().await)
          //         .updated(time.now().await)
          //         .build();
          //     let persistence = Persistence::new(&config, instance, false, false, false);

          //     let venue = persistence.get_venue_by_name(&VenueName::BinanceUsdmFutures).await.unwrap();
          //     let instruments = persistence
          //         .list_instruments_by_venue_symbol(&a.instruments, &venue)
          //         .await
          //         .unwrap();

          //     let ingestor = Arc::new(BinanceIngestor::builder().instruments(instruments.clone()).venue(venue).build());

          //     // Init wide quoter strategy
          //     let execution_order_book = ExecutionOrderBook::new(pubsub.publisher(), true);
          //     let venue_order_book = VenueOrderBook::new(pubsub.publisher(), true);
          //     let exec_strat = WideQuoterExecutionStrategy::new(
          //         execution_order_book,
          //         venue_order_book,
          //         a.quote_spread,
          //         a.requote_threshold,
          //     );

          //     // Executor
          //     let execution = BinanceExecution::new();

          //     // Setup engine
          //     let engine = Engine::new(time.clone(), pubsub.clone(), persistence.clone());
          //     engine.register("pubsub", pubsub.clone(), 0, 9, None).await;
          //     engine
          //         .register("persistence", persistence.clone(), 0, 10, Some(EventFilter::Persistable))
          //         .await;
          //     engine.register("ingestor-binance", ingestor, 1, 3, None).await;
          //     engine
          //         .register(
          //             "exec-strat-wide",
          //             exec_strat,
          //             3,
          //             1,
          //             Some(EventFilter::Events(vec![
          //                 EventType::NewWideQuoterExecutionOrder,
          //                 EventType::CancelWideQuoterExecutionOrder,
          //                 EventType::CancelAllWideQuoterExecutionOrders,
          //                 EventType::VenueOrderInflight,
          //                 EventType::VenueOrderPlaced,
          //                 EventType::VenueOrderRejected,
          //                 EventType::VenueOrderFill,
          //                 EventType::VenueOrderCancelled,
          //                 EventType::VenueOrderExpired,
          //                 EventType::TickUpdate,
          //             ])),
          //         )
          //         .await;
          //     engine
          //         .register(
          //             "exec-binance",
          //             execution,
          //             1,
          //             2,
          //             Some(EventFilter::Events(vec![EventType::NewVenueOrder, EventType::CancelVenueOrder])),
          //         )
          //         .await;
          //     engine.start().await;

          //     tokio::time::sleep(Duration::from_secs(1)).await;

          //     let strategy = Strategy::builder()
          //         .id(Uuid::parse_str("41ba36fb-6171-4d5f-a4b4-25eb5415e426").expect("Invalid UUID"))
          //         .name("wide_quoter".into())
          //         .description(Some("This strategy quotes around the mid price".into()))
          //         .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
          //         .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
          //         .build();
          //     let strategy = Arc::new(strategy);
          //     for inst in instruments {
          //         info!("Sending orders for {}", inst);

          //         let last_price = persistence.get_last_tick(&inst).await.unwrap().unwrap().mid_price();

          //         let lot_size = max(dec!(100) / last_price, inst.lot_size);

          //         // Create Buy exec order
          //         let publisher = pubsub.publisher();
          //         let buy_exec_id = Uuid::new_v4();
          //         let buy_exec = ExecutionOrder::builder()
          //             .id(buy_exec_id)
          //             .strategy(Some(strategy.clone()))
          //             .instrument(inst.clone())
          //             .exec_strategy_type(ExecutionStrategyType::WideQuoter)
          //             .side(MarketSide::Buy)
          //             .set_price(dec!(0))
          //             .set_quantity(lot_size)
          //             .status(ExecutionOrderStatus::New)
          //             .created(time.now().await)
          //             .updated(time.now().await)
          //             .build();

          //         // Create Sell exec order
          //         let sell_exec_id = Uuid::new_v4();
          //         let sell_exec = ExecutionOrder::builder()
          //             .id(sell_exec_id)
          //             .strategy(Some(strategy.clone()))
          //             .instrument(inst.clone())
          //             .exec_strategy_type(ExecutionStrategyType::WideQuoter)
          //             .side(MarketSide::Sell)
          //             .set_price(dec!(0))
          //             .set_quantity(lot_size)
          //             .status(ExecutionOrderStatus::New)
          //             .created(time.now().await)
          //             .updated(time.now().await)
          //             .build();

          //         publisher
          //             .publish(Event::NewWideQuoterExecutionOrder(buy_exec.clone().into()))
          //             .await;

          //         publisher
          //             .publish(Event::NewWideQuoterExecutionOrder(sell_exec.clone().into()))
          //             .await;
          //     }

          //     engine.wait_for_shutdown().await;
          // }
          // Commands::Agent(a) => {
          //     info!("Starting arkin Live Trading 🚀");
          //     // Init core
          //     let time = LiveSystemTime::new();
          //     let pubsub = PubSub::new(true);
          //     let cron = Arc::new(Cron::new(vec![CronInterval::new(
          //         time.now().await.replace_second(0).unwrap().replace_nanosecond(0).unwrap(),
          //         Duration::from_secs(a.tick_frequency),
          //         EventType::InsightsTick,
          //     )]));

          //     // Init persistence
          //     let config = load::<PersistenceConfig>();
          //     let instance = Instance::builder()
          //         .id(Uuid::from_str("5639b172-2229-4dc0-ab30-d9d91d6a4a64").unwrap())
          //         .name("agent-v2".to_owned())
          //         .instance_type(InstanceType::Live)
          //         .created(time.now().await)
          //         .updated(time.now().await)
          //         .build();
          //     let persistence = Persistence::new(&config, instance, false, false, false);

          //     let venue = persistence.get_venue_by_name(&VenueName::BinanceUsdmFutures).await.unwrap();
          //     let instruments = persistence
          //         .list_instruments_by_venue_symbol(&a.instruments, &venue)
          //         .await
          //         .unwrap();

          //     let ingestor = Arc::new(BinanceIngestor::builder().instruments(instruments.clone()).venue(venue).build());

          //     let pipeline_config = load::<InsightsConfig>();
          //     let pipeline_meta = Pipeline::builder()
          //         .id(Uuid::new_v4())
          //         .name(a.pipeline.to_owned())
          //         .description("Pipeline used for agent".to_owned())
          //         .created(time.now().await)
          //         .updated(time.now().await)
          //         .build();
          //     let insights = InsightService::new(
          //         persistence.clone(),
          //         pipeline_meta.into(),
          //         &pipeline_config.insights_service.pipeline,
          //     )
          //     .await;

          //     let strategy_instance = Strategy::builder()
          //         .id(Uuid::parse_str("bf59f914-3304-4f57-89ea-c098b9af3f59").expect("Invalid UUID"))
          //         .name("agent".into())
          //         .description(Some("RL Agent".into()))
          //         .created(datetime!(2025-01-01 00:00:00 UTC).to_utc())
          //         .updated(datetime!(2025-01-01 00:00:00 UTC).to_utc())
          //         .build();
          //     let strategy_instance = Arc::new(strategy_instance);
          //     let agent_strat_cfg = load::<AgentStrategyConfig>().strategy_agent;
          //     let strat_cfg = agent_strat_cfg.strategy;
          //     let model_cfg = agent_strat_cfg.model;
          //     let agent_strategy = AgentStrategy::new(
          //         strategy_instance,
          //         strat_cfg.capital_per_inst,
          //         strat_cfg.inference_interval,
          //         strat_cfg.input_feature_ids,
          //         strat_cfg.input_state_ids,
          //         strat_cfg.inference_host,
          //         strat_cfg.inference_port,
          //         strat_cfg.inference_type,
          //         model_cfg,
          //     );

          //     // Init wide quoter strategy
          //     let execution_order_book = ExecutionOrderBook::new(pubsub.publisher(), true);
          //     let venue_order_book = VenueOrderBook::new(pubsub.publisher(), true);
          //     let exec_strat_wide = WideQuoterExecutionStrategy::new(
          //         execution_order_book.clone(),
          //         venue_order_book.clone(),
          //         dec!(0.005),
          //         dec!(0.0002),
          //     );
          //     let exec_strat_taker = TakerExecutionStrategy::new(execution_order_book, venue_order_book);

          //     // // Executor
          //     let execution = BinanceExecution::new();

          //     // Setup engine
          //     let engine = Engine::new(time.clone(), pubsub.clone(), persistence.clone());
          //     engine.register("pubsub", pubsub.clone(), 0, 10, None).await;
          //     engine
          //         .register("persistence", persistence, 0, 9, Some(EventFilter::Persistable))
          //         .await;
          //     engine.register("ingestor", ingestor, 1, 9, None).await;
          //     engine
          //         .register(
          //             "insights",
          //             insights,
          //             2,
          //             8,
          //             Some(EventFilter::Events(vec![
          //                 EventType::AggTradeUpdate,
          //                 EventType::TickUpdate,
          //                 EventType::InsightsTick,
          //             ])),
          //         )
          //         .await;
          //     engine
          //         .register(
          //             "agent",
          //             agent_strategy,
          //             1,
          //             10,
          //             Some(EventFilter::Events(vec![
          //                 EventType::InsightsUpdate,
          //                 EventType::WarmupInsightsUpdate,
          //                 EventType::TickUpdate,
          //             ])),
          //         )
          //         .await;
          //     engine.register("cron", cron, 3, 10, None).await;
          //     engine
          //         .register(
          //             "exec-strat-wide",
          //             exec_strat_wide,
          //             1,
          //             1,
          //             Some(EventFilter::Events(vec![
          //                 EventType::NewWideQuoterExecutionOrder,
          //                 EventType::CancelWideQuoterExecutionOrder,
          //                 EventType::CancelAllWideQuoterExecutionOrders,
          //                 EventType::VenueOrderInflight,
          //                 EventType::VenueOrderPlaced,
          //                 EventType::VenueOrderRejected,
          //                 EventType::VenueOrderFill,
          //                 EventType::VenueOrderCancelled,
          //                 EventType::VenueOrderExpired,
          //                 EventType::TickUpdate,
          //             ])),
          //         )
          //         .await;
          //     engine
          //         .register(
          //             "exec-strat-taker",
          //             exec_strat_taker,
          //             1,
          //             1,
          //             Some(EventFilter::Events(vec![
          //                 EventType::NewTakerExecutionOrder,
          //                 EventType::CancelTakerExecutionOrder,
          //                 EventType::CancelAllTakerExecutionOrders,
          //                 EventType::VenueOrderInflight,
          //                 EventType::VenueOrderPlaced,
          //                 EventType::VenueOrderRejected,
          //                 EventType::VenueOrderFill,
          //                 EventType::VenueOrderCancelled,
          //                 EventType::VenueOrderExpired,
          //             ])),
          //         )
          //         .await;
          //     engine
          //         .register(
          //             "exec-binance",
          //             execution,
          //             1,
          //             2,
          //             Some(EventFilter::Events(vec![EventType::NewVenueOrder, EventType::CancelVenueOrder])),
          //         )
          //         .await;

          //     engine.start().await;
          //     info!("Engine started, waiting for shutdown...");
          //     engine.wait_for_shutdown().await;
          // }
    }

    Ok(())
}
