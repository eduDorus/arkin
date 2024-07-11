use std::sync::Arc;

use rust_decimal::Decimal;

use crate::{config::VWAPConfig, models::Price, state::State};

use super::{Feature, FeatureEvent};

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
    state: Arc<State>,
    window: u64,
}

impl VWAPFeature {
    pub fn new(state: Arc<State>, config: &VWAPConfig) -> VWAPFeature {
        VWAPFeature {
            state,
            window: config.window,
        }
    }
}

impl Feature for VWAPFeature {
    async fn start(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;
            let vwap = VWAP::new(Price::new(Decimal::new(10, 0)).unwrap());
            self.state.feature_update(&FeatureEvent::VWAP(vwap))
        }
    }
}
