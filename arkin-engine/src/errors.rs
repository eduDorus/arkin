use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradingEngineError {
    #[error(transparent)]
    PersistenceError(#[from] arkin_persistence::PersistenceError),

    #[error(transparent)]
    AccountingError(#[from] arkin_accounting::AccountingError),

    #[error(transparent)]
    IngestorError(#[from] arkin_ingestors::IngestorError),

    #[error(transparent)]
    InsightsError(#[from] arkin_insights::InsightsError),

    #[error(transparent)]
    StrategyError(#[from] arkin_strategies::StrategyError),

    #[error(transparent)]
    AllocationOptimError(#[from] arkin_allocation::AllocationOptimError),

    #[error(transparent)]
    OrderManagerError(#[from] arkin_ordermanager::OrderManagerError),

    #[error(transparent)]
    ExecutorError(#[from] arkin_executors::ExecutorError),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}
