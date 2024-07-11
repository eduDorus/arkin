use flume::Sender;
use rust_decimal::Decimal;

use crate::{config::VWAPConfig, models::Price};

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
    pub config: VWAPConfig,
}

impl VWAPFeature {
    pub fn new(config: &VWAPConfig) -> VWAPFeature {
        VWAPFeature {
            config: config.to_owned(),
        }
    }
}

impl Feature for VWAPFeature {
    async fn start(&self, sender: Sender<FeatureEvent>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;
            let vwap = VWAP::new(Price::new(Decimal::new(10, 0)).unwrap());
            sender.send_async(FeatureEvent::VWAP(vwap)).await.unwrap();
        }
    }
}
