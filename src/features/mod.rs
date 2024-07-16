pub mod errors;
mod factory;
mod vwap;

use core::fmt;

pub use factory::FeatureFactory;
use time::OffsetDateTime;
use vwap::{VWAPFeature, VWAP};

use crate::models::Instrument;

#[trait_variant::make(Send)]
pub trait Feature: Clone {
    async fn start(&self);
}

#[derive(Clone)]
pub enum FeatureEvent {
    VWAP(VWAP),
    // SMA(SMA),
    // EMA(EMA),
}

impl FeatureEvent {
    pub fn instrument(&self) -> &Instrument {
        match self {
            FeatureEvent::VWAP(vwap) => &vwap.instrument,
            // FeatureEvent::SMA(sma) => sma.instrument.clone(),
            // FeatureEvent::EMA(ema) => ema.instrument.clone(),
        }
    }

    pub fn event_time(&self) -> &OffsetDateTime {
        match self {
            FeatureEvent::VWAP(vwap) => &vwap.event_time,
            // FeatureEvent::SMA(sma) => sma.event_time,
            // FeatureEvent::EMA(ema) => ema.event_time,
        }
    }
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
    async fn start(&self) {
        match self {
            FeatureType::VWAP(f) => f.start().await,
        }
    }
}
