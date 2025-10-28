mod config;
mod errors;
mod feature_pipeline;
mod features;
mod instrument_scope;
mod math;
mod service;
mod synthetics;
mod traits;

pub use errors::*;
pub use feature_pipeline::*;
pub use features::*;
pub use instrument_scope::*;
pub use service::InsightService;
pub use synthetics::*;
pub use traits::*;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::errors::*;
    pub use crate::feature_pipeline::{FeatureFactory, FeaturePipeline};
    pub use crate::features::*;
    pub use crate::instrument_scope::*;
    pub use crate::math::*;
    pub use crate::service::InsightService;
    pub use crate::synthetics::*;
    pub use crate::traits::*;
}
