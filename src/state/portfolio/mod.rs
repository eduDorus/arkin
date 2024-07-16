use tracing::info;

use crate::models::{Fill, Position};

pub trait Portfolio {
    fn handle_position_update(&self, update: &Position);
    fn handle_fill_update(&self, fill: &Fill) {}
}

pub enum PortfolioType {
    Single(SinglePortfolio),
}

impl Portfolio for PortfolioType {
    fn handle_position_update(&self, update: &Position) {
        match self {
            PortfolioType::Single(portfolio) => portfolio.handle_position_update(update),
        }
    }

    fn handle_fill_update(&self, fill: &Fill) {
        match self {
            PortfolioType::Single(portfolio) => portfolio.handle_fill_update(fill),
        }
    }
}

#[derive(Default)]
pub struct SinglePortfolio {}

impl SinglePortfolio {
    pub fn new() -> Self {
        SinglePortfolio {}
    }
}

impl Portfolio for SinglePortfolio {
    fn handle_position_update(&self, update: &Position) {
        info!("Portfolio received position update: {}", update)
    }

    fn handle_fill_update(&self, fill: &Fill) {
        info!("Portfolio received fill update: {}", fill)
    }
}
