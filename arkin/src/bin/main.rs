use anyhow::Result;
use arkin::config;
use arkin::logging;
use arkin::server::Server;
use mimalloc::MiMalloc;
use tokio_rustls::rustls::crypto::aws_lc_rs;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tracing::debug;
use tracing::info;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    logging::init_tracing();
    info!("Starting Arkin 🚀");

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    let config = config::load();
    debug!("Loaded configuration: {}", serde_json::to_string_pretty(&config)?);

    let server = Server::builder().config(&config).build();
    server.run().await;
    Ok(())
}
