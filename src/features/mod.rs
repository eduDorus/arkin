pub mod errors;
mod factory;
mod graph;
// mod vwap;

pub use graph::FeatureGraph;
use std::time::Duration;
use tracing::info;

pub trait Feature {
    fn id(&self) -> &str;
    fn sources(&self) -> Vec<&str>;
    fn calculate(&self);
}

pub struct VWAPGen {
    id: String,
    window: Duration,
}

impl VWAPGen {
    pub fn new(id: &str, window: Duration) -> Self {
        VWAPGen {
            id: id.to_string(),
            window,
        }
    }
}

impl Feature for VWAPGen {
    fn id(&self) -> &str {
        &self.id
    }

    fn sources(&self) -> Vec<&str> {
        vec![]
    }

    fn calculate(&self) {
        info!("Calculating VWAP with id: {}", self.id);
    }
}

pub struct SMAGen {
    id: String,
    source: String,
    window: Duration,
}

impl SMAGen {
    pub fn new(id: &str, source: &str, window: Duration) -> Self {
        SMAGen {
            id: id.to_string(),
            source: source.to_string(),
            window,
        }
    }
}

impl Feature for SMAGen {
    fn id(&self) -> &str {
        &self.id
    }

    fn sources(&self) -> Vec<&str> {
        vec![&self.source]
    }

    fn calculate(&self) {
        info!("Calculating SMA with id: {}", self.id);
    }
}

pub struct EMAGen {
    id: String,
    source: String,
    window: Duration,
}

impl EMAGen {
    pub fn new(id: &str, source: &str, window: Duration) -> Self {
        EMAGen {
            id: id.to_string(),
            source: source.to_string(),
            window,
        }
    }
}

impl Feature for EMAGen {
    fn id(&self) -> &str {
        &self.id
    }

    fn sources(&self) -> Vec<&str> {
        vec![&self.source]
    }

    fn calculate(&self) {
        info!("Calculating EMA with id: {}", self.id);
    }
}

pub struct SpreadGen {
    id: String,
    leg_one: String,
    leg_two: String,
}

impl SpreadGen {
    pub fn new(id: &str, leg_one: &str, leg_two: &str) -> Self {
        SpreadGen {
            id: id.to_string(),
            leg_one: leg_one.to_string(),
            leg_two: leg_two.to_string(),
        }
    }
}

impl Feature for SpreadGen {
    fn id(&self) -> &str {
        &self.id
    }

    fn sources(&self) -> Vec<&str> {
        vec![&self.leg_one, &self.leg_two]
    }

    fn calculate(&self) {
        info!("Calculating Spread with id: {}", self.id);
    }
}

pub struct VolumeGen {
    id: String,
    window: Duration,
}

impl VolumeGen {
    pub fn new(id: &str, window: Duration) -> Self {
        VolumeGen {
            id: id.to_string(),
            window,
        }
    }
}

impl Feature for VolumeGen {
    fn id(&self) -> &str {
        &self.id
    }

    fn sources(&self) -> Vec<&str> {
        vec![]
    }

    fn calculate(&self) {
        info!("Calculating Volume with id: {}", self.id);
    }
}
