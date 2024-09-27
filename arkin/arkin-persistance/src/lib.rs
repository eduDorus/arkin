mod config;
mod repos;
mod service;
mod services;

pub use config::*;
pub use service::PersistanceService;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::service::PersistanceService;
}

pub const BIND_LIMIT: usize = 65535;
pub const MAX_CONCURRENT_QUERIES: usize = 10; // Adjust as needed
