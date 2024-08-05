use crate::{
    constants::TIMESTAMP_FORMAT,
    features::{FeatureEvent, FeatureId, QueryType},
    models::{Event, EventType, Instrument},
    utils::CompositeKey,
};
use dashmap::DashMap;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
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

    pub fn list_events(
        &self,
        instrument: &Instrument,
        event_type: EventType,
        from: OffsetDateTime,
        window: Duration,
    ) -> Vec<Event> {
        let from_adjusted = from - Duration::from_nanos(1);
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

    pub fn query(
        &self,
        instrument: &Instrument,
        feature_ids: &[FeatureId],
        from: &OffsetDateTime,
        data_type: &QueryType,
    ) -> HashMap<FeatureId, Vec<f64>> {
        feature_ids
            .iter()
            .map(|feature_id| {
                let data = match data_type {
                    QueryType::Latest => self.get_latest(instrument, feature_id, from),
                    QueryType::Window(window) => self.get_range(instrument, feature_id, from, window),
                    QueryType::Period(periods) => self.get_periods(instrument, feature_id, from, *periods),
                };
                (feature_id.clone(), data)
            })
            .collect()
    }

    fn get_latest(&self, instrument: &Instrument, feature_id: &FeatureId, from: &OffsetDateTime) -> Vec<f64> {
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
        feature_id: &FeatureId,
        from: &OffsetDateTime,
        window: &Duration,
    ) -> Vec<f64> {
        let key = (instrument.clone(), feature_id.clone());
        let from_key = CompositeKey::new(from);
        let end_key = CompositeKey::new(&(*from - *window));

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
        feature_id: &FeatureId,
        from: &OffsetDateTime,
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
