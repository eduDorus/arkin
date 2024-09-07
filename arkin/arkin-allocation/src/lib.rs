mod config;
mod factory;
mod manager;
mod simple;

pub use config::*;
pub use manager::{AllocationManager, AllocationModule};

pub mod prelude {
    pub use crate::config::*;
    pub use crate::manager::*;
}
