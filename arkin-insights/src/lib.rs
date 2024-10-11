mod config;
mod factory;
mod pipeline;
mod service;
mod simple;
mod state;
mod ta;

pub use service::InsightsService;

pub mod prelude {
    // pub use crate::base::*;
    pub use crate::config::*;
    pub use crate::service::InsightsService;
}
