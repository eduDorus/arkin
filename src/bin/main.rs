use anyhow::Result;
use aurelion::config;
use aurelion::logging;
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

    Ok(())
}
