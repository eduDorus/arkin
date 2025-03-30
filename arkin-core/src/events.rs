use std::sync::Arc;

use strum::{Display, EnumDiscriminants};
use time::OffsetDateTime;

use crate::{
    AccountUpdate, BalanceUpdate, Book, ExecutionOrder, InsightsTick, InsightsUpdate, PositionUpdate, Signal, Tick,
    Trade, TransferGroup, VenueOrder,
};

#[derive(Debug, Display, Clone, EnumDiscriminants)]
#[strum_discriminants(name(EventType))]
#[strum_discriminants(derive(Hash))]
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
    // Execution Orders
    ExecutionOrderNew(Arc<ExecutionOrder>),
    ExecutionOrderCancel(Arc<ExecutionOrder>),
    ExecutionOrderStatusUpdate(Arc<ExecutionOrder>),
    ExecutionOrderFillUpdate(Arc<ExecutionOrder>),
    // Venue Orders
    VenueOrderNew(Arc<VenueOrder>),
    VenueOrderCancel(Arc<VenueOrder>),
    VenueOrderStatusUpdate(Arc<VenueOrder>),
    VenueOrderFillUpdate(Arc<VenueOrder>),
    // Other
    Finished,
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
            // Execution Orders
            Event::ExecutionOrderNew(event) => event.event_time,
            Event::ExecutionOrderCancel(event) => event.event_time,
            Event::ExecutionOrderStatusUpdate(event) => event.event_time,
            Event::ExecutionOrderFillUpdate(event) => event.event_time,
            // Venue Orders
            Event::VenueOrderNew(event) => event.event_time,
            Event::VenueOrderCancel(event) => event.event_time,
            Event::VenueOrderStatusUpdate(event) => event.event_time,
            Event::VenueOrderFillUpdate(event) => event.event_time,
            // Other
            Event::Finished => OffsetDateTime::from_unix_timestamp(253402300799).unwrap(), // Year 9999
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
