use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use time::OffsetDateTime;

use crate::{
    config::StateManagerConfig,
    models::{Event, EventTypeOf, FeatureEvent, Instrument, MarketSnapshot, Tick},
};

use super::{EventState, FeatureDataRequest, FeatureDataResponse, FeatureState};

pub struct StateManager {
    features: FeatureState,
    events: EventState,
}

impl StateManager {
    pub fn from_config(_config: &StateManagerConfig) -> Self {
        Self {
            features: FeatureState::default(),
            events: EventState::default(),
        }
    }

    pub fn add_event(&self, event: Event) {
        self.events.add_event(event);
    }

    pub fn add_feature(&self, event: FeatureEvent) {
        self.features.add_feature(event);
    }

    // Get the latest ticks for every instrument
    pub fn market_snapshot(&self, timestamp: &OffsetDateTime) -> MarketSnapshot {
        let instruments = self.events.list_instruments::<Tick>();
        let ticks = instruments
            .iter()
            .filter_map(|i| self.events.last_entry(i, timestamp))
            .collect::<Vec<_>>();
        MarketSnapshot::new(timestamp.to_owned(), ticks)
    }

    pub fn read_features(
        &self,
        instrument: &Instrument,
        timestamp: &OffsetDateTime,
        request: &[FeatureDataRequest],
    ) -> FeatureDataResponse {
        self.features.read_features(instrument, timestamp, request)
    }

    pub fn list_instruments<T>(&self) -> HashSet<Instrument>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        self.events.list_instruments::<T>()
    }

    pub fn events<T>(&self, timestamp: &OffsetDateTime) -> HashMap<Instrument, Vec<T>>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        let instruments = self.events.list_instruments::<T>();
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

    pub fn last_events<T>(&self, timestamp: &OffsetDateTime) -> HashMap<Instrument, Option<T>>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        let instruments = self.events.list_instruments::<T>();
        instruments
            .into_iter()
            .map(|i| {
                let event = self.events.last_entry(&i, timestamp);
                (i, event)
            })
            .collect()
    }

    pub fn last_event_by_instrument<T>(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Option<T>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        self.events.last_entry(instrument, timestamp)
    }

    pub fn events_window<T>(&self, timestamp: &OffsetDateTime, window: &Duration) -> HashMap<Instrument, Vec<T>>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        let instruments = self.events.list_instruments::<T>();
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
