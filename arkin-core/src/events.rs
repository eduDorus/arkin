use std::sync::Arc;

use strum::{Display, EnumDiscriminants, EnumIter};
use time::OffsetDateTime;

use crate::{
    AccountUpdate, BalanceUpdate, Book, ExecutionOrder, InsightsTick, InsightsUpdate, PositionUpdate, Signal, Tick,
    Trade, TransferGroup, VenueOrder,
};

#[derive(Debug, Display, Clone, EnumDiscriminants)]
#[strum_discriminants(name(EventType))]
#[strum_discriminants(derive(Hash), derive(Display), derive(EnumIter))]
#[strum(serialize_all = "snake_case")]
pub enum Event {
    // Market Data
    TickUpdate(Arc<Tick>),
    TradeUpdate(Arc<Trade>),
    BookUpdate(Arc<Book>),
    // Accounting
    BalanceUpdate(Arc<BalanceUpdate>),
    PositionUpdate(Arc<PositionUpdate>),
    AccountNew(Arc<AccountUpdate>),
    TransferNew(Arc<TransferGroup>),
    // Insights
    InsightsTick(Arc<InsightsTick>),
    InsightsUpdate(Arc<InsightsUpdate>),
    // Strategy Signals
    SignalUpdate(Arc<Signal>),
    // Allocation Execution Orders
    NewExecutionOrder(Arc<ExecutionOrder>),
    CancelExecutionOrder(Arc<ExecutionOrder>),
    CancelAllExecutionOrders(OffsetDateTime),

    // Order Manager Venue Orders
    NewVenueOrder(Arc<VenueOrder>),
    CancelVenueOrder(Arc<VenueOrder>),
    CancelAllVenueOrders(OffsetDateTime),
    ExecutionOrderPlaced(Arc<ExecutionOrder>),
    ExecutionOrderFilled(Arc<ExecutionOrder>),
    ExecutionOrderCancelled(Arc<ExecutionOrder>),

    // Execution Venue Orders
    VenueOrderInflight(Arc<VenueOrder>),
    VenueOrderPlaced(Arc<VenueOrder>),
    VenueOrderRejected(Arc<VenueOrder>),
    VenueOrderFill(Arc<VenueOrder>),
    VenueOrderCancelled(Arc<VenueOrder>),
    VenueOrderExpired(Arc<VenueOrder>),

    // Other
    Finished(OffsetDateTime),
}

impl Event {
    pub fn timestamp(&self) -> OffsetDateTime {
        match self {
            // Market Data
            Event::TickUpdate(event) => event.event_time,
            Event::TradeUpdate(event) => event.event_time,
            Event::BookUpdate(event) => event.event_time,
            // Accounting
            Event::BalanceUpdate(event) => event.event_time,
            Event::PositionUpdate(event) => event.event_time,
            Event::AccountNew(event) => event.event_time,
            Event::TransferNew(event) => event.event_time,
            // Insights
            Event::InsightsTick(event) => event.event_time,
            Event::InsightsUpdate(event) => event.event_time,
            // Strategy Signals
            Event::SignalUpdate(event) => event.event_time,
            // Allocation Execution Orders
            Event::NewExecutionOrder(event) => event.event_time,
            Event::CancelExecutionOrder(event) => event.event_time,
            Event::CancelAllExecutionOrders(ts) => *ts,

            // Order Manager Venue Orders
            Event::NewVenueOrder(event) => event.updated_at,
            Event::CancelVenueOrder(event) => event.updated_at,
            Event::CancelAllVenueOrders(ts) => *ts,
            Event::ExecutionOrderPlaced(event) => event.event_time,
            Event::ExecutionOrderFilled(event) => event.event_time,
            Event::ExecutionOrderCancelled(event) => event.event_time,

            // Execution Venue Order Updates
            Event::VenueOrderInflight(event) => event.updated_at,
            Event::VenueOrderPlaced(event) => event.updated_at,
            Event::VenueOrderRejected(event) => event.updated_at,
            Event::VenueOrderFill(event) => event.updated_at,
            Event::VenueOrderCancelled(event) => event.updated_at,
            Event::VenueOrderExpired(event) => event.updated_at,
            // Other
            Event::Finished(ts) => *ts,
        }
    }

    pub fn is_market_data(&self) -> bool {
        matches!(
            self,
            Event::TickUpdate(_)
                | Event::TradeUpdate(_)
                | Event::BookUpdate(_)
                | Event::InsightsTick(_)
                | Event::InsightsUpdate(_)
        )
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp() == other.timestamp()
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp().cmp(&other.timestamp())
    }
}

impl Event {
    pub fn event_type(&self) -> EventType {
        self.into()
    }
}
