use std::sync::Arc;

use strum::{Display, EnumDiscriminants, EnumIter};
use time::UtcDateTime;

use crate::{
    Account, AccountUpdate, BalanceUpdate, Book, ExecutionOrder, InsightsTick, InsightsUpdate, PositionUpdate, Signal,
    Tick, Trade, TransferGroup, VenueOrder,
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

    AccountNew(Arc<Account>),
    TransferNew(Arc<TransferGroup>),

    // Insights
    InsightsTick(Arc<InsightsTick>),
    InsightsUpdate(Arc<InsightsUpdate>),

    // Strategy
    SignalUpdate(Arc<Signal>),

    // Allocation Execution Orders
    NewExecutionOrder(Arc<ExecutionOrder>),
    CancelExecutionOrder(Arc<ExecutionOrder>),
    CancelAllExecutionOrders(UtcDateTime),

    NewTakerExecutionOrder(Arc<ExecutionOrder>),
    CancelTakerExecutionOrder(Arc<ExecutionOrder>),
    CancelAllTakerExecutionOrders(UtcDateTime),

    NewWideQuoterExecutionOrder(Arc<ExecutionOrder>),
    CancelWideQuoterExecutionOrder(Arc<ExecutionOrder>),
    CancelAllWideQuoterExecutionOrders(UtcDateTime),

    // Order Manager

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

            Event::AccountNew(event) => event.updated_at,
            Event::TransferNew(event) => event.event_time,

            // Insights
            Event::InsightsTick(event) => event.event_time,
            Event::InsightsUpdate(event) => event.event_time,

            // Strategy Signals
            Event::SignalUpdate(event) => event.event_time,

            // Allocation Execution Orders
            Event::NewExecutionOrder(event) => event.updated_at,
            Event::CancelExecutionOrder(event) => event.updated_at,
            Event::CancelAllExecutionOrders(ts) => *ts,

            Event::NewTakerExecutionOrder(event) => event.updated_at,
            Event::CancelTakerExecutionOrder(event) => event.updated_at,
            Event::CancelAllTakerExecutionOrders(ts) => *ts,

            Event::NewWideQuoterExecutionOrder(event) => event.updated_at,
            Event::CancelWideQuoterExecutionOrder(event) => event.updated_at,
            Event::CancelAllWideQuoterExecutionOrders(ts) => *ts,

            // Order Manger

            // Execution Strategy
            Event::NewVenueOrder(event) => event.updated_at,
            Event::CancelVenueOrder(event) => event.updated_at,
            Event::CancelAllVenueOrders(ts) => *ts,
            Event::ExecutionOrderActive(event) => event.updated_at,
            Event::ExecutionOrderCompleted(event) => event.updated_at,
            Event::ExecutionOrderCancelled(event) => event.updated_at,
            Event::ExecutionOrderExpired(event) => event.updated_at,

            // Execution
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
