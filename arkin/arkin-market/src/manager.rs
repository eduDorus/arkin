use arkin_common::prelude::*;
use time::OffsetDateTime;

use crate::MarketManagerConfig;

pub struct MarketManager {
    _lookback_min: u64,
}

impl MarketManager {
    pub fn from_config(config: &MarketManagerConfig) -> Self {
        Self {
            _lookback_min: config.lookback_min,
        }
    }

    pub fn snapshot(&self, _timestamp: &OffsetDateTime) -> Vec<Tick> {
        vec![]
    }
}
