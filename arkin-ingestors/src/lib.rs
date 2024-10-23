mod binance;
mod config;
mod factory;
mod service;
mod tardis;
mod ws;

pub use binance::BinanceIngestor;
pub use factory::IngestorFactory;
pub use service::IngestorService;
pub use tardis::TardisIngestor;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::service::IngestorService;
}
