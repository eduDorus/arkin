use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use crate::IngestorError;

#[async_trait]
pub trait Ingestor: std::fmt::Debug + Send + Sync {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), IngestorError>;
}
