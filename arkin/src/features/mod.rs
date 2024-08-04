use anyhow::Result;
use async_trait::async_trait;
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

use crate::constants::TIMESTAMP_FORMAT;
use crate::models::Instrument;

#[derive(Clone)]
pub struct FeatureEvent {
    pub id: FeatureID,
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub value: f64,
}

impl FeatureEvent {
    pub fn new(id: FeatureID, instrument: Instrument, event_time: OffsetDateTime, value: f64) -> Self {
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
pub struct FeatureID(String);

impl From<&str> for FeatureID {
    fn from(id: &str) -> Self {
        FeatureID(id.to_lowercase())
    }
}

impl From<String> for FeatureID {
    fn from(id: String) -> Self {
        FeatureID(id.to_lowercase())
    }
}

impl fmt::Display for FeatureID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub trait Feature: Debug + Send + Sync {
    fn id(&self) -> &FeatureID;
    fn sources(&self) -> Vec<FeatureID>;
    fn data_type(&self) -> DataType;
    fn calculate(&self, data: HashMap<FeatureID, Vec<f64>>) -> Result<HashMap<FeatureID, f64>>;
}

#[derive(Debug)]
pub enum DataType {
    Latest,
    Window(Duration),
    Period(usize),
}
