use tracing::info;

use super::{
    allocation::AllocationType, errors::EngineError, execution::ExecutionType, strategies::StrategyType, Trader,
};

#[derive(Clone)]
pub struct DefaultTrader {
    pub strategies: StrategyType,
    pub allocation: AllocationType,
    pub execution: ExecutionType,
}

impl DefaultTrader {
    pub fn builder() -> DefaultEngineBuilder {
        DefaultEngineBuilder::default()
    }
}

impl Trader for DefaultTrader {
    async fn start(&self) {
        info!(
            "Starting trader with strategy: {}, allocation: {}, execution: {}",
            self.strategies, self.allocation, self.execution
        );
    }
}

#[derive(Default)]
pub struct DefaultEngineBuilder {
    strategy: Option<StrategyType>,
    allocation: Option<AllocationType>,
    execution: Option<ExecutionType>,
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

    pub fn with_execution(mut self, execution: ExecutionType) -> Self {
        self.execution = Some(execution);
        self
    }

    pub fn build(self) -> Result<DefaultTrader, EngineError> {
        Ok(DefaultTrader {
            strategies: self.strategy.ok_or(EngineError::BuilderError("Strategy not set".into()))?,
            allocation: self.allocation.ok_or(EngineError::BuilderError("Allocation not set".into()))?,
            execution: self.execution.ok_or(EngineError::BuilderError("Execution not set".into()))?,
        })
    }
}
