mod base;
mod config;
mod factory;
mod manager;
mod pipeline;
mod risk;
mod state;
mod ta;

use base::*;
use pipeline::*;
use ta::*;

pub use manager::InsightsManager;

pub mod prelude {
    pub use crate::base::*;
    pub use crate::config::*;
    pub use crate::factory::*;
    pub use crate::manager::*;
    pub use crate::pipeline::*;
    // pub use crate::risk::*;
    pub use crate::state::*;
    pub use crate::ta::*;
}
