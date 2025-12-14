use std::{fmt, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum::Display;
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{AccountType, EventPayload, InstrumentQuery, PersistenceReader, Price, Quantity};

use super::{Instrument, MarketSide};

#[derive(Clone, Display, Copy, PartialEq, Eq, Debug, Type, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "position_side", rename_all = "snake_case")]
pub enum PositionSide {
    Long,
    Short,
    // Both, // Quantity decides so the position can be both long and short
}

impl From<MarketSide> for PositionSide {
    fn from(side: MarketSide) -> Self {
        match side {
            MarketSide::Buy => PositionSide::Long,
            MarketSide::Sell => PositionSide::Short,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct PositionUpdate {
    #[builder(default)]
    pub id: Uuid,
    pub event_time: UtcDateTime,
    pub instrument: Arc<Instrument>,
    pub account_type: AccountType,
    pub entry_price: Price,
    pub quantity: Quantity,
    pub realized_pnl: Decimal,
    pub unrealized_pnl: Decimal,
    pub position_side: PositionSide,
}

impl PositionUpdate {
    // TODO: This is only for perpetual swaps (For short you still post collateral)
    pub fn market_value(&self) -> Decimal {
        self.entry_price * self.quantity * self.instrument.contract_size
    }

    pub fn notional_value(&self) -> Decimal {
        self.entry_price * self.quantity.abs() * self.instrument.contract_size
    }
}

#[async_trait]
impl EventPayload for PositionUpdate {
    type Dto = PositionUpdateDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let instrument = persistence
            .get_instrument(&InstrumentQuery::builder().id(dto.instrument_id).build())
            .await
            .context(format!("Failed to get instrument with id {}", dto.instrument_id))?;

        Ok(PositionUpdate::builder()
            .id(dto.id)
            .event_time(dto.event_time)
            .instrument(instrument)
            .account_type(dto.account_type)
            .entry_price(dto.entry_price)
            .quantity(dto.quantity)
            .realized_pnl(dto.realized_pnl)
            .unrealized_pnl(dto.unrealized_pnl)
            .position_side(dto.position_side)
            .build())
    }
}

impl PartialEq for PositionUpdate {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PositionUpdate {}

impl fmt::Display for PositionUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Position: {} {} {} @ {} (pnl: {}/{})",
            self.instrument,
            self.position_side,
            self.quantity,
            self.entry_price,
            self.realized_pnl,
            self.unrealized_pnl,
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct PositionUpdateDto {
    pub id: Uuid,
    pub event_time: UtcDateTime,
    pub instrument_id: Uuid,
    pub account_type: AccountType,
    pub entry_price: Price,
    pub quantity: Quantity,
    pub realized_pnl: Decimal,
    pub unrealized_pnl: Decimal,
    pub position_side: PositionSide,
}

impl From<PositionUpdate> for PositionUpdateDto {
    fn from(update: PositionUpdate) -> Self {
        Self {
            id: update.id,
            event_time: update.event_time,
            instrument_id: update.instrument.id,
            account_type: update.account_type,
            entry_price: update.entry_price,
            quantity: update.quantity,
            realized_pnl: update.realized_pnl,
            unrealized_pnl: update.unrealized_pnl,
            position_side: update.position_side,
        }
    }
}
