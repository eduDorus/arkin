use clap::{Args, Parser, Subcommand, ValueEnum};
use time::UtcDateTime;

use arkin_core::prelude::*;
use arkin_ingestors::prelude::*;

use crate::prelude::AccountingServiceType;

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
    /// Download data
    Download(DownloadArgs),

    /// Run a ingestor
    Ingestor(IngestorsArgs),

    /// Perform insights related operations
    Insights(InsightsArgs),

    /// Configure simulation ingestor
    Simulation(SimulationArgs),

    /// Perform engine related operations
    Live(LiveArgs),
}

#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// Instruments (comma-separated)
    #[arg(long, value_delimiter = ',', value_parser)]
    pub instruments: Vec<String>,

    /// Venue name
    #[arg(long, short)]
    pub venue: TardisExchange,

    /// Channel name
    #[arg(long, short)]
    pub channel: TardisChannel,

    /// Start datetime in "YYYY-MM-DD HH:MM" format
    #[arg(long, value_parser = parse_datetime)]
    pub start: UtcDateTime,

    /// End datetime in "YYYY-MM-DD HH:MM" format
    #[arg(long, value_parser = parse_datetime)]
    pub end: UtcDateTime,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum IngestorType {
    Binance,
}

#[derive(Args, Debug)]
pub struct IngestorsArgs {
    /// Ingestor type
    #[arg(long, short)]
    pub ingestor: IngestorType,

    /// Configure the instruments to subscribe to
    #[arg(long, value_delimiter = ',')]
    pub instruments: Vec<String>,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct InsightsArgs {
    /// Instruments (comma-separated)
    #[arg(long, short, value_delimiter = ',')]
    pub instruments: Vec<String>,

    /// Tick frequency in seconds
    #[arg(long, short)]
    pub tick_frequency: u64,

    /// Pipeline name
    #[arg(long, short)]
    pub pipeline: String,

    /// Start date in "YYYY-MM-DD HH:MM" format
    #[arg(long, short, value_parser = parse_datetime)]
    pub start: UtcDateTime,

    /// End date in "YYYY-MM-DD HH:MM" format
    #[arg(long, short, value_parser = parse_datetime)]
    pub end: UtcDateTime,

    /// Only normalized insights to save down
    #[arg(long)]
    pub only_normalized: bool,

    /// Only prediction insights to save down
    #[arg(long)]
    pub only_predictions: bool,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct SimulationArgs {
    /// Instance name
    #[arg(long, short = 'n')]
    pub instance_name: String,

    /// Instruments (comma-separated)
    #[arg(long, short, value_delimiter = ',', value_parser)]
    pub instruments: Vec<String>,

    /// Tick frequency in seconds
    #[arg(long, short)]
    pub tick_frequency: u64,

    /// Pipeline name
    #[arg(long, short)]
    pub pipeline: String,

    /// Start datetime in "YYYY-MM-DD HH:MM" format
    #[arg(long, short, value_parser = parse_datetime)]
    pub start: UtcDateTime,

    /// End datetime in "YYYY-MM-DD HH:MM" format
    #[arg(long, short, value_parser = parse_datetime)]
    pub end: UtcDateTime,

    /// Accounting
    #[arg(long, short, default_value = "ledger")]
    pub accounting_type: AccountingServiceType,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct LiveArgs {
    /// Instance name
    #[arg(long, short = 'n')]
    pub instance_name: String,

    /// Instruments (comma-separated)
    #[arg(long, value_delimiter = ',', value_parser)]
    pub instruments: Vec<String>,

    /// Ingestors
    #[arg(long, value_delimiter = ',')]
    pub ingestors: Vec<IngestorType>,
}
