mod binance;
mod config;
mod errors;
mod factory;
mod service;
mod tardis;
mod traits;
mod ws;

pub use binance::BinanceIngestor;
pub use errors::IngestorError;
pub use factory::IngestorFactory;
pub use service::IngestorService;
pub use tardis::TardisIngestor;
pub use traits::Ingestor;

pub mod prelude {
    pub use crate::binance::BinanceIngestorBuilder;
    pub use crate::config::*;
    pub use crate::errors::IngestorError;
    pub use crate::service::IngestorService;
    pub use crate::traits::Ingestor;
}
