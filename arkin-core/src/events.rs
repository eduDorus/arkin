use std::{fmt, sync::Arc};

use strum::{Display, EnumDiscriminants, EnumIter, EnumString};
use time::UtcDateTime;
use tracing::{error, warn};

use crate::{
    Account, AggTrade, Book, EventPayload, ExecutionOrder, InsightsTick, InsightsUpdate, Metric, PersistenceReader,
    Tick, Trade, Transfer, TransferBatch, VenueAccountUpdate, VenueOrder, VenueOrderUpdate,
};

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(name(EventType))]
#[strum_discriminants(derive(Hash), derive(Display), derive(EnumIter), derive(EnumString))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
#[strum(serialize_all = "snake_case")]
pub enum Event {
    // Market Data
    TickUpdate(Arc<Tick>),
    TradeUpdate(Arc<Trade>),
    AggTradeUpdate(Arc<AggTrade>),
    BookUpdate(Arc<Book>),
    MetricUpdate(Arc<Metric>),

    // Accounting
    InitialAccountUpdate(Arc<VenueAccountUpdate>),
    ReconcileAccountUpdate(Arc<VenueAccountUpdate>),
    VenueAccountUpdate(Arc<VenueAccountUpdate>),
    // VenueTradeUpdate(Arc<VenueTradeUpdate>),

    // Insights
    InsightsTick(Arc<InsightsTick>),
    InsightsUpdate(Arc<InsightsUpdate>),
    WarmupInsightsUpdate(Arc<InsightsUpdate>),

    // Allocation Execution Orders
    NewExecutionOrder(Arc<ExecutionOrder>),
    CancelExecutionOrder(Arc<ExecutionOrder>),
    CancelAllExecutionOrders(UtcDateTime),

    // Execution Strategy
    NewVenueOrder(Arc<VenueOrder>),
    CancelVenueOrder(Arc<VenueOrder>),
    CancelAllVenueOrders(UtcDateTime),
    ExecutionOrderActive(Arc<ExecutionOrder>),
    ExecutionOrderCompleted(Arc<ExecutionOrder>),
    ExecutionOrderCancelled(Arc<ExecutionOrder>),
    ExecutionOrderExpired(Arc<ExecutionOrder>),

    // Execution
    VenueOrderUpdate(Arc<VenueOrderUpdate>),

    // Ledger
    NewAccount(Arc<Account>),
    NewTransfer(Arc<Transfer>),
    NewTransferBatch(Arc<TransferBatch>),

    // Order Book
    ExecutionOrderBookUpdate(Arc<ExecutionOrder>),
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
            Event::AggTradeUpdate(event) => event.event_time,
            Event::BookUpdate(event) => event.event_time,
            Event::MetricUpdate(event) => event.event_time,

            // Accounting
            Event::InitialAccountUpdate(event) => event.event_time,
            Event::ReconcileAccountUpdate(event) => event.event_time,
            Event::VenueAccountUpdate(event) => event.event_time,

            Event::NewAccount(event) => event.updated,
            Event::NewTransfer(event) => event.created,
            Event::NewTransferBatch(event) => event.event_time, // TODO: This is probably not optimal

            // Insights
            Event::InsightsTick(event) => event.event_time,
            Event::InsightsUpdate(event) => event.event_time,
            Event::WarmupInsightsUpdate(event) => event.event_time,

            // Allocation Execution Orders
            Event::NewExecutionOrder(event) => event.updated,
            Event::CancelExecutionOrder(event) => event.updated,
            Event::CancelAllExecutionOrders(ts) => *ts,

            // Execution Strategy
            Event::NewVenueOrder(event) => event.updated,
            Event::CancelVenueOrder(event) => event.updated,
            Event::CancelAllVenueOrders(ts) => *ts,
            Event::ExecutionOrderActive(event) => event.updated,
            Event::ExecutionOrderCompleted(event) => event.updated,
            Event::ExecutionOrderCancelled(event) => event.updated,
            Event::ExecutionOrderExpired(event) => event.updated,

            // Execution
            Event::VenueOrderUpdate(event) => event.event_time,

            // Order Books Updates
            Event::ExecutionOrderBookUpdate(event) => event.updated,
            Event::VenueOrderBookUpdate(event) => event.updated,

            // Other
            Event::Finished(ts) => *ts,
        }
    }

    pub fn to_msgpack(&self) -> Option<Vec<u8>> {
        match self {
            Event::TickUpdate(tick) => rmp_serde::to_vec(&tick.to_dto()).ok(),
            Event::AggTradeUpdate(trade) => rmp_serde::to_vec(&trade.to_dto()).ok(),
            Event::BookUpdate(book) => rmp_serde::to_vec(&book.to_dto()).ok(),
            Event::MetricUpdate(metric) => rmp_serde::to_vec(&metric.to_dto()).ok(),
            Event::InitialAccountUpdate(update)
            | Event::ReconcileAccountUpdate(update)
            | Event::VenueAccountUpdate(update) => rmp_serde::to_vec(&update.to_dto()).ok(),
            Event::InsightsTick(tick) => rmp_serde::to_vec(&tick.to_dto()).ok(),
            Event::InsightsUpdate(update) | Event::WarmupInsightsUpdate(update) => {
                rmp_serde::to_vec(&update.to_dto()).ok()
            }
            Event::NewExecutionOrder(order)
            | Event::CancelExecutionOrder(order)
            | Event::ExecutionOrderActive(order)
            | Event::ExecutionOrderCompleted(order)
            | Event::ExecutionOrderCancelled(order)
            | Event::ExecutionOrderExpired(order)
            | Event::ExecutionOrderBookUpdate(order) => rmp_serde::to_vec(&order.to_dto()).ok(),
            Event::NewVenueOrder(order) | Event::CancelVenueOrder(order) | Event::VenueOrderBookUpdate(order) => {
                rmp_serde::to_vec(&order.to_dto()).ok()
            }
            Event::VenueOrderUpdate(update) => rmp_serde::to_vec(&update.to_dto()).ok(),
            Event::NewAccount(account) => rmp_serde::to_vec(&account.to_dto()).ok(),
            Event::NewTransfer(transfer) => rmp_serde::to_vec(&transfer.to_dto()).ok(),
            Event::NewTransferBatch(batch) => rmp_serde::to_vec(&batch.to_dto()).ok(),
            Event::CancelAllExecutionOrders(ts) | Event::CancelAllVenueOrders(ts) | Event::Finished(ts) => {
                rmp_serde::to_vec(ts).ok()
            }
            _ => {
                warn!("to_msgpack not implemented for event type: {}", self.event_type());
                None
            }
        }
    }

    pub async fn from_msgpack(
        event_type: &EventType,
        data: &[u8],
        persistence: Arc<dyn PersistenceReader>,
    ) -> Option<Self> {
        match event_type {
            EventType::TickUpdate => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let tick = Tick::from_dto(dto, persistence).await.map_err(|e| error!("{}", e)).ok()?;
                Some(Event::TickUpdate(Arc::new(tick)))
            }
            EventType::AggTradeUpdate => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let trade = AggTrade::from_dto(dto, persistence).await.map_err(|e| error!("{}", e)).ok()?;
                Some(Event::AggTradeUpdate(Arc::new(trade)))
            }
            EventType::BookUpdate => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let book = Book::from_dto(dto, persistence).await.map_err(|e| error!("{}", e)).ok()?;
                Some(Event::BookUpdate(Arc::new(book)))
            }
            EventType::MetricUpdate => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let metric = Metric::from_dto(dto, persistence).await.map_err(|e| error!("{}", e)).ok()?;
                Some(Event::MetricUpdate(Arc::new(metric)))
            }
            EventType::InitialAccountUpdate | EventType::ReconcileAccountUpdate | EventType::VenueAccountUpdate => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let update = VenueAccountUpdate::from_dto(dto, persistence)
                    .await
                    .map_err(|e| error!("{}", e))
                    .ok()?;
                let update = Arc::new(update);

                match event_type {
                    EventType::InitialAccountUpdate => Some(Event::InitialAccountUpdate(update)),
                    EventType::ReconcileAccountUpdate => Some(Event::ReconcileAccountUpdate(update)),
                    EventType::VenueAccountUpdate => Some(Event::VenueAccountUpdate(update)),
                    _ => None,
                }
            }
            EventType::VenueOrderUpdate => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let update = VenueOrderUpdate::from_dto(dto, persistence)
                    .await
                    .map_err(|e| error!("{}", e))
                    .ok()?;
                Some(Event::VenueOrderUpdate(Arc::new(update)))
            }
            EventType::InsightsTick => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let tick = InsightsTick::from_dto(dto, persistence)
                    .await
                    .map_err(|e| error!("{}", e))
                    .ok()?;
                Some(Event::InsightsTick(Arc::new(tick)))
            }
            EventType::InsightsUpdate | EventType::WarmupInsightsUpdate => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let update = InsightsUpdate::from_dto(dto, persistence)
                    .await
                    .map_err(|e| error!("{}", e))
                    .ok()?;
                let update = Arc::new(update);

                match event_type {
                    EventType::InsightsUpdate => Some(Event::InsightsUpdate(update)),
                    EventType::WarmupInsightsUpdate => Some(Event::WarmupInsightsUpdate(update)),
                    _ => None,
                }
            }
            EventType::NewExecutionOrder
            | EventType::CancelExecutionOrder
            | EventType::ExecutionOrderActive
            | EventType::ExecutionOrderCompleted
            | EventType::ExecutionOrderCancelled
            | EventType::ExecutionOrderExpired
            | EventType::ExecutionOrderBookUpdate => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let order = ExecutionOrder::from_dto(dto, persistence)
                    .await
                    .map_err(|e| error!("{}", e))
                    .ok()?;
                let order = Arc::new(order);

                match event_type {
                    EventType::NewExecutionOrder => Some(Event::NewExecutionOrder(order)),
                    EventType::CancelExecutionOrder => Some(Event::CancelExecutionOrder(order)),
                    EventType::ExecutionOrderActive => Some(Event::ExecutionOrderActive(order)),
                    EventType::ExecutionOrderCompleted => Some(Event::ExecutionOrderCompleted(order)),
                    EventType::ExecutionOrderCancelled => Some(Event::ExecutionOrderCancelled(order)),
                    EventType::ExecutionOrderExpired => Some(Event::ExecutionOrderExpired(order)),
                    EventType::ExecutionOrderBookUpdate => Some(Event::ExecutionOrderBookUpdate(order)),
                    _ => None,
                }
            }
            EventType::NewVenueOrder | EventType::CancelVenueOrder | EventType::VenueOrderBookUpdate => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let order = VenueOrder::from_dto(dto, persistence).await.map_err(|e| error!("{}", e)).ok()?;
                let order = Arc::new(order);

                match event_type {
                    EventType::NewVenueOrder => Some(Event::NewVenueOrder(order)),
                    EventType::CancelVenueOrder => Some(Event::CancelVenueOrder(order)),
                    EventType::VenueOrderBookUpdate => Some(Event::VenueOrderBookUpdate(order)),
                    _ => None,
                }
            }
            EventType::NewAccount => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let account = Account::from_dto(dto, persistence).await.map_err(|e| error!("{}", e)).ok()?;
                Some(Event::NewAccount(Arc::new(account)))
            }
            EventType::NewTransfer => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let transfer = Transfer::from_dto(dto, persistence).await.map_err(|e| error!("{}", e)).ok()?;
                Some(Event::NewTransfer(Arc::new(transfer)))
            }
            EventType::NewTransferBatch => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let batch = TransferBatch::from_dto(dto, persistence)
                    .await
                    .map_err(|e| error!("{}", e))
                    .ok()?;
                Some(Event::NewTransferBatch(Arc::new(batch)))
            }
            EventType::CancelAllExecutionOrders | EventType::CancelAllVenueOrders | EventType::Finished => {
                let ts: UtcDateTime = rmp_serde::from_slice(data).ok()?;
                match event_type {
                    EventType::CancelAllExecutionOrders => Some(Event::CancelAllExecutionOrders(ts)),
                    EventType::CancelAllVenueOrders => Some(Event::CancelAllVenueOrders(ts)),
                    EventType::Finished => Some(Event::Finished(ts)),
                    _ => None,
                }
            }
            _ => {
                warn!("from_msgpack not implemented for event type: {}", event_type);
                None
            }
        }
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

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::TickUpdate(t) => write!(f, "Tick update for {} {} / {}", t.instrument, t.bid_price, t.ask_price),
            Event::TradeUpdate(t) => {
                write!(f, "Trade update on {} {} {} @ {}", t.instrument, t.side, t.quantity, t.price)
            }
            Event::AggTradeUpdate(t) => write!(
                f,
                "Aggregated trade update on {} {} {} @ {}",
                t.instrument, t.side, t.quantity, t.price
            ),
            Event::BookUpdate(b) => write!(
                f,
                "Book update for {} bids: {} asks: {}",
                b.instrument,
                b.bids.len(),
                b.asks.len()
            ),
            Event::MetricUpdate(m) => write!(f, "Metric update for {} {}: {}", m.instrument, m.metric_type, m.value),

            Event::InitialAccountUpdate(a) => write!(f, "Initial account update on {} ({})", a.venue, a.reason),
            Event::ReconcileAccountUpdate(a) => write!(f, "Reconcile account update on {} ({})", a.venue, a.reason),
            Event::VenueAccountUpdate(a) => write!(f, "Account update on {} ({})", a.venue, a.reason),
            Event::VenueOrderUpdate(o) => write!(f, "Venue order update for {} status {}", o.id, o.status),

            Event::InsightsTick(t) => write!(f, "Insights tick frequency {:?}", t.frequency),
            Event::InsightsUpdate(i) => write!(f, "Insights update with {} insights", i.insights.len()),
            Event::WarmupInsightsUpdate(i) => write!(f, "Warmup insights update with {} insights", i.insights.len()),

            Event::NewExecutionOrder(o) => write!(
                f,
                "Created execution order to {} {} {} ({})",
                o.side, o.quantity, o.instrument, o.exec_strategy_type
            ),
            Event::CancelExecutionOrder(o) => {
                write!(f, "Cancelling execution order to {} {} {}", o.side, o.quantity, o.instrument)
            }
            Event::CancelAllExecutionOrders(ts) => write!(f, "Cancelling all execution orders at {}", ts),

            Event::NewVenueOrder(o) => write!(
                f,
                "Submitting new order to {}: {} {} {} @ {} ({})",
                o.instrument.venue, o.side, o.quantity, o.instrument, o.price, o.order_type
            ),
            Event::CancelVenueOrder(o) => write!(
                f,
                "Cancelling venue order on {} for {} {} {} @ {}",
                o.instrument.venue, o.side, o.quantity, o.instrument, o.price
            ),
            Event::CancelAllVenueOrders(ts) => write!(f, "Cancelling all venue orders at {}", ts),

            Event::ExecutionOrderActive(o) => {
                write!(f, "Execution order active for {} {} {}", o.side, o.quantity, o.instrument)
            }
            Event::ExecutionOrderCompleted(o) => {
                write!(f, "Execution order completed for {} {} {}", o.side, o.quantity, o.instrument)
            }
            Event::ExecutionOrderCancelled(o) => {
                write!(f, "Execution order cancelled for {} {} {}", o.side, o.quantity, o.instrument)
            }
            Event::ExecutionOrderExpired(o) => {
                write!(f, "Execution order expired for {} {} {}", o.side, o.quantity, o.instrument)
            }

            Event::NewAccount(a) => write!(f, "New account {} on {}", a.id, a.venue),
            Event::NewTransfer(t) => write!(f, "New transfer {}", t),
            Event::NewTransferBatch(b) => write!(f, "New transfer batch with {} transfers", b.transfers.len()),

            Event::ExecutionOrderBookUpdate(o) => write!(
                f,
                "Execution order {} {} {} is now {}",
                o.side, o.quantity, o.instrument, o.status
            ),
            Event::VenueOrderBookUpdate(o) => write!(
                f,
                "Venue order {} {} {} @ {} is now {}",
                o.side, o.quantity, o.instrument, o.price, o.status
            ),

            Event::Finished(ts) => write!(f, "Finished at {}", ts),
        }
    }
}

impl EventType {
    pub fn is_market_data(&self) -> bool {
        matches!(
            self,
            EventType::TickUpdate
                | EventType::TradeUpdate
                | EventType::BookUpdate
                | EventType::MetricUpdate
                | EventType::AggTradeUpdate
        )
    }

    pub fn is_insight(&self) -> bool {
        matches!(self, EventType::InsightsUpdate)
    }

    pub fn is_simulation(&self) -> bool {
        matches!(
            self,
            EventType::NewAccount
                | EventType::NewTransfer
                | EventType::NewTransferBatch
                | EventType::ExecutionOrderBookUpdate
                | EventType::VenueOrderBookUpdate
        )
    }

    pub fn is_persistable(&self) -> bool {
        matches!(
            self,
            EventType::TickUpdate
                | EventType::TradeUpdate
                | EventType::InsightsUpdate
                | EventType::MetricUpdate
                | EventType::NewAccount
                | EventType::NewTransfer
                | EventType::NewTransferBatch
                | EventType::ExecutionOrderBookUpdate
                | EventType::VenueOrderBookUpdate
        )
    }
}
