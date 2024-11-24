use std::sync::Arc;

use arkin_core::prelude::*;

use crate::{Portfolio, PortfolioConfig, PortfolioType, SingleStrategyPortfolioBuilder};

pub struct PortfolioFactory {}

impl PortfolioFactory {
    pub fn from_config(config: &PortfolioConfig, pubsub: Arc<PubSub>) -> Arc<dyn Portfolio> {
        let portfolio: Arc<dyn Portfolio> = match &config.portfolio {
            PortfolioType::SingleStrategy(_c) => Arc::new(
                SingleStrategyPortfolioBuilder::default()
                    .pubsub(pubsub.clone())
                    .build()
                    .expect("Failed to build SimplePortfolio"),
            ),
        };
        portfolio
    }
}
