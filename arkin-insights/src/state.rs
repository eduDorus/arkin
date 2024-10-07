use std::{
    collections::{BTreeMap, HashSet},
    time::Duration,
};

use arkin_core::prelude::*;
use dashmap::DashMap;
use rayon::prelude::*;
use rust_decimal::prelude::*;
use time::OffsetDateTime;

// use crate::config::{LatestInputConfig, PeriodInputConfig, WindowInputConfig};

#[derive(Default)]
pub struct InsightsState {
    features: DashMap<(Option<Instrument>, FeatureId), BTreeMap<CompositeIndex, Decimal>>,
}

impl InsightsState {
    pub fn insert(&self, event: Insight) {
        let key = (event.instrument().clone(), event.id().clone());
        let mut composit_key = CompositeIndex::new(&event.event_time());

        let mut entry = self.features.entry(key).or_default();
        while entry.get(&composit_key).is_some() {
            composit_key.increment();
        }
        entry.insert(composit_key, event.value().to_owned());
    }

    pub fn insert_batch(&self, events: Vec<Insight>) {
        events.par_iter().for_each(|event| {
            let key = (event.instrument().clone(), event.id().clone());
            let mut composit_key = CompositeIndex::new(&event.event_time());

            let mut entry = self.features.entry(key).or_default();
            while entry.get(&composit_key).is_some() {
                composit_key.increment();
            }
            entry.insert(composit_key, event.value().to_owned());
        });
    }

    // pub fn read_features(
    //     &self,
    //     instrument: &Instrument,
    //     timestamp: &OffsetDateTime,
    //     request: &[DataRequest],
    // ) -> DataResponse {
    //     DataResponse::new(
    //         request
    //             .iter()
    //             .map(|r| {
    //                 let data = match &r {
    //                     DataRequest::Latest { feature_id } => self.last_entry(instrument, feature_id, timestamp),
    //                     DataRequest::Window { feature_id, window } => {
    //                         self.list_entries_window(instrument, feature_id, timestamp, window)
    //                     }
    //                     DataRequest::Period {
    //                         feature_id,
    //                         periods,
    //                     } => self.list_entries_periods(instrument, feature_id, timestamp, periods),
    //                 };
    //                 (r.feature_id().clone(), data)
    //             })
    //             .collect(),
    //     )
    // }

    pub fn instruments(&self) -> HashSet<Instrument> {
        self.features.iter().filter_map(|v| v.key().0.clone()).collect::<HashSet<_>>()
    }

    pub fn last_entry(
        &self,
        instrument: Option<&Instrument>,
        feature_id: &FeatureId,
        timestamp: &OffsetDateTime,
    ) -> Option<Decimal> {
        let index = CompositeIndex::new_max(timestamp);

        if let Some(tree) = self.features.get(&(instrument.cloned(), feature_id.to_owned())) {
            if let Some((_, v)) = tree.range(..=index).rev().take(1).next() {
                return Some(v.clone());
            }
        }
        None
    }

    pub fn list_entries_window(
        &self,
        instrument: Option<&Instrument>,
        feature_id: &FeatureId,
        timestamp: &OffsetDateTime,
        window: &Duration,
    ) -> Vec<Decimal> {
        let index = CompositeIndex::new_max(timestamp);
        let end_index = CompositeIndex::new(&(*timestamp - *window));

        if let Some(tree) = self.features.get(&(instrument.cloned(), feature_id.to_owned())) {
            tree.range(end_index..=index).map(|(_, v)| *v).collect()
        } else {
            Vec::new()
        }
    }

    pub fn list_entries_periods(
        &self,
        instrument: Option<&Instrument>,
        feature_id: &FeatureId,
        timestamp: &OffsetDateTime,
        periods: &usize,
    ) -> Vec<Decimal> {
        let index = CompositeIndex::new_max(timestamp);

        if let Some(tree) = self.features.get(&(instrument.cloned(), feature_id.to_owned())) {
            let mut res = tree.range(..=index).rev().take(*periods).map(|(_, v)| *v).collect::<Vec<_>>();
            res.reverse();
            res
        } else {
            Vec::new()
        }
    }
}

// #[derive(Debug)]
// pub enum DataRequest {
//     Latest {
//         feature_id: FeatureId,
//     },
//     Window {
//         feature_id: FeatureId,
//         window: Duration,
//     },
//     Period {
//         feature_id: FeatureId,
//         periods: usize,
//     },
// }

// impl From<LatestInputConfig> for DataRequest {
//     fn from(v: LatestInputConfig) -> Self {
//         DataRequest::Latest {
//             feature_id: v.feature_id,
//         }
//     }
// }

// impl From<WindowInputConfig> for DataRequest {
//     fn from(v: WindowInputConfig) -> Self {
//         DataRequest::Window {
//             feature_id: v.feature_id,
//             window: Duration::from_secs(v.window),
//         }
//     }
// }

// impl From<PeriodInputConfig> for DataRequest {
//     fn from(v: PeriodInputConfig) -> Self {
//         DataRequest::Period {
//             feature_id: v.feature_id,
//             periods: v.periods,
//         }
//     }
// }

// impl DataRequest {
//     pub fn feature_id(&self) -> &FeatureId {
//         match self {
//             DataRequest::Latest { feature_id } => feature_id,
//             DataRequest::Window { feature_id, .. } => feature_id,
//             DataRequest::Period { feature_id, .. } => feature_id,
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct DataResponse {
//     data: HashMap<FeatureId, Vec<Decimal>>,
// }

// impl DataResponse {
//     pub fn new(data: HashMap<FeatureId, Vec<Decimal>>) -> Self {
//         DataResponse { data }
//     }

//     // Convenience method to get the last value for a feature ID
//     pub fn last(&self, feature_id: &FeatureId) -> Option<Decimal> {
//         self.data.get(feature_id).and_then(|values| values.last().cloned())
//     }

//     pub fn count(&self, feature_id: &FeatureId) -> Option<Decimal> {
//         self.data.get(feature_id).map(|values| values.len().into())
//     }

//     // Convenience method to get the sum of values for a feature ID
//     pub fn sum(&self, feature_id: &FeatureId) -> Option<Decimal> {
//         self.data.get(feature_id).map(|values| values.iter().sum())
//     }

//     pub fn mean(&self, feature_id: &FeatureId) -> Option<Decimal> {
//         self.data.get(feature_id).map(|values| {
//             let sum = values.iter().sum::<Decimal>();
//             sum / Decimal::from(values.len())
//         })
//     }

//     pub fn get(&self, feature_id: &FeatureId) -> Vec<Decimal> {
//         self.data.get(feature_id).unwrap_or(&vec![]).to_vec()
//     }
// }
