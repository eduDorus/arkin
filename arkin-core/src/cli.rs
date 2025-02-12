use clap::{Args, Parser, Subcommand};
use time::OffsetDateTime;

use crate::utils::parse_datetime;

/// CLI application for X
#[derive(Parser)]
#[clap(
    name = "arkin",
    version = "0.1.0",
    about = "Welcome to the world of arkin!"
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a ingestor
    #[clap(subcommand)]
    Ingestor(IngestorsCommands),

    /// Perform insights related operations
    Insights(InsightsArgs),

    /// Configure simulation ingestor
    Simulation(SimulationArgs),

    /// Perform engine related operations
    Live(LiveArgs),
}

#[derive(Subcommand, Debug)]
pub enum IngestorsCommands {
    /// Configure and start Binance ingestor
    Binance(BinanceIngestorArgs),

    /// Configure and start Tardis ingestor
    Tardis(TardisIngestorArgs),
}

#[derive(Args, Debug)]
pub struct BinanceIngestorArgs {
    /// Configure the channels to subscribe to
    #[arg(long, short, value_delimiter = ',')]
    pub channels: Vec<String>,

    /// Configure the instruments to subscribe to
    #[arg(long, short, value_delimiter = ',')]
    pub instruments: Vec<String>,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct TardisIngestorArgs {
    /// Venue name
    #[arg(long)]
    pub venue: String,

    /// Channel name
    #[arg(long)]
    pub channel: String,

    /// Instruments (comma-separated)
    #[arg(long, value_delimiter = ',', value_parser)]
    pub instruments: Vec<String>,

    /// Start datetime in "YYYY-MM-DD HH:MM" format
    #[arg(long, value_parser = parse_datetime)]
    pub start: OffsetDateTime,

    /// End datetime in "YYYY-MM-DD HH:MM" format
    #[arg(long, value_parser = parse_datetime)]
    pub end: OffsetDateTime,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct InsightsArgs {
    /// Instruments (comma-separated)
    #[arg(long, short, value_delimiter = ',')]
    pub instruments: Vec<String>,

    /// Pipeline name
    #[arg(long, short)]
    pub pipeline: String,

    /// Start date in "YYYY-MM-DD HH:MM" format
    #[arg(long, short, value_parser = parse_datetime)]
    pub start: OffsetDateTime,

    /// End date in "YYYY-MM-DD HH:MM" format
    #[arg(long, short, value_parser = parse_datetime)]
    pub end: OffsetDateTime,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct SimulationArgs {
    /// Instruments (comma-separated)
    #[arg(long, value_delimiter = ',', value_parser)]
    pub instruments: Vec<String>,

    /// Pipeline name
    #[arg(long)]
    pub pipeline: String,

    /// Start datetime in "YYYY-MM-DD HH:MM" format
    #[arg(long, value_parser = parse_datetime)]
    pub start: OffsetDateTime,

    /// End datetime in "YYYY-MM-DD HH:MM" format
    #[arg(long, value_parser = parse_datetime)]
    pub end: OffsetDateTime,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct LiveArgs {
    /// Instruments (comma-separated)
    #[arg(long, value_delimiter = ',', value_parser)]
    pub instruments: Vec<String>,
}
