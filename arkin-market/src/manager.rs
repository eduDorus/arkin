use std::{collections::HashMap, time::Duration};

use arkin_core::prelude::*;
use time::OffsetDateTime;

use crate::{events::MarketState, MarketManagerConfig};

pub struct MarketManager {
    _lookback_min: u64,
    state: MarketState,
}

impl MarketManager {
    pub fn from_config(config: &MarketManagerConfig) -> Self {
        Self {
            _lookback_min: config.lookback_min,
            state: MarketState::default(),
        }
    }

    pub fn insert(&self, event: Event) {
        self.state.insert(event);
    }

    pub fn insert_batch(&self, events: Vec<Event>) {
        events.into_iter().for_each(|e| self.insert(e))
    }

    pub fn snapshot(&self, timestamp: OffsetDateTime, window: Duration) -> MarketSnapshot {
        let ticks = self
            .state
            .list_instruments::<Tick>()
            .into_iter()
            .map(|i| (i.clone(), self.state.list_entries_window::<Tick>(&i, timestamp, &window)))
            .collect::<HashMap<_, Vec<_>>>();
        let trades = self
            .state
            .list_instruments::<Trade>()
            .into_iter()
            .map(|i| (i.clone(), self.state.list_entries_window::<Trade>(&i, timestamp, &window)))
            .collect::<HashMap<_, Vec<_>>>();

        MarketSnapshot::new(timestamp.to_owned(), ticks, trades)
    }
}
