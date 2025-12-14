use std::{fmt, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::ValueEnum;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use strum::Display;
use time::UtcDateTime;
use tracing::error;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{
    types::Commission, EventPayload, InstrumentQuery, Notional, PersistenceReader, Price, Quantity, StrategyQuery,
    VenueOrderId,
};

use super::{Instrument, MarketSide, Strategy};

pub type ExecutionOrderId = Uuid;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type, ValueEnum, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "execution_order_type", rename_all = "snake_case")]
pub enum ExecutionStrategyType {
    WideQuoter,
    Maker,
    Taker,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type, PartialOrd, Ord, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "execution_order_status", rename_all = "snake_case")]
pub enum ExecutionOrderStatus {
    New,
    Placed,
    Rejected,
    PartiallyFilled,
    Filled,
    Cancelling,
    PartiallyFilledCancelled,
    Cancelled,
    PartiallyFilledExpired,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, TypedBuilder, Hash)]
#[builder(mutators(
    #[mutator(requires = [instrument])]
    pub fn set_price(&mut self, value: Price) {
        if value.is_zero() {
            self.price = Price::ZERO;
            return;
        }
        let scaling_factor = Decimal::ONE / self.instrument.tick_size;
        let scaled_price = value * scaling_factor;
        let rounded_scaled_price = scaled_price.round();
        let rounded_price = rounded_scaled_price * self.instrument.tick_size;
        self.price = rounded_price.round_dp(self.instrument.price_precision);
    }

    #[mutator(requires = [instrument])]
    pub fn set_quantity(&mut self, value: Quantity) {
        if value.is_zero() {
            self.quantity = Quantity::ZERO;
            return;
        }
        let scaling_factor = Decimal::ONE / self.instrument.lot_size;
        let scaled_quantity = value * scaling_factor;
        let rounded_scaled_quantity = scaled_quantity.round();
        let round_quantity = rounded_scaled_quantity * self.instrument.lot_size;
        self.quantity = round_quantity.round_dp(self.instrument.quantity_precision);
    }
))]
pub struct ExecutionOrder {
    #[builder(default = Uuid::new_v4())]
    pub id: ExecutionOrderId,
    pub instrument: Arc<Instrument>,
    pub strategy: Option<Arc<Strategy>>,
    #[builder(default = vec![])]
    pub venue_order_ids: Vec<VenueOrderId>,
    pub exec_strategy_type: ExecutionStrategyType,
    pub side: MarketSide,
    #[builder(via_mutators(init = Price::ZERO))]
    pub price: Price,
    #[builder(via_mutators(init = Quantity::ZERO))]
    pub quantity: Quantity,
    #[builder(default = Price::ZERO)]
    pub fill_price: Price,
    #[builder(default = Quantity::ZERO)]
    pub filled_quantity: Quantity,
    #[builder(default = Notional::ZERO)]
    pub total_commission: Commission,
    #[builder(default = ExecutionOrderStatus::New)]
    pub status: ExecutionOrderStatus,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl ExecutionOrder {
    pub fn place(&mut self, event_time: UtcDateTime) {
        let new_status = ExecutionOrderStatus::Placed;
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn add_fill(&mut self, event_time: UtcDateTime, price: Price, quantity: Quantity, commission: Commission) {
        if !matches!(self.status, ExecutionOrderStatus::Placed | ExecutionOrderStatus::Cancelling) {
            error!("Cannot add fill in state {}", self.status);
            return;
        }
        self.fill_price =
            (self.fill_price * self.filled_quantity + price * quantity) / (self.filled_quantity + quantity);
        self.filled_quantity += quantity;
        self.total_commission += commission;
        self.updated = event_time;

        if self.remaining_quantity().is_zero() {
            let new_status = ExecutionOrderStatus::Filled;
            if self.is_valid_transition(&new_status) {
                self.status = new_status;
            } else {
                error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
            }
        }
    }

    pub fn cancel(&mut self, event_time: UtcDateTime) {
        let new_status = ExecutionOrderStatus::Cancelling;
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn expire(&mut self, event_time: UtcDateTime) {
        let new_status = if self.has_fill() {
            ExecutionOrderStatus::PartiallyFilledExpired
        } else {
            ExecutionOrderStatus::Expired
        };
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn reject(&mut self, event_time: UtcDateTime) {
        let new_status = ExecutionOrderStatus::Rejected;
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn finalize_terminate(&mut self, event_time: UtcDateTime) -> bool {
        let new_status = match self.status {
            ExecutionOrderStatus::Cancelling => {
                if self.remaining_quantity().is_zero() {
                    ExecutionOrderStatus::Filled
                } else if self.has_fill() {
                    ExecutionOrderStatus::PartiallyFilledCancelled
                } else {
                    ExecutionOrderStatus::Cancelled
                }
            }
            _ => {
                return false;
            }
        };
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
            true
        } else {
            error!("Invalid transition to {} from {}", new_status, self.status);
            false
        }
    }

    fn is_valid_transition(&self, new_status: &ExecutionOrderStatus) -> bool {
        matches!(
            (&self.status, new_status),
            (ExecutionOrderStatus::New, ExecutionOrderStatus::Placed)
                | (ExecutionOrderStatus::New, ExecutionOrderStatus::Rejected)
                | (ExecutionOrderStatus::New, ExecutionOrderStatus::Cancelling)
                | (ExecutionOrderStatus::Placed, ExecutionOrderStatus::PartiallyFilled)
                | (ExecutionOrderStatus::Placed, ExecutionOrderStatus::Filled)
                | (ExecutionOrderStatus::Placed, ExecutionOrderStatus::Cancelling)
                | (ExecutionOrderStatus::Placed, ExecutionOrderStatus::Expired)
                | (ExecutionOrderStatus::PartiallyFilled, ExecutionOrderStatus::PartiallyFilled)
                | (ExecutionOrderStatus::PartiallyFilled, ExecutionOrderStatus::Filled)
                | (ExecutionOrderStatus::Cancelling, ExecutionOrderStatus::PartiallyFilledCancelled)
                | (ExecutionOrderStatus::Cancelling, ExecutionOrderStatus::Filled)
                | (ExecutionOrderStatus::Cancelling, ExecutionOrderStatus::Cancelled)
        )
    }

    pub fn is_new(&self) -> bool {
        matches!(self.status, ExecutionOrderStatus::New)
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            ExecutionOrderStatus::Placed | ExecutionOrderStatus::PartiallyFilled | ExecutionOrderStatus::Cancelling
        )
    }

    pub fn is_terminating(&self) -> bool {
        matches!(self.status, ExecutionOrderStatus::Cancelling)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            ExecutionOrderStatus::Rejected
                | ExecutionOrderStatus::PartiallyFilledCancelled
                | ExecutionOrderStatus::PartiallyFilledExpired
                | ExecutionOrderStatus::Filled
                | ExecutionOrderStatus::Cancelled
                | ExecutionOrderStatus::Expired
        )
    }

    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity - self.filled_quantity
    }

    pub fn has_fill(&self) -> bool {
        self.filled_quantity > Quantity::ZERO
    }

    pub fn notional(&self) -> Notional {
        self.fill_price * self.filled_quantity
    }

    pub fn total_value(&self) -> Decimal {
        self.price * self.quantity * self.instrument.contract_size
    }
}

#[async_trait]
impl EventPayload for ExecutionOrder {
    type Dto = ExecutionOrderDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let instrument = persistence
            .get_instrument(&InstrumentQuery::builder().id(dto.instrument_id).build())
            .await
            .context(format!("Failed to get instrument with id {}", dto.instrument_id))?;

        let strategy = if let Some(sid) = dto.strategy_id {
            persistence.get_strategy(&StrategyQuery::builder().id(sid).build()).await.ok()
        } else {
            None
        };

        let mut order = ExecutionOrder::builder()
            .id(dto.id)
            .instrument(instrument)
            .strategy(strategy)
            .venue_order_ids(dto.venue_order_ids)
            .exec_strategy_type(dto.exec_strategy_type)
            .side(dto.side)
            .fill_price(dto.fill_price)
            .filled_quantity(dto.filled_quantity)
            .total_commission(dto.total_commission)
            .status(dto.status)
            .created(dto.created)
            .updated(dto.updated)
            .build();

        order.price = dto.price;
        order.quantity = dto.quantity;

        Ok(order)
    }
}

impl fmt::Display for ExecutionOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} ({}) {}",
            self.side, self.quantity, self.instrument.symbol, self.exec_strategy_type, self.status
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct ExecutionOrderDto {
    pub id: ExecutionOrderId,
    pub instrument_id: Uuid,
    pub strategy_id: Option<Uuid>,
    pub venue_order_ids: Vec<VenueOrderId>,
    pub exec_strategy_type: ExecutionStrategyType,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
    pub fill_price: Price,
    pub filled_quantity: Quantity,
    pub total_commission: Commission,
    pub status: ExecutionOrderStatus,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl From<ExecutionOrder> for ExecutionOrderDto {
    fn from(order: ExecutionOrder) -> Self {
        Self {
            id: order.id,
            instrument_id: order.instrument.id,
            strategy_id: order.strategy.map(|s| s.id),
            venue_order_ids: order.venue_order_ids,
            exec_strategy_type: order.exec_strategy_type,
            side: order.side,
            price: order.price,
            quantity: order.quantity,
            fill_price: order.fill_price,
            filled_quantity: order.filled_quantity,
            total_commission: order.total_commission,
            status: order.status,
            created: order.created,
            updated: order.updated,
        }
    }
}
