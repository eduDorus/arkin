use std::time::Duration;

use mimalloc::MiMalloc;
use time::macros::datetime;
use tokio_rustls::rustls::crypto::aws_lc_rs;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tracing::info;

use arkin_allocation::prelude::*;
use arkin_common::prelude::*;
use arkin_engine::prelude::*;
use arkin_execution::prelude::*;
use arkin_insights::prelude::*;
use arkin_market::prelude::*;
use arkin_persistance::prelude::*;
use arkin_portfolio::prelude::*;
use arkin_strategies::prelude::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();
    info!("Starting Arkin 🚀");

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    // Start up binance ingestor
    let ingestor_manager = IngestorManager::from_config(&load::<IngestorConfig>());
    // let database = DBManager::from_config(&load::<PersistanceConfig>().database);
}
