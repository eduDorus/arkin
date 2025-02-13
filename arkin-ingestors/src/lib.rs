mod binance;
mod errors;
mod sim;
mod tardis;
mod traits;
mod ws;

pub use binance::BinanceIngestor;
pub use errors::IngestorError;
pub use sim::SimIngestor;
pub use tardis::TardisIngestor;
pub use traits::Ingestor;

pub mod prelude {
    pub use crate::binance::BinanceIngestor;
    pub use crate::errors::IngestorError;
    pub use crate::sim::SimIngestor;
    pub use crate::tardis::*;
    pub use crate::traits::Ingestor;
}
