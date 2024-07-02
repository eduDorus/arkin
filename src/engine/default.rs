use crate::{
    allocation::AllocationType,
    execution::ExecutionType,
    features::FeatureType,
    order_manager::{OrderManagerType, SingleOrderManager},
    portfolio::{PortfolioType, SinglePortfolio},
    risk::RiskType,
    state::State,
    strategies::StrategyType,
};

use super::errors::EngineError;

pub struct DefaultEngine {
    pub state: State,
    pub features: Vec<FeatureType>,
    pub strategies: Vec<StrategyType>,
    pub allocation: AllocationType,
    pub risk: RiskType,
    pub execution: ExecutionType,
}

impl DefaultEngine {
    pub fn builder() -> DefaultEngineBuilder {
        DefaultEngineBuilder::default()
    }
}

#[derive(Default)]
pub struct DefaultEngineBuilder {
    features: Vec<FeatureType>,
    strategies: Vec<StrategyType>,
    allocation: Option<AllocationType>,
    risk: Option<RiskType>,
    execution: Option<ExecutionType>,
}

impl DefaultEngineBuilder {
    pub fn with_feature(mut self, feature: FeatureType) -> Self {
        self.features.push(feature);
        self
    }

    pub fn with_strategy(mut self, strategy: StrategyType) -> Self {
        self.strategies.push(strategy);
        self
    }

    pub fn with_allocation(mut self, allocation: AllocationType) -> Self {
        self.allocation = Some(allocation);
        self
    }

    pub fn with_risk(mut self, risk: RiskType) -> Self {
        self.risk = Some(risk);
        self
    }

    pub fn with_execution(mut self, execution: ExecutionType) -> Self {
        self.execution = Some(execution);
        self
    }

    pub fn build(self) -> Result<DefaultEngine, EngineError> {
        let Ok(state) = State::builder()
            .with_order_manager(OrderManagerType::SingleVenue(SingleOrderManager::new()))
            .with_portfolio(PortfolioType::Single(SinglePortfolio::new()))
            .build()
        else {
            return Err(EngineError::BuilderError("State build error".into()));
        };

        Ok(DefaultEngine {
            state,
            features: self.features,
            strategies: self.strategies,
            allocation: self.allocation.ok_or(EngineError::BuilderError("Allocation not set".into()))?,
            risk: self.risk.ok_or(EngineError::BuilderError("Risk not set".into()))?,
            execution: self.execution.ok_or(EngineError::BuilderError("Execution not set".into()))?,
        })
    }
}
