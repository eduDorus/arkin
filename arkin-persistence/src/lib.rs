mod config;
mod errors;
mod repos;
mod service;
mod services;
mod traits;

pub use config::*;
pub use errors::*;
pub use service::*;
pub use traits::*;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::errors::*;
    pub use crate::service::PersistenceService;
    pub use crate::traits::*;
}

pub const BIND_LIMIT: usize = 65535;
pub const MAX_CONCURRENT_QUERIES: usize = 10; // Adjust as needed
