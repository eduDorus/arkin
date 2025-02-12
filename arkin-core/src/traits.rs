use async_trait::async_trait;

use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait RunnableService: Send + Sync {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error>;
}
