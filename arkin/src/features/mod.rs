use crate::constants::TIMESTAMP_FORMAT;
use crate::models::Instrument;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::time::Duration;
use time::OffsetDateTime;

pub mod errors;
mod factory;
mod sma;
mod spread;
mod volume;
mod vwap;

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
pub struct FeatureId(String);

impl From<&str> for FeatureId {
    fn from(id: &str) -> Self {
        FeatureId(id.to_lowercase())
    }
}

impl From<String> for FeatureId {
    fn from(id: String) -> Self {
        FeatureId(id.to_lowercase())
    }
}

impl fmt::Display for FeatureId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Feature: Debug + Send + Sync {
    fn id(&self) -> &FeatureId;
    fn sources(&self) -> &[FeatureId];
    fn data_type(&self) -> &QueryType;
    fn calculate(&self, data: HashMap<FeatureId, Vec<f64>>) -> Result<HashMap<FeatureId, f64>>;
}

#[derive(Debug)]
pub enum QueryType {
    Latest,
    Window(Duration),
    Period(usize),
}
