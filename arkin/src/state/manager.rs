use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use time::OffsetDateTime;

use crate::{
    features::FeatureEvent,
    models::{Event, EventType, EventTypeOf, Instrument},
};

use super::{EventState, FeatureDataRequest, FeatureDataResponse, FeatureState};

#[derive(Default)]
pub struct StateManager {
    feature_state: FeatureState,
    event_state: EventState,
}

impl StateManager {
    pub fn add_event(&self, event: Event) {
        self.event_state.add_event(event);
    }

    pub fn add_feature(&self, event: FeatureEvent) {
        self.feature_state.add_feature(event);
    }

    pub fn read_features(
        &self,
        instrument: &Instrument,
        timestamp: &OffsetDateTime,
        request: &[FeatureDataRequest],
    ) -> FeatureDataResponse {
        self.feature_state.read_features(instrument, timestamp, request)
    }

    pub fn list_instruments(&self, event_type: &EventType) -> HashSet<Instrument> {
        self.event_state.list_instruments(event_type)
    }

    pub fn events<T>(&self, timestamp: &OffsetDateTime) -> HashMap<Instrument, Vec<T>>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        let event_type = T::event_type();
        let instruments = self.list_instruments(&event_type);
        instruments
            .into_iter()
            .map(|i| {
                let event = self.event_state.list_entries_since_start(&i, timestamp);
                (i, event)
            })
            .collect()
    }

    pub fn events_by_instrument<T>(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Vec<T>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        self.event_state.list_entries_since_start(instrument, timestamp)
    }

    pub fn latest_events<T>(&self, timestamp: &OffsetDateTime) -> HashMap<Instrument, Option<T>>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        let event_type = T::event_type();
        let instruments = self.list_instruments(&event_type);
        instruments
            .into_iter()
            .map(|i| {
                let event = self.event_state.last_entry(&i, timestamp);
                (i, event)
            })
            .collect()
    }

    pub fn latest_event_by_instrument<T>(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Option<T>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        self.event_state.last_entry(instrument, timestamp)
    }

    pub fn events_window<T>(&self, timestamp: &OffsetDateTime, window: &Duration) -> HashMap<Instrument, Vec<T>>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        let event_type = T::event_type();
        let instruments = self.list_instruments(&event_type);
        instruments
            .into_iter()
            .map(|i| {
                let event = self.event_state.list_entries_window(&i, timestamp, window);
                (i, event)
            })
            .collect()
    }

    pub fn events_window_by_instrument<T>(
        &self,
        instrument: &Instrument,
        timestamp: &OffsetDateTime,
        window: &Duration,
    ) -> Vec<T>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        self.event_state.list_entries_window(instrument, timestamp, window)
    }
}
