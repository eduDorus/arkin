use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{types::Commission, Asset, Price, Quantity, VenueOrder};

#[derive(Debug, Clone, TypedBuilder)]
pub struct VenueTradeUpdate {
    #[builder(default)]
    pub id: Uuid,
    pub event_time: UtcDateTime,
    pub order: Arc<VenueOrder>,
    pub fill_price: Price,
    pub fill_quantity: Quantity,
    pub commission_asset: Arc<Asset>,
    pub commission: Commission,
    pub realized_pnl: Decimal,
}

impl PartialEq for VenueTradeUpdate {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for VenueTradeUpdate {}

impl fmt::Display for VenueTradeUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VenueTrade: {} filled {} @ {} (comm: {} {}, pnl: {})",
            self.order,
            self.fill_quantity,
            self.fill_price,
            self.commission,
            self.commission_asset,
            self.realized_pnl
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct VenueTradeUpdateDto {
    pub id: Uuid,
    pub event_time: UtcDateTime,
    pub order_id: Uuid,
    pub fill_price: Price,
    pub fill_quantity: Quantity,
    pub commission_asset_id: Uuid,
    pub commission: Commission,
    pub realized_pnl: Decimal,
}

impl From<VenueTradeUpdate> for VenueTradeUpdateDto {
    fn from(update: VenueTradeUpdate) -> Self {
        Self {
            id: update.id,
            event_time: update.event_time,
            order_id: update.order.id,
            fill_price: update.fill_price,
            fill_quantity: update.fill_quantity,
            commission_asset_id: update.commission_asset.id,
            commission: update.commission,
            realized_pnl: update.realized_pnl,
        }
    }
}
