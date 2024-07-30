use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClockConfig {
    pub tick_frequency: u64,
}
