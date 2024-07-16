use super::Feature;
use crate::{config::VWAPConfig, models::Price, state::StateManager};
use std::{sync::Arc, time::Duration};
use tracing::info;

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct VWAP {
    pub price: Price,
}

impl VWAP {
    pub fn new(price: Price) -> VWAP {
        VWAP { price }
    }
}

#[derive(Clone)]
pub struct VWAPFeature {
    state: Arc<StateManager>,
    window: Duration,
}

impl VWAPFeature {
    pub fn new(state: Arc<StateManager>, config: &VWAPConfig) -> VWAPFeature {
        let window = Duration::from_secs(config.window);
        VWAPFeature { state, window }
    }
}

impl Feature for VWAPFeature {
    async fn start(&self) {
        info!("Starting VWAP feature...");

        let mut rx = self.state.listen_feature_frequency(Duration::from_secs(5));

        while let Ok(_) = rx.recv().await {
            info!("VWAPFeature new tick...");
        }
    }
}
