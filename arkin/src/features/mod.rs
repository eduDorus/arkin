use crate::constants::TIMESTAMP_FORMAT;
use crate::models::Instrument;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use time::OffsetDateTime;

mod base;
mod factory;
mod performance;
mod risk;
mod ta;

use base::*;
use performance::*;
use ta::*;

pub use factory::FeatureFactory;

#[derive(Clone)]
pub struct FeatureEvent {
    pub id: FeatureId,
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub value: f64,
}

impl FeatureEvent {
    pub fn new(id: FeatureId, instrument: Instrument, event_time: OffsetDateTime, value: f64) -> Self {
        FeatureEvent {
            id,
            instrument,
            event_time,
            value,
        }
    }
}

impl fmt::Display for FeatureEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let event_time = self.event_time.format(TIMESTAMP_FORMAT).expect("Failed to format time");
        write!(f, "{} {} {} {}", event_time, self.instrument, self.id, self.value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(String);

impl From<&str> for NodeId {
    fn from(id: &str) -> Self {
        NodeId(id.to_lowercase())
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct FeatureId(String);

impl From<&str> for FeatureId {
    fn from(id: &str) -> Self {
        FeatureId(id.to_lowercase())
    }
}

impl fmt::Display for FeatureId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Feature: Debug + Send + Sync {
    fn id(&self) -> &NodeId;
    fn sources(&self) -> &[NodeId];
    fn data(&self) -> &[FeatureDataRequest];
    fn calculate(&self, data: FeatureDataResponse) -> Result<HashMap<FeatureId, f64>>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FeatureDataRequest {
    #[serde(rename = "latest")]
    Latest(Latest),
    #[serde(rename = "window")]
    Window(Window),
    #[serde(rename = "periods")]
    Period(Period),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Latest {
    pub from: NodeId,
    #[serde(rename = "feature")]
    pub feature_id: FeatureId,
}

impl From<Latest> for FeatureDataRequest {
    fn from(v: Latest) -> Self {
        FeatureDataRequest::Latest(v)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Window {
    pub from: NodeId,
    #[serde(rename = "feature")]
    pub feature_id: FeatureId,
    pub window: u64,
}

impl From<Window> for FeatureDataRequest {
    fn from(v: Window) -> Self {
        FeatureDataRequest::Window(v)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Period {
    pub from: NodeId,
    #[serde(rename = "feature")]
    pub feature_id: FeatureId,
    pub periods: usize,
}

impl From<Period> for FeatureDataRequest {
    fn from(v: Period) -> Self {
        FeatureDataRequest::Period(v)
    }
}

impl FeatureDataRequest {
    pub fn node_id(&self) -> &NodeId {
        match self {
            FeatureDataRequest::Latest(v) => &v.from,
            FeatureDataRequest::Window(v) => &v.from,
            FeatureDataRequest::Period(v) => &v.from,
        }
    }

    pub fn feature_id(&self) -> &FeatureId {
        match self {
            FeatureDataRequest::Latest(v) => &v.feature_id,
            FeatureDataRequest::Window(v) => &v.feature_id,
            FeatureDataRequest::Period(v) => &v.feature_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FeatureDataResponse {
    data: HashMap<FeatureId, Vec<f64>>,
}

impl FeatureDataResponse {
    pub fn new(data: HashMap<FeatureId, Vec<f64>>) -> Self {
        FeatureDataResponse { data }
    }

    // Convenience method to get the last value for a feature ID
    pub fn last(&self, feature_id: &FeatureId) -> Option<f64> {
        self.data.get(feature_id).and_then(|values| values.last().cloned())
    }

    pub fn count(&self, feature_id: &FeatureId) -> Option<f64> {
        self.data.get(feature_id).map(|values| values.len() as f64)
    }

    // Convenience method to get the sum of values for a feature ID
    pub fn sum(&self, feature_id: &FeatureId) -> Option<f64> {
        self.data.get(feature_id).map(|values| values.iter().sum())
    }

    pub fn mean(&self, feature_id: &FeatureId) -> Option<f64> {
        self.data.get(feature_id).map(|values| {
            let sum: f64 = values.iter().sum();
            sum / values.len() as f64
        })
    }

    pub fn max(&self, feature_id: &FeatureId) -> Option<f64> {
        self.data
            .get(feature_id)
            .map(|values| values.iter().fold(f64::MIN, |acc, &v| acc.max(v)))
    }

    pub fn min(&self, feature_id: &FeatureId) -> Option<f64> {
        self.data
            .get(feature_id)
            .map(|values| values.iter().fold(f64::MAX, |acc, &v| acc.min(v)))
    }

    pub fn get(&self, feature_id: &FeatureId) -> Vec<f64> {
        self.data.get(feature_id).unwrap_or(&vec![]).to_vec()
    }
}
