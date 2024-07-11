pub mod errors;
mod factory;
mod vwap;

use core::fmt;

pub use factory::FeatureFactory;
use flume::Sender;
use vwap::{VWAPFeature, VWAP};

#[trait_variant::make(Send)]
pub trait Feature: Clone {
    async fn start(&self, sender: Sender<FeatureEvent>);
}

#[derive(Clone)]
pub enum FeatureEvent {
    VWAP(VWAP),
    // SMA(SMA),
    // EMA(EMA),
}

impl fmt::Display for FeatureEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FeatureEvent::VWAP(vwap) => write!(f, "VWAP: {}", vwap.price),
            // FeatureEvent::SMA(sma) => write!(f, "SMA: {}", sma.price),
            // FeatureEvent::EMA(ema) => write!(f, "EMA: {}", ema.price),
        }
    }
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
    async fn start(&self, sender: Sender<FeatureEvent>) {
        match self {
            FeatureType::VWAP(f) => f.start(sender).await,
        }
    }
}
