mod config;
mod errors;
mod feature_factory;
mod pipeline;
mod scaler;
mod service;
mod simple;
mod state;
mod ta;
mod traits;

pub use errors::*;
pub use service::InsightsService;
pub use traits::*;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::errors::*;
    pub use crate::scaler::*;
    pub use crate::service::InsightsService;
    pub use crate::state::*;
    pub use crate::traits::*;
}
