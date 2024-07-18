pub mod errors;
mod factory;
mod vwap;

use core::fmt;

pub use factory::FeatureFactory;
use vwap::VWAPFeature;
pub use vwap::VWAP;

#[trait_variant::make(Send)]
pub trait Feature: Clone {
    async fn start(&self);
}

#[derive(Clone)]
pub enum FeatureType {
    VWAP(VWAPFeature),
    // SMA(SMAFeature),
    // EMA(EMAFeature),
}

impl fmt::Display for FeatureType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FeatureType::VWAP(_) => write!(f, "VWAP"),
            // FeatureType::SMA(sma) => write!(f, "SMA: {}", sma.config),
            // FeatureType::EMA(ema) => write!(f, "EMA: {}", ema.config),
        }
    }
}

impl Feature for FeatureType {
    async fn start(&self) {
        match self {
            FeatureType::VWAP(f) => f.start().await,
        }
    }
}
