use time::{format_description::FormatItem, macros::format_description};

// Timestamp formats for the instrument and tracing
pub const INSTRUMENT_TIMESTAMP_FORMAT: &[FormatItem] = format_description!("[year][month][day]");
pub const TIMESTAMP_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
