use clap::{Args, Subcommand};
use time::OffsetDateTime;

use arkin_core::prelude::*;

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
