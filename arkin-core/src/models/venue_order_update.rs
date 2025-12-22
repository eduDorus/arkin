use core::fmt;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{Asset, AssetQuery, Commission, EventPayload, PersistenceReader, Price, Quantity, VenueOrderStatus};

#[derive(Debug, Clone, TypedBuilder)]
pub struct VenueOrderUpdate {
    pub id: Uuid,
    pub event_time: UtcDateTime,

    // State
    #[builder(default = Quantity::ZERO)]
    pub filled_quantity: Quantity,
    #[builder(default = Price::ZERO)]
    pub filled_price: Price,

    // Delta
    #[builder(default = Quantity::ZERO)]
    pub last_filled_quantity: Quantity,
    #[builder(default = Price::ZERO)]
    pub last_filled_price: Price,
    #[builder(default = Commission::ZERO)]
    pub commission: Commission,
    #[builder(default)]
    pub commission_asset: Option<Arc<Asset>>,

    pub status: VenueOrderStatus,
}

impl fmt::Display for VenueOrderUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} @ {} {} @ {}",
            self.id,
            self.status,
            self.filled_quantity,
            self.filled_price,
            self.last_filled_quantity,
            self.last_filled_price
        )
    }
}

#[async_trait]
impl EventPayload for VenueOrderUpdate {
    type Dto = VenueOrderUpdateDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let commission_asset = if let Some(asset_id) = dto.commission_asset_id {
            Some(persistence.get_asset(&AssetQuery::builder().id(asset_id).build()).await?)
        } else {
            None
        };

        Ok(VenueOrderUpdate::builder()
            .id(dto.id)
            .event_time(dto.event_time)
            .status(dto.status)
            .filled_quantity(dto.filled_quantity)
            .filled_price(dto.filled_price)
            .last_filled_quantity(dto.last_filled_quantity)
            .last_filled_price(dto.last_filled_price)
            .commission(dto.commission)
            .commission_asset(commission_asset)
            .build())
    }
}

#[derive(Serialize, Deserialize)]
pub struct VenueOrderUpdateDto {
    pub id: Uuid,
    pub event_time: UtcDateTime,
    pub status: VenueOrderStatus,
    pub filled_quantity: Quantity,
    pub filled_price: Price,
    pub last_filled_quantity: Quantity,
    pub last_filled_price: Price,
    pub commission: Commission,
    pub commission_asset_id: Option<Uuid>,
}

impl From<VenueOrderUpdate> for VenueOrderUpdateDto {
    fn from(update: VenueOrderUpdate) -> Self {
        Self {
            id: update.id,
            event_time: update.event_time,
            status: update.status,
            filled_quantity: update.filled_quantity,
            filled_price: update.filled_price,
            last_filled_quantity: update.last_filled_quantity,
            last_filled_price: update.last_filled_price,
            commission: update.commission,
            commission_asset_id: update.commission_asset.map(|a| a.id),
        }
    }
}
