use std::{collections::HashSet, time::Duration};

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

    pub fn latest_event<T>(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Option<T>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        self.event_state.last_entry(instrument, timestamp)
    }

    pub fn list_events_since_beginning<T>(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Vec<T>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        self.event_state.list_entries_since_start(instrument, timestamp)
    }

    pub fn list_events_window<T>(
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
