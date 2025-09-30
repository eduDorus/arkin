mod config;
mod context;
#[allow(unused)]
mod repos;
mod service;
#[allow(unused)]
mod stores;
mod test_utils;

pub use config::*;
pub use service::*;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::service::*;
}

pub const BIND_LIMIT: usize = 65535;
