use rust_decimal::Decimal;
use std::sync::Arc;
use time::OffsetDateTime;

use crate::{
    config::PortfolioConfig,
    models::{ExecutionOrder, ExecutionStatus, Instrument, Notional, PositionSnapshot},
    state::StateManager,
    strategies::StrategyId,
};

pub struct Portfolio {
    state: Arc<StateManager>,
    initial_capital: Notional,
    leverage: Decimal,
}

impl Portfolio {
    pub fn from_config(state: Arc<StateManager>, config: &PortfolioConfig) -> Self {
        Self {
            state,
            initial_capital: config.initial_capital.into(),
            leverage: config.leverage,
        }
    }

    pub fn update(&self, order: &ExecutionOrder) {
        if order.status == ExecutionStatus::PartiallyFilled
            || order.status == ExecutionStatus::PartiallyFilledCancelled
            || order.status == ExecutionStatus::Filled
        {
            self.update_position(order);
        }
    }

    fn update_position(&self, order: &ExecutionOrder) {
        todo!()
    }

    pub fn open_positions(&self, timestamp: &OffsetDateTime) -> PositionSnapshot {
        todo!()
    }

    pub fn total_capital(&self) -> Notional {
        todo!()
    }

    pub fn buying_power(&self, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn leverage(&self, timestamp: &OffsetDateTime) -> Decimal {
        self.leverage
    }

    pub fn total_value(&self, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn total_value_strategy(&self, strategy: &StrategyId, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn total_value_instrument(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn total_exposure(&self, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn total_exposure_strategy(&self, strategy: &StrategyId, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn total_exposure_instrument(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn total_realized_pnl(&self, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn total_realized_pnl_strategy(&self, strategy: &StrategyId, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn total_realized_pnl_instrument(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn total_unrealized_pnl(&self, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn total_unrealized_pnl_strategy(&self, strategy: &StrategyId, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }

    pub fn total_unrealized_pnl_instrument(&self, instrument: &Instrument, timestamp: &OffsetDateTime) -> Notional {
        todo!()
    }
}
