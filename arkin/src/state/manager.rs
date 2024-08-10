use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use time::OffsetDateTime;

use crate::{
    config::StateManagerConfig,
    features::FeatureEvent,
    models::{Event, EventType, EventTypeOf, Instrument},
};

use super::{EventState, FeatureDataRequest, FeatureDataResponse, FeatureState, PortfolioState};

pub struct StateManager {
    features: FeatureState,
    events: EventState,
    portfolio: PortfolioState,
}

impl StateManager {
    pub fn from_config(config: &StateManagerConfig) -> Self {
        Self {
            features: FeatureState::default(),
            events: EventState::default(),
            portfolio: PortfolioState::from_config(&config.portfolio),
        }
    }
    pub fn add_event(&self, event: Event) {
        self.events.add_event(event);
    }

    pub fn add_feature(&self, event: FeatureEvent) {
        self.features.add_feature(event);
    }

    pub fn read_features(
        &self,
        instrument: &Instrument,
        timestamp: &OffsetDateTime,
        request: &[FeatureDataRequest],
    ) -> FeatureDataResponse {
        self.features.read_features(instrument, timestamp, request)
    }

    pub fn list_instruments(&self, event_type: &EventType) -> HashSet<Instrument> {
        self.events.list_instruments(event_type)
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
                let event = self.events.list_entries_since_start(&i, timestamp);
                (i, event)
            })
            .collect()
    }

    pub fn events_by_instrument<T>(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Vec<T>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        self.events.list_entries_since_start(instrument, timestamp)
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
                let event = self.events.last_entry(&i, timestamp);
                (i, event)
            })
            .collect()
    }

    pub fn latest_event_by_instrument<T>(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Option<T>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        self.events.last_entry(instrument, timestamp)
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
                let event = self.events.list_entries_window(&i, timestamp, window);
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
        self.events.list_entries_window(instrument, timestamp, window)
    }
}
