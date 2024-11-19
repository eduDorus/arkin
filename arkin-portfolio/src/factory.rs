use std::sync::Arc;

use crate::{Portfolio, PortfolioConfig, PortfolioType, SingleStrategyPortfolioBuilder};

pub struct PortfolioFactory {}

impl PortfolioFactory {
    pub fn from_config(config: &PortfolioConfig) -> Arc<dyn Portfolio> {
        let portfolio: Arc<dyn Portfolio> = match &config.portfolio {
            PortfolioType::SingleStrategy(_c) => Arc::new(
                SingleStrategyPortfolioBuilder::default()
                    .build()
                    .expect("Failed to build SimplePortfolio"),
            ),
        };
        portfolio
    }
}
