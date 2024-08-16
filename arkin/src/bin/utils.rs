use anyhow::Result;
use arkin::allocation::AllocationManager;
use arkin::config;
use arkin::constants::TRADE_PRICE_ID;
use arkin::constants::TRADE_QUANTITY_ID;
use arkin::db::DBManager;
use arkin::execution::ExecutionManager;
use arkin::features::FeatureManager;
use arkin::ingestors::BinanceParser;
use arkin::ingestors::TardisChannel;
use arkin::ingestors::TardisExchange;
use arkin::ingestors::TardisRequest;
use arkin::ingestors::TardisService;
use arkin::logging;
use arkin::models::Event;
use arkin::models::FeatureEvent;
use arkin::models::Instrument;
use arkin::models::Venue;
use arkin::state::StateManager;
use arkin::strategies::StrategyManager;
use clap::Parser;
use clap::Subcommand;
use futures_util::StreamExt;
use mimalloc::MiMalloc;
use rust_decimal::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use time::macros::format_description;
use time::OffsetDateTime;
use time::PrimitiveDateTime;
use tokio::pin;
use tracing::debug;
use tracing::error;
use tracing::info;

/// CLI application for X
#[derive(Parser)]
#[clap(
    name = "Arkin Utils",
    version = "0.1.0",
    author = "Dorus Janssens",
    about = "This utility downloads data from various exchanges"
)]

struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Processes streaming data
    Tardis {
        /// Sets the exchange to use
        #[clap(long)]
        exchange: TardisExchange,

        /// Type of data to stream
        #[clap(long)]
        channel: TardisChannel,

        #[clap(long, value_delimiter = ',')]
        instruments: Vec<String>,

        /// Start date for the data stream
        #[clap(long)]
        start: String,

        /// End date for the data stream
        #[clap(long)]
        end: String,
    },

    /// Run pipeline
    Pipeline {
        // /// Filter by exchange
        // #[clap(long)]
        // exchange: Option<String>,

        // /// Filter on instruments
        // #[clap(long)]
        // instrument: Vec<String>,
        /// Filter on start date
        #[clap(long, short)]
        start: String,

        /// Filter on end date
        #[clap(long, short)]
        end: String,

        /// Frequency
        #[clap(long, short)]
        frequency: u64,
    },
}

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    logging::init_tracing();
    info!("Starting Arkin 🚀");

    let config = config::load();
    let manager = Arc::new(DBManager::from_config(&config.db).await);

    let args = Cli::parse();

    match args.command {
        Commands::Tardis {
            exchange,
            channel,
            instruments,
            start,
            end,
        } => {
            let format = format_description!("[year]-[month]-[day] [hour]:[minute]");
            let start = PrimitiveDateTime::parse(&start, &format)?.assume_utc();
            let end = PrimitiveDateTime::parse(&end, &format)?.assume_utc();

            info!("Starting data stream");
            info!("Exchange: {}", exchange);
            info!("Channel: {}", exchange.channel_str(&channel)?);
            info!("Instruments: {:?}", instruments);
            info!("Start: {}", start);
            info!("End: {}", end);

            let req = TardisRequest::new(&exchange, &channel, &instruments, &start, &end);
            let tardis = TardisService::builder()
                .base_url("https://api.tardis.dev/v1/data-feeds".into())
                .max_concurrent_requests(5)
                .build();
            let stream = tardis.stream(req);
            pin!(stream);
            // process_stream_concurrently(stream, manager.clone(), 10).await;

            // batch insert 5000 events
            let mut events = Vec::with_capacity(10000);
            while let Some((_ts, json)) = stream.next().await {
                let event = BinanceParser::parse_swap(&json)?;
                events.push(event);

                if events.len() >= 10000 {
                    if let Err(e) = manager.insert_events_batch(&events).await {
                        error!("Failed to add events: {}", e);
                    }
                    info!("Inserted 10000 events");
                    events.clear();
                }
            }
        }
        Commands::Pipeline {
            start,
            end,
            frequency,
        } => {
            let format = format_description!("[year]-[month]-[day] [hour]:[minute]");
            let start = PrimitiveDateTime::parse(&start, &format)?.assume_utc();
            let end = PrimitiveDateTime::parse(&end, &format)?.assume_utc();

            info!(
                "Starting pipeline from {} to {}",
                start.format(&format).expect("Failed to format date"),
                end.format(&format).expect("Failed to format date")
            );

            // Load data
            let db = Arc::new(DBManager::from_config(&config.db).await);
            let state = Arc::new(StateManager::from_config(&config.state_manager));
            init_state(state.clone(), db.clone(), &start, &end).await;

            // INITIALIZE
            let feature_manager = FeatureManager::from_config(state.clone(), &config.feature_manager);
            let strategy_manager = StrategyManager::from_config(state.clone(), &config.strategy_manager);
            let allocation_manager = AllocationManager::from_config(state.clone(), &config.allocation_manager);
            let execution_manager = ExecutionManager::from_config(state.clone(), &config.execution_manager);

            // RUN
            let timer = Instant::now();
            let instruments = vec![Instrument::perpetual(Venue::Binance, "btc".into(), "usdt".into())];
            let mut timestamp = start + Duration::from_secs(frequency);
            let intervals = ((end - start).whole_seconds() / frequency as i64) - 1;

            for _ in 0..intervals {
                debug!("----------------- {:?} -----------------", timestamp);
                // Run pipeline
                let features = feature_manager.calculate(&timestamp, &instruments);

                // Run strategies
                let signals = strategy_manager.calculate(&timestamp, &features);

                // Run portfolio
                let positions = state.position_snapshot(&timestamp);
                let market = state.market_snapshot(&timestamp);

                // Run allocation
                let allocations = allocation_manager.calculate(&timestamp, &signals, &market, &positions);

                // Run simulation
                execution_manager.execute(allocations);
                // Run analytics

                timestamp += Duration::from_secs(frequency);
            }

            info!("Elapsed time: {:?}", timer.elapsed());
            // info!("Timestamp: {:?}", end);
            // let latest_price = pipeline.get_latest(&instrument, &"trade_price".into(), &end);
            // info!("Latest price: {:?}", latest_price);
            // let latest_quantity = pipeline.get_latest(&instrument, &"trade_quantity".into(), &end);
            // info!("Latest quantity: {:?}", latest_quantity);
            // let range_price = pipeline.get_range(&instrument, &"trade_price".into(), &end, &Duration::from_secs(1));
            // info!("Range price: {:?}", range_price);
            // let periods = pipeline.get_periods(&instrument, &"trade_price".into(), &end, 5);
            // info!("Periods: {:?}", periods);
            // pipeline.calculate();
        }
    }
    Ok(())
}

pub async fn init_state(state: Arc<StateManager>, db: Arc<DBManager>, start: &OffsetDateTime, end: &OffsetDateTime) {
    // Load trades
    let trades = db.read_trades(start, end).await;
    // split trades to feature events
    trades.into_iter().for_each(|t| {
        state.add_event(Event::Trade(t.clone()));
        state.add_feature(FeatureEvent::new(
            TRADE_PRICE_ID.to_owned(),
            t.instrument.clone(),
            t.event_time,
            t.price.value().to_f64().unwrap(),
        ));
        state.add_feature(FeatureEvent::new(
            TRADE_QUANTITY_ID.to_owned(),
            t.instrument,
            t.event_time,
            t.quantity.value().to_f64().unwrap(),
        ));
    });

    // Load ticks
    let ticks = db.read_ticks(start, end).await;
    ticks.into_iter().for_each(|t| {
        state.add_event(t.into());
    });
}
