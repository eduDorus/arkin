use std::sync::Arc;

use arkin_core::prelude::*;

use crate::{PortfolioConfig, PortfolioService, PortfolioType, SingleStrategyPortfolio};

pub struct PortfolioFactory {}

impl PortfolioFactory {
    pub fn init(pubsub: Arc<PubSub>) -> Arc<dyn PortfolioService> {
        let config = load::<PortfolioConfig>();
        let portfolio: Arc<dyn PortfolioService> = match &config.portfolio {
            PortfolioType::SingleStrategy(_c) => {
                Arc::new(SingleStrategyPortfolio::builder().pubsub(pubsub.clone()).build())
            }
        };
        portfolio
    }
}
