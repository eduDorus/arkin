use std::sync::Arc;

use arkin_core::prelude::*;

use crate::{Accounting, PortfolioConfig, PortfolioType, SingleStrategyPortfolio};

pub struct PortfolioFactory {}

impl PortfolioFactory {
    pub fn from_config(config: &PortfolioConfig, pubsub: Arc<PubSub>) -> Arc<dyn Accounting> {
        let portfolio: Arc<dyn Accounting> = match &config.portfolio {
            PortfolioType::SingleStrategy(_c) => {
                Arc::new(SingleStrategyPortfolio::builder().pubsub(pubsub.clone()).build())
            }
        };
        portfolio
    }
}
