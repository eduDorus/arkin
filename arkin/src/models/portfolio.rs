use super::{ExecutionOrder, Instrument, Notional, OrderSide, Price, Quantity};
use crate::strategies::StrategyId;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct InternalTrade {
    pub strategy_id: StrategyId,
    pub instrument: Instrument,
    pub side: OrderSide,
    pub open_price: Price,
    pub avg_open_price: Price,
    pub close_price: Price,
    pub avg_close_price: Price,
    pub quantity: Quantity,
    pub realized_pnl: Notional,
    pub commission: Notional,
    pub status: TradeStatus,
    pub created_at: OffsetDateTime,
    pub last_updated_at: OffsetDateTime,
}

impl From<ExecutionOrder> for InternalTrade {
    fn from(order: ExecutionOrder) -> Self {
        Self {
            strategy_id: order.strategy_id,
            instrument: order.instrument,
            side: order.side,
            open_price: order.avg_price,
            avg_open_price: order.avg_price,
            close_price: Price::default(),
            avg_close_price: Price::default(),
            quantity: order.quantity,
            realized_pnl: Notional::default(),
            commission: Notional::default(),
            status: TradeStatus::Open,
            created_at: order.last_updated_at,
            last_updated_at: order.last_updated_at,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TradeStatus {
    Open,
    Closed,
}

#[derive(Default)]
pub struct Position {
    avg_price: Price,
    quantity: Quantity,
}

impl Position {
    pub fn exposure(&self) -> Notional {
        self.quantity.abs() * self.avg_price
    }
}

// impl CompletedTrade {
//     pub fn from_fill(fill: &Fill) -> Self {
//         let position = Self {
//             strategy_id: fill.strategy_id.clone(),
//             instrument: fill.instrument.clone(),
//             start_time: fill.event_time,
//             exit_time: None,
//             entry_price: fill.price,
//             exit_price: None,
//             avg_price: fill.price,
//             quantity: fill.quantity,
//             commission: fill.commission,
//         };
//         debug!("Opening position: {}", position);
//         position
//     }
//     pub fn update(&mut self, fill: &Fill) -> Option<Fill> {
//         let new_quantity = self.quantity + fill.quantity;
//         self.commission += fill.commission;

//         match (new_quantity.is_zero(), self.quantity.is_positive(), new_quantity.is_positive()) {
//             // Quantity is zero so we close the position
//             (true, _, _) => {
//                 //self.avg_price = (self.notional() + fill.notional()) / (self.quantity.abs() + fill.quantity.abs());
//                 self.exit_price = Some(fill.price);
//                 self.exit_time = Some(fill.event_time);
//                 self.status = TradeStatus::Closed;
//                 debug!("Closing position: {}", self);
//                 None
//             }
//             // Position flips
//             (_, true, false) | (_, false, true) => {
//                 // Calculate how many shares we can fill till we reach 0
//                 //self.avg_price =
//                 //(self.notional() + (fill.price * self.quantity)) / (self.quantity.abs() + self.quantity.abs());
//                 self.exit_price = Some(fill.price);
//                 self.exit_time = Some(fill.event_time);
//                 debug!("Closing position: {}", self);
//                 Some(Fill::new(
//                     fill.event_time,
//                     fill.instrument.clone(),
//                     fill.order_id,
//                     fill.strategy_id.clone(),
//                     fill.price,
//                     new_quantity,
//                     fill.commission * (self.quantity.abs() / fill.quantity.abs()),
//                 ))
//             }
//             // Position is still open
//             _ => {
//                 self.avg_price = (self.notional() + fill.notional()) / (self.quantity.abs() + fill.quantity.abs());
//                 self.quantity += fill.quantity;
//                 debug!("Updating position: {}", self);
//                 None
//             }
//         }
//     }

//     pub fn notional(&self) -> Notional {
//         self.avg_price * self.quantity.abs()
//     }
// }

// impl fmt::Display for InternalTrade {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "Completed trade: {} {} avg price: {} quantity: {}",
//             self.start_time.format(TIMESTAMP_FORMAT).unwrap(),
//             self.instrument,
//             self.avg_price,
//             self.quantity
//         )
//     }
// }
