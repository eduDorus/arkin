use crate::{
    constants::TIMESTAMP_FORMAT,
    features::{FeatureDataRequest, FeatureDataResponse, FeatureEvent, FeatureId},
    models::{Event, EventType, Instrument, Price},
    utils::CompositeKey,
};
use dashmap::DashMap;
use std::{
    collections::{BTreeMap, HashSet},
    time::Duration,
};
use time::OffsetDateTime;
use tracing::debug;

#[derive(Default)]
pub struct State {
    features: DashMap<(Instrument, FeatureId), BTreeMap<CompositeKey, f64>>,
    events: DashMap<(Instrument, EventType), BTreeMap<CompositeKey, Event>>,
}

impl State {
    pub fn add_event(&self, event: Event) {
        let key = (event.instrument().clone(), event.event_type().clone());
        let mut composit_key = CompositeKey::new(event.event_time());

        let mut entry = self.events.entry(key).or_default();
        while entry.get(&composit_key).is_some() {
            composit_key.increment();
        }
        entry.insert(composit_key, event);
    }

    pub fn list_instruments(&self) -> HashSet<Instrument> {
        self.events.iter().map(|k| k.key().0.clone()).collect()
    }

    pub fn latest_price(&self, instrument: &Instrument, from: &OffsetDateTime) -> Option<Price> {
        let from_key = CompositeKey::new_max(from);

        if let Some(tree) = self.events.get(&(instrument.to_owned(), EventType::Tick)) {
            tree.value()
                .range(..=from_key)
                .rev()
                .take(1)
                // Map into trade event and get the price
                .map(|(_, e)| match e {
                    Event::Tick(t) => Some(t.mid_price()),
                    _ => None,
                })
                .next()
                .flatten()
        } else {
            None
        }
    }

    pub fn list_events_since_beginning(
        &self,
        instrument: &Instrument,
        event_type: &EventType,
        event_time: &OffsetDateTime,
    ) -> Vec<Event> {
        debug!(
            "Getting {} for {} from: {}",
            event_type,
            instrument,
            event_time.format(TIMESTAMP_FORMAT).expect("Unable to format timestamp"),
        );

        let from_key = CompositeKey::new(event_time);

        if let Some(tree) = self.events.get(&(instrument.to_owned(), event_type.to_owned())) {
            tree.range(..from_key).map(|(_, e)| e).cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn list_events(
        &self,
        instrument: &Instrument,
        event_type: EventType,
        from: OffsetDateTime,
        window: Duration,
    ) -> Vec<Event> {
        let from_adjusted = from;
        let till = from - (window);

        debug!(
            "Getting {} for {} from: {} till: {}",
            event_type,
            instrument,
            from_adjusted.format(TIMESTAMP_FORMAT).expect("Unable to format timestamp"),
            till.format(TIMESTAMP_FORMAT).expect("Unable to format timestamp")
        );

        let from_key = CompositeKey::new_max(&from_adjusted);
        let end_key = CompositeKey::new(&till);

        if let Some(tree) = self.events.get(&(instrument.to_owned(), event_type)) {
            tree.range(end_key..=from_key).map(|(_, e)| e).cloned().collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    }

    pub fn add_feature(&self, event: FeatureEvent) {
        let key = (event.instrument, event.id);
        let mut composit_key = CompositeKey::new(&event.event_time);

        let mut entry = self.features.entry(key).or_default();
        while entry.get(&composit_key).is_some() {
            composit_key.increment();
        }
        entry.insert(composit_key, event.value);
    }

    pub fn read_features(
        &self,
        instrument: &Instrument,
        from: &OffsetDateTime,
        request: &[FeatureDataRequest],
    ) -> FeatureDataResponse {
        FeatureDataResponse::new(
            request
                .iter()
                .map(|r| {
                    let data = match &r {
                        FeatureDataRequest::Latest(r) => self.get_latest(instrument, from, &r.feature_id),
                        FeatureDataRequest::Window(r) => self.get_range(instrument, from, &r.feature_id, r.window),
                        FeatureDataRequest::Period(r) => self.get_periods(instrument, from, &r.feature_id, r.periods),
                    };
                    (r.feature_id().clone(), data)
                })
                .collect(),
        )
    }

    fn get_latest(&self, instrument: &Instrument, from: &OffsetDateTime, feature_id: &FeatureId) -> Vec<f64> {
        let key = (instrument.clone(), feature_id.clone());
        let from_key = CompositeKey::new_max(from);

        if let Some(tree) = self.features.get(&key) {
            tree.value()
                .range(..=from_key)
                .rev()
                .take(1)
                .map(|(_, v)| {
                    debug!("Found value");
                    *v
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn get_range(
        &self,
        instrument: &Instrument,
        from: &OffsetDateTime,
        feature_id: &FeatureId,
        window: u64,
    ) -> Vec<f64> {
        let key = (instrument.clone(), feature_id.clone());
        let from_key = CompositeKey::new(from);
        let end_key = CompositeKey::new(&(*from - Duration::from_secs(window)));

        debug!("Getting range for {} from {} to {}", feature_id, from_key, end_key);

        if let Some(tree) = self.features.get(&key) {
            tree.value()
                .range(end_key..=from_key)
                .map(|(_, v)| {
                    debug!("Found value");
                    *v
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn get_periods(
        &self,
        instrument: &Instrument,
        from: &OffsetDateTime,
        feature_id: &FeatureId,
        periods: usize,
    ) -> Vec<f64> {
        let key = (instrument.clone(), feature_id.clone());
        let from_key = CompositeKey::new_max(from);

        if let Some(tree) = self.features.get(&key) {
            let mut res = tree
                .value()
                .range(..from_key)
                .rev()
                .take(periods)
                .map(|(_, v)| {
                    debug!("Found value");
                    *v
                })
                .collect::<Vec<_>>();
            res.reverse();
            res
        } else {
            Vec::new()
        }
    }
}
