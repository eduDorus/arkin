use anyhow::Result;
use aurelion::config;
use aurelion::logging;
use aurelion::services::TardisChannel;
use aurelion::services::TardisExchange;
use aurelion::services::TardisModel;
use aurelion::services::TardisRequest;
use aurelion::services::TardisService;
use clap::Parser;
use clap::Subcommand;
use futures_util::StreamExt;
use mimalloc::MiMalloc;
use time::macros::format_description;
use time::PrimitiveDateTime;
use tokio::pin;
use tracing::info;

/// CLI application for X
#[derive(Parser)]
#[clap(
    name = "Aurelion Utils",
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
    info!("Starting Aurelion 🚀");

    let config = config::load();
    info!("Loaded configuration: {}", serde_json::to_string_pretty(&config)?);

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

            let req = TardisRequest {
                exchange,
                channel,
                instruments,
                start,
                end,
            };

            let tardis = TardisService::builder().config(&config.tardis).build();
            let stream = tardis.stream_parsed::<TardisModel>(req);
            pin!(stream);
            while let Some(data) = stream.next().await {
                info!("{:?}", data);
            }
        }
    }
    Ok(())
}
