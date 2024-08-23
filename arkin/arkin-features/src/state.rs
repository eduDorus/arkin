use std::{
    collections::{BTreeMap, HashMap},
    time::Duration,
};

use arkin_common::prelude::*;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use time::OffsetDateTime;

use crate::config::{LatestInputConfig, PeriodInputConfig, WindowInputConfig};

#[derive(Default)]
pub struct FeatureState {
    features: DashMap<(Instrument, FeatureId), BTreeMap<CompositeIndex, Decimal>>,
}

impl FeatureState {
    pub fn add_feature(&self, event: Feature) {
        let key = (event.instrument, event.id);
        let mut composit_key = CompositeIndex::new(&event.event_time);

        let mut entry = self.features.entry(key).or_default();
        while entry.get(&composit_key).is_some() {
            composit_key.increment();
        }
        entry.insert(composit_key, event.value);
    }

    pub fn read_features(
        &self,
        instrument: &Instrument,
        timestamp: &OffsetDateTime,
        request: &[FeatureDataRequest],
    ) -> FeatureDataResponse {
        FeatureDataResponse::new(
            request
                .iter()
                .map(|r| {
                    let data = match &r {
                        FeatureDataRequest::Latest { feature_id } => self.last_entry(instrument, feature_id, timestamp),
                        FeatureDataRequest::Window { feature_id, window } => {
                            self.list_entries_window(instrument, feature_id, timestamp, window)
                        }
                        FeatureDataRequest::Period {
                            feature_id,
                            periods,
                        } => self.list_entries_periods(instrument, feature_id, timestamp, periods),
                    };
                    (r.feature_id().clone(), data)
                })
                .collect(),
        )
    }

    fn last_entry(&self, instrument: &Instrument, feature_id: &FeatureId, timestamp: &OffsetDateTime) -> Vec<Decimal> {
        let index = CompositeIndex::new_max(timestamp);

        if let Some(tree) = self.features.get(&(instrument.to_owned(), feature_id.to_owned())) {
            tree.value().range(..=index).rev().take(1).map(|(_, v)| *v).collect()
        } else {
            Vec::new()
        }
    }

    fn list_entries_window(
        &self,
        instrument: &Instrument,
        feature_id: &FeatureId,
        timestamp: &OffsetDateTime,
        window: &Duration,
    ) -> Vec<Decimal> {
        let index = CompositeIndex::new_max(timestamp);
        let end_index = CompositeIndex::new(&(*timestamp - *window));

        if let Some(tree) = self.features.get(&(instrument.to_owned(), feature_id.to_owned())) {
            tree.value().range(end_index..=index).map(|(_, v)| *v).collect()
        } else {
            Vec::new()
        }
    }

    fn list_entries_periods(
        &self,
        instrument: &Instrument,
        feature_id: &FeatureId,
        timestamp: &OffsetDateTime,
        periods: &usize,
    ) -> Vec<Decimal> {
        let index = CompositeIndex::new_max(timestamp);

        if let Some(tree) = self.features.get(&(instrument.to_owned(), feature_id.to_owned())) {
            let mut res = tree
                .value()
                .range(..=index)
                .rev()
                .take(*periods)
                .map(|(_, v)| *v)
                .collect::<Vec<_>>();
            res.reverse();
            res
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug)]
pub enum FeatureDataRequest {
    Latest {
        feature_id: FeatureId,
    },
    Window {
        feature_id: FeatureId,
        window: Duration,
    },
    Period {
        feature_id: FeatureId,
        periods: usize,
    },
}

impl From<LatestInputConfig> for FeatureDataRequest {
    fn from(v: LatestInputConfig) -> Self {
        FeatureDataRequest::Latest {
            feature_id: v.feature_id,
        }
    }
}

impl From<WindowInputConfig> for FeatureDataRequest {
    fn from(v: WindowInputConfig) -> Self {
        FeatureDataRequest::Window {
            feature_id: v.feature_id,
            window: Duration::from_secs(v.window),
        }
    }
}

impl From<PeriodInputConfig> for FeatureDataRequest {
    fn from(v: PeriodInputConfig) -> Self {
        FeatureDataRequest::Period {
            feature_id: v.feature_id,
            periods: v.periods,
        }
    }
}

impl FeatureDataRequest {
    pub fn feature_id(&self) -> &FeatureId {
        match self {
            FeatureDataRequest::Latest { feature_id } => feature_id,
            FeatureDataRequest::Window { feature_id, .. } => feature_id,
            FeatureDataRequest::Period { feature_id, .. } => feature_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FeatureDataResponse {
    data: HashMap<FeatureId, Vec<Decimal>>,
}

impl FeatureDataResponse {
    pub fn new(data: HashMap<FeatureId, Vec<Decimal>>) -> Self {
        FeatureDataResponse { data }
    }

    // Convenience method to get the last value for a feature ID
    pub fn last(&self, feature_id: &FeatureId) -> Option<Decimal> {
        self.data.get(feature_id).and_then(|values| values.last().cloned())
    }

    pub fn count(&self, feature_id: &FeatureId) -> Option<Decimal> {
        self.data.get(feature_id).map(|values| values.len().into())
    }

    // Convenience method to get the sum of values for a feature ID
    pub fn sum(&self, feature_id: &FeatureId) -> Option<Decimal> {
        self.data.get(feature_id).map(|values| values.iter().sum())
    }

    pub fn mean(&self, feature_id: &FeatureId) -> Option<Decimal> {
        self.data.get(feature_id).map(|values| {
            let sum = values.iter().sum::<Decimal>();
            sum / Decimal::from(values.len())
        })
    }

    pub fn get(&self, feature_id: &FeatureId) -> Vec<Decimal> {
        self.data.get(feature_id).unwrap_or(&vec![]).to_vec()
    }
}
