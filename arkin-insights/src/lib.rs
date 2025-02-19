// mod allocation;
mod config;
mod errors;
mod feature_factory;
// mod forecast;
mod pipeline;
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
    pub use crate::service::InsightsService;
    pub use crate::traits::*;
}
