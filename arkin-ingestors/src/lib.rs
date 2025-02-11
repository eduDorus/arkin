mod args;
mod binance;
mod config;
mod errors;
mod factory;
mod sim;
mod tardis;
mod traits;
mod ws;

pub use binance::BinanceIngestor;
pub use errors::IngestorError;
pub use factory::IngestorFactory;
pub use sim::SimIngestor;
pub use tardis::TardisIngestor;
pub use traits::Ingestor;

pub mod prelude {
    pub use crate::args::*;
    pub use crate::binance::BinanceIngestorBuilder;
    pub use crate::config::*;
    pub use crate::errors::IngestorError;
    pub use crate::factory::IngestorFactory;
    pub use crate::sim::SimIngestor;
    pub use crate::traits::Ingestor;
}
