use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
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
          "VenueTradeUpdate event_time: {}, order: {}, fill_price: {}, fill_quantity: {}, commission_asset: {}, commission: {}, realized_pnl: {}",
          self.event_time,
          self.order,
          self.fill_price,
          self.fill_quantity,
          self.commission_asset,
          self.commission,
          self.realized_pnl
        )
    }
}
