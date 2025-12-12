use std::{fmt, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{
    BalanceUpdate, BalanceUpdateDto, EventPayload, PersistenceReader, PositionUpdate, PositionUpdateDto, Venue,
    VenueQuery,
};

#[derive(Debug, Clone, TypedBuilder)]
pub struct VenueAccountUpdate {
    #[builder(default)]
    pub id: Uuid,
    pub event_time: UtcDateTime,
    pub venue: Arc<Venue>,
    pub balances: Vec<BalanceUpdate>,
    pub positions: Vec<PositionUpdate>,
    pub reason: String, // "m" from stream, e.g., "ORDER"
}

#[async_trait]
impl EventPayload for VenueAccountUpdate {
    type Dto = VenueAccountUpdateDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let venue = persistence
            .get_venue(&VenueQuery::builder().id(dto.venue_id).build())
            .await
            .context(format!("Failed to get venue with id {}", dto.venue_id))?;

        let mut balances = Vec::new();
        for b in dto.balances {
            balances.push(BalanceUpdate::from_dto(b, persistence.clone()).await?);
        }

        let mut positions = Vec::new();
        for p in dto.positions {
            positions.push(PositionUpdate::from_dto(p, persistence.clone()).await?);
        }

        Ok(VenueAccountUpdate::builder()
            .id(dto.id)
            .event_time(dto.event_time)
            .venue(venue)
            .balances(balances)
            .positions(positions)
            .reason(dto.reason)
            .build())
    }
}

impl PartialEq for VenueAccountUpdate {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for VenueAccountUpdate {}

impl fmt::Display for VenueAccountUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Account Update (ID: {}, Time: {}, Reason: {})",
            self.id, self.event_time, self.reason
        )?;
        writeln!(f, "Balances:")?;
        for bal in &self.balances {
            writeln!(
                f,
                "  - Asset: {}, Change: {}, Quantity: {}, Type: {:?}",
                bal.asset, bal.quantity_change, bal.quantity, bal.account_type
            )?;
        }
        writeln!(f, "Positions:")?;
        for pos in &self.positions {
            writeln!(
                f,
                "  - Instrument: {}, Entry: {}, Qty: {}, Realized PNL: {}, Unreal PNL: {}, Side: {:?}",
                pos.instrument.symbol,
                pos.entry_price,
                pos.quantity,
                pos.realized_pnl,
                pos.unrealized_pnl,
                pos.position_side
            )?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct VenueAccountUpdateDto {
    pub id: Uuid,
    pub event_time: UtcDateTime,
    pub venue_id: Uuid,
    pub balances: Vec<BalanceUpdateDto>,
    pub positions: Vec<PositionUpdateDto>,
    pub reason: String,
}

impl From<VenueAccountUpdate> for VenueAccountUpdateDto {
    fn from(update: VenueAccountUpdate) -> Self {
        Self {
            id: update.id,
            event_time: update.event_time,
            venue_id: update.venue.id,
            balances: update.balances.iter().map(|b| b.clone().into()).collect(),
            positions: update.positions.iter().map(|p| p.clone().into()).collect(),
            reason: update.reason,
        }
    }
}
