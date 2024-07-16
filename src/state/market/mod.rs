use std::cmp::Ordering;

use crate::{
    features::FeatureEvent,
    models::{AccountEvent, Instrument, MarketEvent},
};
use scc::{ebr::Guard, TreeIndex};
use time::{Duration, OffsetDateTime};
use tracing::info;

#[derive(Default)]
#[allow(unused)]
pub struct StateData {
    market: TreeIndex<CompositeKey, MarketEvent>,
    account: TreeIndex<CompositeKey, AccountEvent>,
    features: TreeIndex<CompositeKey, FeatureEvent>,
}

// Composite key struct
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CompositeKey {
    timestamp: OffsetDateTime,
    index: u64,
}

impl CompositeKey {
    pub fn new(timestamp: &OffsetDateTime) -> Self {
        CompositeKey {
            timestamp: timestamp.to_owned(),
            index: 0,
        }
    }

    pub fn new_max(timestamp: &OffsetDateTime) -> Self {
        CompositeKey {
            timestamp: timestamp.to_owned(),
            index: u64::MAX,
        }
    }

    pub fn increment(&mut self) {
        self.index += 1;
    }
}

// Implement ordering for the CompositeKey
impl Ord for CompositeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp).then_with(|| self.index.cmp(&other.index))
    }
}

impl PartialOrd for CompositeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl StateData {
    pub async fn handle_market_event(&self, event: &MarketEvent) {
        let mut key = CompositeKey::new(&event.event_time());
        while (self.market.insert_async(key.clone(), event.to_owned()).await).is_err() {
            key.increment();
        }
    }

    pub async fn handle_account_event(&self, event: &AccountEvent) {
        let mut key = CompositeKey::new(&event.event_time());
        while (self.account.insert_async(key.clone(), event.to_owned()).await).is_err() {
            key.increment();
        }
    }

    pub async fn handle_feature_event(&self, event: &FeatureEvent) {
        let mut key = CompositeKey::new(event.event_time());
        while (self.features.insert_async(key.clone(), event.to_owned()).await).is_err() {
            key.increment();
        }
    }

    pub fn list_market(&self, instrument: &Instrument, from: &OffsetDateTime, window: &Duration) -> Vec<MarketEvent> {
        let from_key = CompositeKey::new_max(from);
        let end_time = *from - *window;
        let end_key = CompositeKey::new(&end_time);
        info!(
            "Getting trades for instrument: {} from: {} till: {}",
            instrument, from, end_time
        );
        let guard = Guard::new();
        self.market
            .range(end_key..=from_key, &guard)
            .map(|(_, trade)| trade)
            .filter(|e| matches!(e, MarketEvent::AggTrade(_)))
            .filter(|e| e.instrument() == *instrument)
            .cloned()
            .collect()
    }
}
