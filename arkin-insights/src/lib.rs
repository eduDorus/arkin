mod config;
mod errors;
mod feature_pipeline;
mod features;
mod math;
mod service;
mod traits;

pub use errors::*;
pub use feature_pipeline::*;
pub use features::*;
pub use service::InsightService;
pub use traits::*;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::errors::*;
    pub use crate::feature_pipeline::{FeatureFactory, FeaturePipeline};
    pub use crate::features::*;
    pub use crate::math::*;
    pub use crate::service::InsightService;
    pub use crate::traits::*;
}
