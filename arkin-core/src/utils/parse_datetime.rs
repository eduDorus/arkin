use time::{macros::format_description, PrimitiveDateTime, UtcDateTime};

/// Custom parser to convert string to UtcDateTime
pub fn parse_datetime(s: &str) -> Result<UtcDateTime, String> {
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]");
    let ts = PrimitiveDateTime::parse(&s, &format)
        .map_err(|e| format!("Failed to parse datetime '{}': {}", s, e))?
        .as_utc();
    Ok(ts)
}
