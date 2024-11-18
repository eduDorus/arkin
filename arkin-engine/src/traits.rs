use async_trait::async_trait;

use crate::TradingEngineError;

#[async_trait]
pub trait TradingEngine: Send + Sync {
    async fn start(&self) -> Result<(), TradingEngineError>;
    async fn stop(&self) -> Result<(), TradingEngineError>;
}
