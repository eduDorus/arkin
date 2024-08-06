use std::sync::LazyLock;

use time::{format_description::FormatItem, macros::format_description};

use crate::features::FeatureId;

// Timestamp formats for the instrument and tracing
pub const INSTRUMENT_TIMESTAMP_FORMAT: &[FormatItem] = format_description!("[year][month][day]");
pub const TIMESTAMP_FORMAT: &[FormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

// Features
pub static POSITIONS_ID: LazyLock<FeatureId> = LazyLock::new(|| FeatureId::from("positions"));
pub static TRADE_PRICE_ID: LazyLock<FeatureId> = LazyLock::new(|| FeatureId::from("trade_price"));
pub static TRADE_QUANTITY_ID: LazyLock<FeatureId> = LazyLock::new(|| FeatureId::from("trade_quantity"));
pub static FILL_PRICE_ID: LazyLock<FeatureId> = LazyLock::new(|| FeatureId::from("fill_price"));
pub static FILL_QUANTITY_ID: LazyLock<FeatureId> = LazyLock::new(|| FeatureId::from("fill_quantity"));
