use std::sync::Arc;

use anyhow::Result;
use arkin::config;
use arkin::db::DBManager;
use arkin::ingestors::BinanceParser;
use arkin::ingestors::TardisChannel;
use arkin::ingestors::TardisExchange;
use arkin::ingestors::TardisRequest;
use arkin::ingestors::TardisService;
use arkin::logging;
use clap::Parser;
use clap::Subcommand;
use futures_util::Stream;
use futures_util::StreamExt;
use mimalloc::MiMalloc;
use time::macros::format_description;
use time::OffsetDateTime;
use time::PrimitiveDateTime;
use tokio::pin;
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

            // Create file in ./tests/data
            // let filename = format!(
            //     "./tests/data/{}_{}_{}_{}_{}.json",
            //     exchange,
            //     channel,
            //     instruments.join("_"),
            //     start.format(format_description!("[year][month][day][hour][minute]"))?,
            //     end.format(format_description!("[year][month][day][hour][minute]"))?
            // );

            // Open file
            // let file = std::fs::File::create(filename)?;
            // let mut writer = std::io::BufWriter::new(file);
            // while let Some((_ts, json)) = stream.next().await {
            //     // let event = BinanceParser::parse_swap(&json)?;
            //     // let event_json = serde_json::to_string(&event)?;
            //     writer.write_all(&json.into_bytes())?;
            //     writer.write_all(b"\n")?;
            // }
            // writer.flush()?;
        }
    }
    Ok(())
}

async fn _process_stream_concurrently(
    stream: impl Stream<Item = (OffsetDateTime, String)>,
    manager: Arc<DBManager>,
    concurrency: usize,
) {
    stream
        .for_each_concurrent(concurrency, |(_, json)| {
            let manager_clone = manager.clone(); // Clone manager for each concurrent operation
            async move {
                // Attempt to parse the JSON
                let res = BinanceParser::parse_swap(&json);
                match res {
                    Ok(event) => {
                        // On success, clone manager and add the event
                        let manager_clone = manager_clone.clone();
                        if let Err(e) = manager_clone.add_event(event).await {
                            // Log error if adding event fails
                            error!("Failed to add event: {}", e);
                        }
                    }
                    Err(e) => {
                        // Log error if parsing fails
                        error!("Failed to parse event: {}", e);
                    }
                }
            }
        })
        .await; // Await the completion of the concurrent operations
}
