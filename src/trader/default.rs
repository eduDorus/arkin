use tracing::info;

use crate::trader::strategies::Strategy;

use super::{allocation::AllocationType, errors::EngineError, strategies::StrategyType, Trader};

#[derive(Clone)]
pub struct DefaultTrader {
    pub strategies: StrategyType,
    pub allocation: AllocationType,
}

impl DefaultTrader {
    pub fn builder() -> DefaultEngineBuilder {
        DefaultEngineBuilder::default()
    }
}

impl Trader for DefaultTrader {
    async fn start(&self) {
        info!(
            "Starting trader with strategy: {}, allocation: {}",
            self.strategies, self.allocation
        );
        self.strategies.start().await;
    }
}

#[derive(Default)]
pub struct DefaultEngineBuilder {
    strategy: Option<StrategyType>,
    allocation: Option<AllocationType>,
}

impl DefaultEngineBuilder {
    pub fn with_strategy(mut self, strategy: StrategyType) -> Self {
        self.strategy = Some(strategy);
        self
    }

    pub fn with_allocation(mut self, allocation: AllocationType) -> Self {
        self.allocation = Some(allocation);
        self
    }

    pub fn build(self) -> Result<DefaultTrader, EngineError> {
        Ok(DefaultTrader {
            strategies: self.strategy.ok_or(EngineError::BuilderError("Strategy not set".into()))?,
            allocation: self.allocation.ok_or(EngineError::BuilderError("Allocation not set".into()))?,
        })
    }
}
