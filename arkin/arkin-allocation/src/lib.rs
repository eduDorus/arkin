mod config;
mod equal;
mod factory;
mod manager;

pub use config::*;
pub use manager::{AllocationManager, AllocationModule};

pub mod prelude {
    pub use crate::config::*;
    pub use crate::manager::*;
}
