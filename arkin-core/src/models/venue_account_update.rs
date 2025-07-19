use std::{fmt, sync::Arc};

use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{BalanceUpdate, PositionUpdate, Venue};

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
