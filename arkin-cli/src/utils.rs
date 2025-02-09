use clap::Parser;
use time::{macros::format_description, OffsetDateTime, PrimitiveDateTime};

use crate::commands::Cli;

pub fn parse_cli() -> Cli {
    Cli::parse()
}

/// Custom parser to convert string to OffsetDateTime
pub fn parse_datetime(s: &str) -> Result<OffsetDateTime, String> {
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]");
    let ts = PrimitiveDateTime::parse(&s, &format)
        .map_err(|e| format!("Failed to parse datetime '{}': {}", s, e))?
        .assume_utc();
    Ok(ts)
}
