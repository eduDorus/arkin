use async_trait::async_trait;
use mockall::automock;
use tokio_util::sync::CancellationToken;

use crate::PersistenceError;

#[automock]
#[async_trait]
pub trait Persistor: std::fmt::Debug + Send + Sync {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), PersistenceError>;
    async fn flush(&self) -> Result<(), PersistenceError>;
    async fn close(&self) -> Result<(), PersistenceError>;
}
