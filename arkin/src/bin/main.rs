use anyhow::Result;
use arkin::config;
use arkin::logging;
use arkin::server::Server;
use mimalloc::MiMalloc;
use tracing::info;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    logging::init_tracing();
    info!("Starting Arkin 🚀");

    let config = config::load();
    info!("Loaded configuration: {}", serde_json::to_string_pretty(&config)?);

    let server = Server::builder().config(&config).build();
    server.run().await;
    Ok(())
}
