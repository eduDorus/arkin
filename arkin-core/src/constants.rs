use std::sync::{Arc, LazyLock};

use time::{format_description::FormatItem, macros::format_description};

use super::FeatureId;

// Timestamp formats for the instrument and tracing
pub const INSTRUMENT_TIMESTAMP_FORMAT: &[FormatItem] = format_description!("[year][month][day]");
pub const TIMESTAMP_FORMAT: &[FormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

// Features
pub static TRADE_PRICE_FEATURE_ID: LazyLock<FeatureId> = LazyLock::new(|| Arc::new("trade_price".to_string()));
pub static TRADE_QUANTITY_FEATURE_ID: LazyLock<FeatureId> = LazyLock::new(|| Arc::new("trade_quantity".to_string()));
pub static TICK_BID_PRICE_FEATURE_ID: LazyLock<FeatureId> = LazyLock::new(|| Arc::new("tick_bid_price".to_string()));
pub static TICK_BID_QUANTITY_FEATURE_ID: LazyLock<FeatureId> =
    LazyLock::new(|| Arc::new("tick_bid_quantity".to_string()));
pub static TICK_ASK_PRICE_FEATURE_ID: LazyLock<FeatureId> = LazyLock::new(|| Arc::new("tick_ask_price".to_string()));
pub static TICK_ASK_QUANTITY_FEATURE_ID: LazyLock<FeatureId> =
    LazyLock::new(|| Arc::new("tick_ask_quantity".to_string()));

pub static RAW_FEATURE_IDS: LazyLock<Vec<FeatureId>> = LazyLock::new(|| {
    vec![
        TRADE_PRICE_FEATURE_ID.clone(),
        TRADE_QUANTITY_FEATURE_ID.clone(),
        TICK_BID_PRICE_FEATURE_ID.clone(),
        TICK_BID_QUANTITY_FEATURE_ID.clone(),
        TICK_ASK_PRICE_FEATURE_ID.clone(),
        TICK_ASK_QUANTITY_FEATURE_ID.clone(),
    ]
});
