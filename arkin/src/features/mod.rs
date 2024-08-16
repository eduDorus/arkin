use crate::models::{FeatureId, NodeId};
use crate::state::{FeatureDataRequest, FeatureDataResponse};
use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Debug;

mod base;
mod factory;
mod manager;
mod pipeline;
mod risk;
mod ta;

use base::*;
use factory::FeatureFactory;
use pipeline::*;
use ta::*;

pub use manager::FeatureManager;

pub trait Feature: Debug + Send + Sync {
    fn id(&self) -> &NodeId;
    fn sources(&self) -> &[NodeId];
    fn data(&self) -> &[FeatureDataRequest];
    fn calculate(&self, data: FeatureDataResponse) -> Result<HashMap<FeatureId, f64>>;
}
