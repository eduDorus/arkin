use std::{fmt, sync::Arc};

use strum::{Display, EnumDiscriminants, EnumIter, EnumString};
use time::UtcDateTime;
use tracing::{error, warn};

use crate::{
    Account, AggTrade, Book, EventPayload, ExecutionOrder, InsightsTick, InsightsUpdate, Metric, PersistenceReader,
    Tick, Trade, Transfer, TransferBatch, VenueAccountUpdate, VenueOrder,
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
    NewAccount(Arc<Account>),
    NewTransfer(Arc<Transfer>),
    NewTransferBatch(Arc<TransferBatch>),

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
            Event::AggTradeUpdate(event) => event.event_time,
            Event::BookUpdate(event) => event.event_time,
            Event::MetricUpdate(event) => event.event_time,

            // Accounting
            Event::InitialAccountUpdate(event) => event.event_time,
            Event::ReconcileAccountUpdate(event) => event.event_time,
            Event::VenueAccountUpdate(event) => event.event_time,
            // Event::VenueTradeUpdate(event) => event.event_time,
            Event::NewAccount(event) => event.updated,
            Event::NewTransfer(event) => event.created,
            Event::NewTransferBatch(event) => event.event_time, // TODO: This is probably not optimal

            // Insights
            Event::InsightsTick(event) => event.event_time,
            Event::InsightsUpdate(event) => event.event_time,
            Event::WarmupInsightsUpdate(event) => event.event_time,

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

    pub fn to_msgpack(&self) -> Option<Vec<u8>> {
        match self {
            Event::TickUpdate(tick) => rmp_serde::to_vec(&tick.to_dto()).ok(),
            Event::AggTradeUpdate(trade) => rmp_serde::to_vec(&trade.to_dto()).ok(),
            Event::BookUpdate(book) => rmp_serde::to_vec(&book.to_dto()).ok(),
            Event::MetricUpdate(metric) => rmp_serde::to_vec(&metric.to_dto()).ok(),
            Event::InitialAccountUpdate(update)
            | Event::ReconcileAccountUpdate(update)
            | Event::VenueAccountUpdate(update) => rmp_serde::to_vec(&update.to_dto()).ok(),
            // Event::VenueTradeUpdate(update) => {
            //     let dto: VenueTradeUpdateDto = update.deref().clone().into();
            //     rmp_serde::to_vec(&dto).ok()
            // }
            Event::InsightsTick(tick) => rmp_serde::to_vec(&tick.to_dto()).ok(),
            Event::InsightsUpdate(update) | Event::WarmupInsightsUpdate(update) => {
                rmp_serde::to_vec(&update.to_dto()).ok()
            }
            Event::NewExecutionOrder(order)
            | Event::NewTakerExecutionOrder(order)
            | Event::NewWideQuoterExecutionOrder(order)
            | Event::CancelExecutionOrder(order)
            | Event::CancelTakerExecutionOrder(order)
            | Event::CancelWideQuoterExecutionOrder(order)
            | Event::ExecutionOrderActive(order)
            | Event::ExecutionOrderCompleted(order)
            | Event::ExecutionOrderCancelled(order)
            | Event::ExecutionOrderExpired(order)
            | Event::ExecutionOrderBookNew(order)
            | Event::ExecutionOrderBookUpdate(order) => rmp_serde::to_vec(&order.to_dto()).ok(),
            Event::NewVenueOrder(order)
            | Event::CancelVenueOrder(order)
            | Event::VenueOrderInflight(order)
            | Event::VenueOrderPlaced(order)
            | Event::VenueOrderRejected(order)
            | Event::VenueOrderFill(order)
            | Event::VenueOrderCancelled(order)
            | Event::VenueOrderExpired(order)
            | Event::VenueOrderBookNew(order)
            | Event::VenueOrderBookUpdate(order) => rmp_serde::to_vec(&order.to_dto()).ok(),
            Event::NewAccount(account) => rmp_serde::to_vec(&account.to_dto()).ok(),
            Event::NewTransfer(transfer) => rmp_serde::to_vec(&transfer.to_dto()).ok(),
            Event::NewTransferBatch(batch) => rmp_serde::to_vec(&batch.to_dto()).ok(),
            Event::CancelAllExecutionOrders(ts)
            | Event::CancelAllTakerExecutionOrders(ts)
            | Event::CancelAllWideQuoterExecutionOrders(ts)
            | Event::CancelAllVenueOrders(ts)
            | Event::Finished(ts) => rmp_serde::to_vec(ts).ok(),
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
            | EventType::NewTakerExecutionOrder
            | EventType::NewWideQuoterExecutionOrder
            | EventType::CancelExecutionOrder
            | EventType::CancelTakerExecutionOrder
            | EventType::CancelWideQuoterExecutionOrder
            | EventType::ExecutionOrderActive
            | EventType::ExecutionOrderCompleted
            | EventType::ExecutionOrderCancelled
            | EventType::ExecutionOrderExpired
            | EventType::ExecutionOrderBookNew
            | EventType::ExecutionOrderBookUpdate => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let order = ExecutionOrder::from_dto(dto, persistence)
                    .await
                    .map_err(|e| error!("{}", e))
                    .ok()?;
                let order = Arc::new(order);

                match event_type {
                    EventType::NewExecutionOrder => Some(Event::NewExecutionOrder(order)),
                    EventType::NewTakerExecutionOrder => Some(Event::NewTakerExecutionOrder(order)),
                    EventType::NewWideQuoterExecutionOrder => Some(Event::NewWideQuoterExecutionOrder(order)),
                    EventType::CancelExecutionOrder => Some(Event::CancelExecutionOrder(order)),
                    EventType::CancelTakerExecutionOrder => Some(Event::CancelTakerExecutionOrder(order)),
                    EventType::CancelWideQuoterExecutionOrder => Some(Event::CancelWideQuoterExecutionOrder(order)),
                    EventType::ExecutionOrderActive => Some(Event::ExecutionOrderActive(order)),
                    EventType::ExecutionOrderCompleted => Some(Event::ExecutionOrderCompleted(order)),
                    EventType::ExecutionOrderCancelled => Some(Event::ExecutionOrderCancelled(order)),
                    EventType::ExecutionOrderExpired => Some(Event::ExecutionOrderExpired(order)),
                    EventType::ExecutionOrderBookNew => Some(Event::ExecutionOrderBookNew(order)),
                    EventType::ExecutionOrderBookUpdate => Some(Event::ExecutionOrderBookUpdate(order)),
                    _ => None,
                }
            }
            EventType::NewVenueOrder
            | EventType::CancelVenueOrder
            | EventType::VenueOrderInflight
            | EventType::VenueOrderPlaced
            | EventType::VenueOrderRejected
            | EventType::VenueOrderFill
            | EventType::VenueOrderCancelled
            | EventType::VenueOrderExpired
            | EventType::VenueOrderBookNew
            | EventType::VenueOrderBookUpdate => {
                let dto = rmp_serde::from_slice(data).ok()?;
                let order = VenueOrder::from_dto(dto, persistence).await.map_err(|e| error!("{}", e)).ok()?;
                let order = Arc::new(order);

                match event_type {
                    EventType::NewVenueOrder => Some(Event::NewVenueOrder(order)),
                    EventType::CancelVenueOrder => Some(Event::CancelVenueOrder(order)),
                    EventType::VenueOrderInflight => Some(Event::VenueOrderInflight(order)),
                    EventType::VenueOrderPlaced => Some(Event::VenueOrderPlaced(order)),
                    EventType::VenueOrderRejected => Some(Event::VenueOrderRejected(order)),
                    EventType::VenueOrderFill => Some(Event::VenueOrderFill(order)),
                    EventType::VenueOrderCancelled => Some(Event::VenueOrderCancelled(order)),
                    EventType::VenueOrderExpired => Some(Event::VenueOrderExpired(order)),
                    EventType::VenueOrderBookNew => Some(Event::VenueOrderBookNew(order)),
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
            EventType::CancelAllExecutionOrders
            | EventType::CancelAllTakerExecutionOrders
            | EventType::CancelAllWideQuoterExecutionOrders
            | EventType::CancelAllVenueOrders
            | EventType::Finished => {
                let ts: UtcDateTime = rmp_serde::from_slice(data).ok()?;
                match event_type {
                    EventType::CancelAllExecutionOrders => Some(Event::CancelAllExecutionOrders(ts)),
                    EventType::CancelAllTakerExecutionOrders => Some(Event::CancelAllTakerExecutionOrders(ts)),
                    EventType::CancelAllWideQuoterExecutionOrders => {
                        Some(Event::CancelAllWideQuoterExecutionOrders(ts))
                    }
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
        write!(f, "{}: ", self.event_type())?;
        match self {
            Event::TickUpdate(inner) => write!(f, "{}", inner),
            Event::TradeUpdate(inner) => write!(f, "{}", inner),
            Event::AggTradeUpdate(inner) => write!(f, "{}", inner),
            Event::BookUpdate(inner) => write!(f, "{}", inner),
            Event::MetricUpdate(inner) => write!(f, "{}", inner),
            Event::InitialAccountUpdate(inner) => write!(f, "{}", inner),
            Event::ReconcileAccountUpdate(inner) => write!(f, "{}", inner),
            Event::VenueAccountUpdate(inner) => write!(f, "{}", inner),
            // Event::VenueTradeUpdate(inner) => write!(f, "{}", inner),
            Event::InsightsTick(inner) => write!(f, "{}", inner),
            Event::InsightsUpdate(inner) => write!(f, "{}", inner),
            Event::WarmupInsightsUpdate(inner) => write!(f, "{}", inner),
            Event::NewExecutionOrder(inner) => write!(f, "{}", inner),
            Event::NewTakerExecutionOrder(inner) => write!(f, "{}", inner),
            Event::NewWideQuoterExecutionOrder(inner) => write!(f, "{}", inner),
            Event::CancelExecutionOrder(inner) => write!(f, "{}", inner),
            Event::CancelTakerExecutionOrder(inner) => write!(f, "{}", inner),
            Event::CancelWideQuoterExecutionOrder(inner) => write!(f, "{}", inner),
            Event::CancelAllExecutionOrders(inner) => write!(f, "{}", inner),
            Event::CancelAllTakerExecutionOrders(inner) => write!(f, "{}", inner),
            Event::CancelAllWideQuoterExecutionOrders(inner) => write!(f, "{}", inner),
            Event::NewVenueOrder(inner) => write!(f, "{}", inner),
            Event::CancelVenueOrder(inner) => write!(f, "{}", inner),
            Event::CancelAllVenueOrders(inner) => write!(f, "{}", inner),
            Event::ExecutionOrderActive(inner) => write!(f, "{}", inner),
            Event::ExecutionOrderCompleted(inner) => write!(f, "{}", inner),
            Event::ExecutionOrderCancelled(inner) => write!(f, "{}", inner),
            Event::ExecutionOrderExpired(inner) => write!(f, "{}", inner),
            Event::VenueOrderInflight(inner) => write!(f, "{}", inner),
            Event::VenueOrderPlaced(inner) => write!(f, "{}", inner),
            Event::VenueOrderRejected(inner) => write!(f, "{}", inner),
            Event::VenueOrderFill(inner) => write!(f, "{}", inner),
            Event::VenueOrderCancelled(inner) => write!(f, "{}", inner),
            Event::VenueOrderExpired(inner) => write!(f, "{}", inner),
            Event::NewAccount(inner) => write!(f, "{}", inner),
            Event::NewTransfer(inner) => write!(f, "{}", inner),
            Event::NewTransferBatch(inner) => write!(f, "{}", inner),
            Event::ExecutionOrderBookNew(inner) => write!(f, "{}", inner),
            Event::ExecutionOrderBookUpdate(inner) => write!(f, "{}", inner),
            Event::VenueOrderBookNew(inner) => write!(f, "{}", inner),
            Event::VenueOrderBookUpdate(inner) => write!(f, "{}", inner),
            Event::Finished(inner) => write!(f, "{}", inner),
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
                | EventType::ExecutionOrderBookNew
                | EventType::ExecutionOrderBookUpdate
                | EventType::VenueOrderBookNew
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
                | EventType::ExecutionOrderBookNew
                | EventType::ExecutionOrderBookUpdate
                | EventType::VenueOrderBookNew
                | EventType::VenueOrderBookUpdate
        )
    }
}
