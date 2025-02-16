use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_persistence::PersistenceService;
use arkin_portfolio::prelude::*;
use rust_decimal_macros::dec;
use time::OffsetDateTime;

use crate::config::{PortfolioConfig, PortfolioType};

pub struct PortfolioFactory {}

impl PortfolioFactory {
    pub async fn init(pubsub: Arc<PubSub>, persistence: Arc<PersistenceService>) -> Arc<dyn PortfolioService> {
        let config = load::<PortfolioConfig>();
        let portfolio: Arc<dyn PortfolioService> = match &config.portfolio {
            PortfolioType::SingleStrategy(_c) => {
                let portfolio = SingleStrategyPortfolio::builder().pubsub(pubsub.clone()).build();

                let asset = persistence.asset_store.read_by_symbol("USDT").await.unwrap();
                let balance = BalanceUpdate::builder()
                    .event_time(OffsetDateTime::now_utc())
                    .portfolio(test_portfolio())
                    .asset(asset)
                    .quantity(dec!(100_000))
                    .build();
                portfolio.add_balance(balance.into());
                Arc::new(portfolio)
            }
        };
        portfolio
    }
}
