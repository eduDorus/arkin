pub mod errors;
mod factory;
mod graph;
// mod vwap;

use async_trait::async_trait;
use core::fmt;
pub use graph::FeatureGraph;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
pub trait Feature {
    fn id(&self) -> &FeatureID;
    fn sources(&self) -> Vec<&FeatureID>;
    async fn calculate(&self);
}

pub struct VWAPGen {
    id: FeatureID,
    window: Duration,
}

impl VWAPGen {
    pub fn new(id: FeatureID, window: Duration) -> Self {
        VWAPGen { id, window }
    }
}

#[async_trait]
impl Feature for VWAPGen {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![]
    }

    async fn calculate(&self) {
        info!("Calculating VWAP with id: {}", self.id);
    }
}

pub struct SMAGen {
    id: FeatureID,
    source: FeatureID,
    window: Duration,
}

impl SMAGen {
    pub fn new(id: FeatureID, source: FeatureID, window: Duration) -> Self {
        SMAGen { id, source, window }
    }
}

#[async_trait]
impl Feature for SMAGen {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![&self.source]
    }

    async fn calculate(&self) {
        info!("Calculating SMA with id: {}", self.id);
    }
}

pub struct EMAGen {
    id: FeatureID,
    source: FeatureID,
    window: Duration,
}

impl EMAGen {
    pub fn new(id: FeatureID, source: FeatureID, window: Duration) -> Self {
        EMAGen { id, source, window }
    }
}

#[async_trait]
impl Feature for EMAGen {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![&self.source]
    }

    async fn calculate(&self) {
        info!("Calculating EMA with id: {}", self.id);
    }
}

pub struct SpreadGen {
    id: FeatureID,
    leg_one: FeatureID,
    leg_two: FeatureID,
}

impl SpreadGen {
    pub fn new(id: FeatureID, leg_one: FeatureID, leg_two: FeatureID) -> Self {
        SpreadGen {
            id,
            leg_one,
            leg_two,
        }
    }
}

#[async_trait]
impl Feature for SpreadGen {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![&self.leg_one, &self.leg_two]
    }

    async fn calculate(&self) {
        info!("Calculating Spread with id: {}", self.id);
    }
}

pub struct VolumeGen {
    id: FeatureID,
    window: Duration,
}

impl VolumeGen {
    pub fn new(id: FeatureID, window: Duration) -> Self {
        VolumeGen { id, window }
    }
}

#[async_trait]
impl Feature for VolumeGen {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![]
    }

    async fn calculate(&self) {
        info!("Calculating Volume with id: {}", self.id);
    }
}
