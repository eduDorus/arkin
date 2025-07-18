use std::sync::Arc;

use strum::{Display, EnumDiscriminants, EnumIter};
use time::UtcDateTime;

use crate::{
    Account, AccountUpdate, BalanceUpdate, Book, ExecutionOrder, InsightsTick, InsightsUpdate, PositionUpdate, Tick,
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
    InitialAccountUpdate(Arc<AccountUpdate>),
    ReconcileAccountUpdate(Arc<AccountUpdate>),
    AccountUpdate(Arc<AccountUpdate>),

    InitialBalanceUpdate(Arc<BalanceUpdate>),
    ReconcileBalanceUpdate(Arc<BalanceUpdate>),
    BalanceUpdate(Arc<BalanceUpdate>),

    InitialPositionUpdate(Arc<PositionUpdate>),
    ReconcilePositionUpdate(Arc<PositionUpdate>),
    PositionUpdate(Arc<PositionUpdate>),

    // Insights
    InsightsTick(Arc<InsightsTick>),
    InsightsUpdate(Arc<InsightsUpdate>),

    // Allocation Execution Orders
    NewExecutionOrder(Arc<ExecutionOrder>),
    NewTakerExecutionOrder(Arc<ExecutionOrder>),
    NewWideQuoterExecutionOrder(Arc<ExecutionOrder>),

    CancelExecutionOrder(Arc<ExecutionOrder>),
    CancelTakerExecutionOrder(Arc<ExecutionOrder>),
    CancelWideQuoterExecutionOrder(Arc<ExecutionOrder>),

    CancelAllExecutionOrders(UtcDateTime),
    CancelAllTakerExecutionOrders(UtcDateTime),
    CancelAllWideQuoterExecutionOrders(UtcDateTime),

    // Execution Strategy
    NewVenueOrder(Arc<VenueOrder>),
    CancelVenueOrder(Arc<VenueOrder>),
    CancelAllVenueOrders(UtcDateTime),
    ExecutionOrderActive(Arc<ExecutionOrder>),
    ExecutionOrderCompleted(Arc<ExecutionOrder>),
    ExecutionOrderCancelled(Arc<ExecutionOrder>),
    ExecutionOrderExpired(Arc<ExecutionOrder>),

    // Execution
    VenueOrderInflight(Arc<VenueOrder>),
    VenueOrderPlaced(Arc<VenueOrder>),
    VenueOrderRejected(Arc<VenueOrder>),
    VenueOrderFill(Arc<VenueOrder>),
    VenueOrderCancelled(Arc<VenueOrder>),
    VenueOrderExpired(Arc<VenueOrder>),

    // Ledger
    AccountNew(Arc<Account>),
    TransferNew(Arc<TransferGroup>),

    // Order Book
    ExecutionOrderBookNew(Arc<ExecutionOrder>),
    ExecutionOrderBookUpdate(Arc<ExecutionOrder>),
    VenueOrderBookNew(Arc<VenueOrder>),
    VenueOrderBookUpdate(Arc<VenueOrder>),

    // Other
    Finished(UtcDateTime),
}

impl Event {
    pub fn timestamp(&self) -> UtcDateTime {
        match self {
            // Market Data
            Event::TickUpdate(event) => event.event_time,
            Event::TradeUpdate(event) => event.event_time,
            Event::BookUpdate(event) => event.event_time,

            // Accounting
            Event::InitialAccountUpdate(event) => event.event_time,
            Event::ReconcileAccountUpdate(event) => event.event_time,
            Event::AccountUpdate(event) => event.event_time,

            Event::InitialBalanceUpdate(event) => event.event_time,
            Event::ReconcileBalanceUpdate(event) => event.event_time,
            Event::BalanceUpdate(event) => event.event_time,

            Event::InitialPositionUpdate(event) => event.event_time,
            Event::ReconcilePositionUpdate(event) => event.event_time,
            Event::PositionUpdate(event) => event.event_time,

            Event::AccountNew(event) => event.updated,
            Event::TransferNew(event) => event.created,

            // Insights
            Event::InsightsTick(event) => event.event_time,
            Event::InsightsUpdate(event) => event.event_time,

            // Allocation Execution Orders
            Event::NewExecutionOrder(event) => event.updated,
            Event::NewTakerExecutionOrder(event) => event.updated,
            Event::NewWideQuoterExecutionOrder(event) => event.updated,

            Event::CancelExecutionOrder(event) => event.updated,
            Event::CancelTakerExecutionOrder(event) => event.updated,
            Event::CancelWideQuoterExecutionOrder(event) => event.updated,

            Event::CancelAllExecutionOrders(ts) => *ts,
            Event::CancelAllTakerExecutionOrders(ts) => *ts,
            Event::CancelAllWideQuoterExecutionOrders(ts) => *ts,

            // Execution Strategy
            Event::NewVenueOrder(event) => event.updated,
            Event::CancelVenueOrder(event) => event.updated,
            Event::CancelAllVenueOrders(ts) => *ts,
            Event::ExecutionOrderActive(event) => event.updated,
            Event::ExecutionOrderCompleted(event) => event.updated,
            Event::ExecutionOrderCancelled(event) => event.updated,
            Event::ExecutionOrderExpired(event) => event.updated,

            // Execution
            Event::VenueOrderInflight(event) => event.updated,
            Event::VenueOrderPlaced(event) => event.updated,
            Event::VenueOrderRejected(event) => event.updated,
            Event::VenueOrderFill(event) => event.updated,
            Event::VenueOrderCancelled(event) => event.updated,
            Event::VenueOrderExpired(event) => event.updated,

            // Order Books Updates
            Event::ExecutionOrderBookNew(event) => event.updated,
            Event::ExecutionOrderBookUpdate(event) => event.updated,
            Event::VenueOrderBookNew(event) => event.updated,
            Event::VenueOrderBookUpdate(event) => event.updated,

            // Other
            Event::Finished(ts) => *ts,
        }
    }
}

impl EventType {
    pub fn is_market_data(&self) -> bool {
        matches!(self, EventType::TickUpdate | EventType::TradeUpdate | EventType::BookUpdate)
    }

    pub fn is_persistable(&self) -> bool {
        matches!(
            self,
            EventType::TickUpdate
                | EventType::TradeUpdate
                | EventType::InsightsUpdate
                | EventType::AccountNew
                | EventType::TransferNew
                | EventType::ExecutionOrderBookNew
                | EventType::ExecutionOrderBookUpdate
                | EventType::VenueOrderBookNew
                | EventType::VenueOrderBookUpdate
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
