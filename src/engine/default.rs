use super::{allocation::AllocationType, errors::EngineError, execution::ExecutionType, strategies::StrategyType};

pub struct DefaultEngine {
    pub strategies: StrategyType,
    pub allocation: AllocationType,
    pub execution: ExecutionType,
}

impl DefaultEngine {
    pub fn builder() -> DefaultEngineBuilder {
        DefaultEngineBuilder::default()
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
        self.strategies = Some(strategy);
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

    pub fn build(self) -> Result<DefaultEngine, EngineError> {
        Ok(DefaultEngine {
            strategies: self.strategies,
            allocation: self.allocation.ok_or(EngineError::BuilderError("Allocation not set".into()))?,
            execution: self.execution.ok_or(EngineError::BuilderError("Execution not set".into()))?,
        })
    }
}
