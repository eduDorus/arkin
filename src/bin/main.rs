use anyhow::Result;
use aurelion::config;
use aurelion::logging;
use aurelion::server::Server;
use mimalloc::MiMalloc;
use tracing::info;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    logging::init_tracing();
    info!("Starting Aurelion 🚀");

    let config = config::load();
    info!("Loaded configuration: {:?}", config);

    let server = Server::builder().config(&config).build();
    server.run().await;
    Ok(())
}
