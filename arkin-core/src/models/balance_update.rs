use std::{fmt, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{AccountType, AssetQuery, EventPayload, PersistenceReader, Venue, VenueQuery};

use super::Asset;

#[derive(Debug, Clone, TypedBuilder)]
pub struct BalanceUpdate {
    #[builder(default)]
    pub id: Uuid,
    pub event_time: UtcDateTime,
    pub venue: Arc<Venue>,
    pub account_type: AccountType,
    pub asset: Arc<Asset>,
    pub quantity_change: Decimal,
    pub quantity: Decimal,
}

#[async_trait]
impl EventPayload for BalanceUpdate {
    type Dto = BalanceUpdateDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let venue = persistence
            .get_venue(&VenueQuery::builder().id(dto.venue_id).build())
            .await
            .context(format!("Failed to get venue with id {}", dto.venue_id))?;
        let asset = persistence
            .get_asset(&AssetQuery::builder().id(dto.asset_id).build())
            .await
            .context(format!("Failed to get asset with id {}", dto.asset_id))?;

        Ok(BalanceUpdate::builder()
            .id(dto.id)
            .event_time(dto.event_time)
            .venue(venue)
            .account_type(dto.account_type)
            .asset(asset)
            .quantity_change(dto.quantity_change)
            .quantity(dto.quantity)
            .build())
    }
}

impl PartialEq for BalanceUpdate {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BalanceUpdate {}

impl fmt::Display for BalanceUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Balance: {} {} (change: {})",
            self.asset, self.quantity, self.quantity_change
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct BalanceUpdateDto {
    pub id: Uuid,
    pub event_time: UtcDateTime,
    pub venue_id: Uuid,
    pub account_type: AccountType,
    pub asset_id: Uuid,
    pub quantity_change: Decimal,
    pub quantity: Decimal,
}

impl From<BalanceUpdate> for BalanceUpdateDto {
    fn from(update: BalanceUpdate) -> Self {
        Self {
            id: update.id,
            event_time: update.event_time,
            venue_id: update.venue.id,
            account_type: update.account_type,
            asset_id: update.asset.id,
            quantity_change: update.quantity_change,
            quantity: update.quantity,
        }
    }
}
