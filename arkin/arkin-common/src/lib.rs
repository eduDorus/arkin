mod config;
mod constants;
mod events;
mod logging;
mod models;
mod snapshots;
mod types;
mod utils;

// Re-export items that should be publicly accessible
pub use config::load;
pub use events::Event;
pub use models::Instrument;
pub use types::{FeatureId, Maturity, Notional, Price, Quantity, StrategyId, Weight};

// Prelude module
pub mod prelude {
    pub use crate::{config::*, constants::*, events::*, logging::*, models::*, snapshots::*, types::*, utils::*};

    // Re-export commonly used traits
}
