#![allow(dead_code)]
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use rayon::prelude::*;
use rust_decimal::prelude::*;
use time::OffsetDateTime;

use arkin_core::prelude::*;
use tracing::debug;
use typed_builder::TypedBuilder;
use yata::core::Candle;

#[derive(Debug, Default, TypedBuilder)]
pub struct InsightsState {
    features: DashMap<(Option<Arc<Instrument>>, FeatureId), BTreeMap<i64, Decimal>>,
}

impl InsightsState {
    pub fn insert(&self, event: Arc<Insight>) {
        let start = Instant::now();
        let key = (event.instrument.clone(), event.feature_id.clone());
        // let mut composit_key = CompositeIndex::new(event.event_time);

        let mut entry = self.features.entry(key).or_default();
        // while entry.get(&composit_key).is_some() {
        //     composit_key.increment();
        // }
        entry.insert(event.event_time.unix_timestamp(), event.value);
        debug!("Insert into insight state took {:?}", start.elapsed());
    }

    pub fn insert_batch(&self, events: &[Arc<Insight>]) {
        let start = Instant::now();
        events.into_par_iter().for_each(|event| {
            self.insert(event.clone());
        });
        debug!("Insert batch into insight state took {:?}", start.elapsed());
    }

    pub fn remove(&self, event_time: OffsetDateTime) {
        let start = Instant::now();
        self.features.retain(|_, v| {
            v.retain(|i, _| OffsetDateTime::from_unix_timestamp(*i).expect("") >= event_time);
            !v.is_empty()
        });
        debug!("Remove from insight state took {:?}", start.elapsed());
    }

    pub fn last_candle(&self, instrument: Arc<Instrument>, timestamp: OffsetDateTime) -> Option<Candle> {
        let start = Instant::now();
        let open = self
            .last(Some(instrument.clone()), FeatureId::new("open".into()), timestamp)?
            .to_f64()?;
        let high = self
            .last(Some(instrument.clone()), FeatureId::new("high".into()), timestamp)?
            .to_f64()?;
        let low = self
            .last(Some(instrument.clone()), FeatureId::new("low".into()), timestamp)?
            .to_f64()?;
        let close = self
            .last(Some(instrument.clone()), FeatureId::new("close".into()), timestamp)?
            .to_f64()?;
        let volume = self
            .last(Some(instrument.clone()), FeatureId::new("volume".into()), timestamp)?
            .to_f64()?;

        debug!("Last candle took {:?}", start.elapsed());
        Some(Candle {
            open,
            high,
            low,
            close,
            volume,
        })
    }

    pub fn last(
        &self,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        timestamp: OffsetDateTime,
    ) -> Option<Decimal> {
        let start = Instant::now();
        if let Some(tree) = self.features.get(&(instrument, feature_id)) {
            if let Some((_, v)) = tree.range(..=timestamp.unix_timestamp()).rev().take(1).next() {
                debug!("Last took {:?}", start.elapsed());
                return Some(v.clone());
            }
        }
        debug!("Last took {:?}", start.elapsed());
        None
    }

    pub fn window(
        &self,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        timestamp: OffsetDateTime,
        window: Duration,
    ) -> Vec<Decimal> {
        let end_time = timestamp - window;

        if let Some(tree) = self.features.get(&(instrument, feature_id)) {
            tree.range(end_time.unix_timestamp()..=timestamp.unix_timestamp())
                .map(|(_, v)| *v)
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn periods(
        &self,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        timestamp: OffsetDateTime,
        periods: usize,
    ) -> Vec<Decimal> {
        if let Some(tree) = self.features.get(&(instrument, feature_id)) {
            let mut res = tree
                .range(..=timestamp.unix_timestamp())
                .rev()
                .take(periods)
                .map(|(_, v)| *v)
                .collect::<Vec<_>>();
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
