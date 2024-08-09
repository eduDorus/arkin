use crate::constants::TIMESTAMP_FORMAT;
use crate::models::Instrument;
use crate::state::{FeatureDataRequest, FeatureDataResponse};
use anyhow::Result;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use time::OffsetDateTime;

mod base;
mod factory;
mod risk;
mod ta;

use base::*;
use ta::*;

pub use factory::FeatureFactory;

pub type NodeId = String;
pub type FeatureId = String;

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

pub trait Feature: Debug + Send + Sync {
    fn id(&self) -> &NodeId;
    fn sources(&self) -> &[NodeId];
    fn data(&self) -> &[FeatureDataRequest];
    fn calculate(&self, data: FeatureDataResponse) -> Result<HashMap<FeatureId, f64>>;
}
