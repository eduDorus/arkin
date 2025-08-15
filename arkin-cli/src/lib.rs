use clap::{Args, Parser, Subcommand, ValueEnum};
use rust_decimal::{dec, Decimal};
use time::UtcDateTime;

use arkin_core::prelude::*;

/// CLI application for Arkin.
///
/// This is the entry point for all commands related to data downloading, ingestion,
/// insights generation, simulations, and live operations. Use `--help` for details on subcommands.
#[derive(Parser, Debug)]
#[clap(
    name = "arkin",
    version = "0.1.0",
    about = "Welcome to the world of Arkin! A tool for data ingestion, insights, and simulations in financial contexts."
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

/// Available subcommands for Arkin operations.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Download historical market data from specified venues.
    ///
    /// This command fetches raw data for analysis or ingestion. Supports dry runs to simulate without actual downloads.
    Download(DownloadArgs),

    /// Run data ingestors to process and store incoming data streams.
    ///
    /// Ingestors subscribe to live or historical feeds. Currently supports Binance; extend `IngestorType` for more.
    Ingestor(IngestorsArgs),

    /// Generate and manage insights from processed data.
    ///
    /// Runs pipelines to compute normalized or predictive insights over a time range.
    Insights(InsightsArgs),

    /// Generate and manage scalers from processed data.
    ///
    /// Runs scaler initialization to compute normalization of insights.
    Scaler(ScalerArgs),

    /// Configure and run simulation ingestors for backtesting.
    ///
    /// Simulates trading scenarios with configurable accounting and tick frequencies.
    Simulation(SimulationArgs),

    /// Perform wide quoting operations, such as real-time processing.
    ///
    /// Connects to live ingestors for ongoing data handling.
    WideQuoter(WideQuoterArgs),

    /// Perform agent operations, such as real-time processing.
    ///
    /// Connects to live ingestors for ongoing data handling.
    Agent(AgentArgs),
}

/// Arguments for the `download` subcommand.
#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// Instruments to download data for (comma-separated, e.g., "BTCUSDT,ETHUSDT").
    #[arg(long, value_delimiter = ',', value_parser)]
    pub instruments: Vec<String>,

    /// Exchange venue to fetch data from (e.g., Binance, Coinbase).
    #[arg(long, short)]
    pub venue: Exchange,

    /// Data channel/type (e.g., trades, order books).
    #[arg(long, short)]
    pub channel: Channel,

    /// Start datetime in "YYYY-MM-DD HH:MM" UTC format.
    #[arg(long, value_parser = parse_datetime)]
    pub start: UtcDateTime,

    /// End datetime in "YYYY-MM-DD HH:MM" UTC format (exclusive).
    #[arg(long, value_parser = parse_datetime)]
    pub end: UtcDateTime,

    /// Perform a dry run: simulate download without saving files.
    #[arg(long)]
    pub dry_run: bool,
}

/// Supported ingestor types.
///
/// Extend this enum to add more exchange-specific ingestors.
#[derive(Debug, Clone, ValueEnum)]
pub enum IngestorType {
    /// Binance exchange ingestor.
    Binance,
}

/// Arguments for the `ingestor` subcommand.
#[derive(Args, Debug)]
pub struct IngestorsArgs {
    /// Type of ingestor to run.
    // #[arg(long, )]
    // pub ingestor: IngestorType,

    /// Instruments to subscribe to (comma-separated, e.g., "BTCUSDT,ETHUSDT").
    #[arg(long, short, value_delimiter = ',')]
    pub instruments: Vec<String>,

    /// Perform a dry run: simulate ingestion without processing data.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `insights` subcommand.
#[derive(Args, Debug)]
pub struct InsightsArgs {
    /// Instruments to generate insights for (comma-separated, e.g., "BTCUSDT,ETHUSDT").
    #[arg(long, short, value_delimiter = ',')]
    pub instruments: Vec<String>,

    /// Tick frequency for data aggregation, in seconds (e.g., 60 for 1-minute bars).
    #[arg(long, short)]
    pub tick_frequency: u64,

    /// Name of the insights pipeline to run (e.g., "volatility_analysis").
    #[arg(long, short)]
    pub pipeline: String,

    /// Start datetime in "YYYY-MM-DD HH:MM" UTC format.
    #[arg(long, short, value_parser = parse_datetime)]
    pub start: UtcDateTime,

    /// End datetime in "YYYY-MM-DD HH:MM" UTC format (exclusive).
    #[arg(long, short, value_parser = parse_datetime)]
    pub end: UtcDateTime,

    /// Pipeline warmup ticks
    #[arg(short, long, default_value_t = 1440)]
    pub warmup: u16,

    /// Save only normalized insights (e.g., scaled features).
    #[arg(long)]
    pub only_normalized: bool,

    /// Save only prediction insights (e.g., model outputs).
    #[arg(long)]
    pub only_predictions: bool,

    /// Perform a dry run: simulate insights generation without saving.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `scaler` subcommand.
#[derive(Args, Debug)]
pub struct ScalerArgs {
    /// Name of the insights pipeline to run (e.g., "volatility_analysis").
    #[arg(long, short)]
    pub pipeline: String,

    /// Instruments to generate insights for (comma-separated, e.g., "BTCUSDT,ETHUSDT").
    #[arg(long, short, value_delimiter = ',')]
    pub instruments: Vec<String>,

    /// Start datetime in "YYYY-MM-DD HH:MM" UTC format.
    #[arg(long, short, value_parser = parse_datetime)]
    pub start: UtcDateTime,

    /// End datetime in "YYYY-MM-DD HH:MM" UTC format (exclusive).
    #[arg(long, short, value_parser = parse_datetime)]
    pub end: UtcDateTime,

    /// Number of quantiles
    #[arg(short, long, default_value_t = 1000)]
    pub n_quantiles: u16,

    /// Perform a dry run: simulate insights generation without saving.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `simulation` subcommand.
#[derive(Args, Debug)]
pub struct SimulationArgs {
    /// Unique name for this simulation instance.
    #[arg(long, short = 'n')]
    pub instance_name: String,

    /// Instruments to simulate (comma-separated, e.g., "BTCUSDT,ETHUSDT").
    #[arg(long, short, value_delimiter = ',', value_parser)]
    pub instruments: Vec<String>,

    /// Tick frequency for simulation steps, in seconds.
    #[arg(long, short)]
    pub tick_frequency: u64,

    /// Name of the simulation pipeline (e.g., "backtest_strategy").
    #[arg(long, short)]
    pub pipeline: String,

    /// Start datetime in "YYYY-MM-DD HH:MM" UTC format.
    #[arg(long, short, value_parser = parse_datetime)]
    pub start: UtcDateTime,

    /// End datetime in "YYYY-MM-DD HH:MM" UTC format (exclusive).
    #[arg(long, short, value_parser = parse_datetime)]
    pub end: UtcDateTime,

    /// Pipeline warmup ticks
    #[arg(short, long, default_value_t = 1440)]
    pub warmup: u16,

    /// Perform a dry run: simulate without executing trades or saving results.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `wide-quoter` subcommand.
#[derive(Args, Debug)]
pub struct WideQuoterArgs {
    // /// Unique name for this live instance.
    // #[arg(long, short = 'n')]
    // pub instance_name: String,
    /// Instruments to process live (comma-separated, e.g., "BTCUSDT,ETHUSDT").
    #[arg(long, short = 'i', value_delimiter = ',', value_parser)]
    pub instruments: Vec<String>,

    /// Quote percentage from mid price
    #[arg(long, short, default_value_t = dec!(0.005))]
    pub quote_spread: Decimal,

    /// Requote threshold
    #[arg(long, short, default_value_t = dec!(0.0002))]
    pub requote_threshold: Decimal,
}

/// Arguments for the `wide-quoter` subcommand.
#[derive(Args, Debug)]
pub struct AgentArgs {
    /// Unique name for this live instance.
    #[arg(long, short = 'n')]
    pub instance_name: String,

    /// Instruments to process live (comma-separated, e.g., "BTCUSDT,ETHUSDT").
    #[arg(long, short = 'i', value_delimiter = ',', value_parser)]
    pub instruments: Vec<String>,

    /// Tick frequency for simulation steps, in seconds.
    #[arg(long, short)]
    pub tick_frequency: u64,

    /// Name of the simulation pipeline (e.g., "backtest_strategy").
    #[arg(long, short)]
    pub pipeline: String,

    /// Pipeline warmup ticks
    #[arg(short, long, default_value_t = 1440)]
    pub warmup: u16,
}
