use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use rust_decimal::Decimal;
use time::OffsetDateTime;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, info};
use typed_builder::TypedBuilder;

use strum::EnumDiscriminants;

use crate::{
    Balance, BalanceUpdate, Book, ExecutionOrder, Insight, Instrument, Position, PositionUpdate, Signal, Tick, Trade,
    VenueOrder, VenueOrderUpdate,
};

pub trait EventTypeOf: fmt::Debug + Send + Sync + Clone + 'static {
    fn event_type() -> EventType;
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct IntervalTick {
    pub event_time: OffsetDateTime,
    pub instruments: Vec<Arc<Instrument>>,
    pub frequency: Duration,
}

impl EventTypeOf for IntervalTick {
    fn event_type() -> EventType {
        EventType::IntervalTick
    }
}

impl From<IntervalTick> for Event {
    fn from(tick: IntervalTick) -> Self {
        Event::IntervalTick(Arc::new(tick))
    }
}

impl From<Arc<IntervalTick>> for Event {
    fn from(tick: Arc<IntervalTick>) -> Self {
        Event::IntervalTick(tick)
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct InsightTick {
    pub event_time: OffsetDateTime,
    pub instruments: Vec<Arc<Instrument>>,
    pub insights: Vec<Arc<Insight>>,
}

impl EventTypeOf for InsightTick {
    fn event_type() -> EventType {
        EventType::InsightTick
    }
}

impl From<Arc<InsightTick>> for Event {
    fn from(tick: Arc<InsightTick>) -> Self {
        Event::InsightTick(tick)
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct AllocationWeight {
    pub event_time: OffsetDateTime,
    pub instrument: Arc<Instrument>,
    pub weight: Decimal,
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct AllocationTick {
    pub event_time: OffsetDateTime,
    pub allocations: Vec<Arc<AllocationWeight>>,
}

impl EventTypeOf for AllocationTick {
    fn event_type() -> EventType {
        EventType::AllocationTick
    }
}

impl From<Arc<AllocationTick>> for Event {
    fn from(tick: Arc<AllocationTick>) -> Self {
        Event::AllocationTick(tick)
    }
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(name(EventType))]
#[strum_discriminants(derive(Hash))]
pub enum Event {
    IntervalTick(Arc<IntervalTick>),
    Tick(Arc<Tick>),
    Trade(Arc<Trade>),
    Book(Arc<Book>),
    Balance(Arc<Balance>),
    BalanceUpdate(Arc<BalanceUpdate>),
    Position(Arc<Position>),
    PositionUpdate(Arc<PositionUpdate>),
    Insight(Arc<Insight>),
    InsightTick(Arc<InsightTick>),
    Signal(Arc<Signal>),
    AllocationTick(Arc<AllocationTick>),
    ExecutionOrderNew(Arc<ExecutionOrder>),
    VenueOrder(Arc<VenueOrder>),
    VenueOrderUpdate(Arc<VenueOrderUpdate>),
    Finished,
}

impl Event {
    pub fn event_type(&self) -> EventType {
        self.into()
    }
}

#[derive(Debug)]
pub struct PubSub {
    sender: Sender<Event>,
    capacity: usize,
}

impl PubSub {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            sender: tx,
            capacity,
        }
    }

    pub async fn publish<E>(&self, event: E)
    where
        E: Into<Event>,
    {
        let event: Event = event.into();
        debug!("Publishing event: {:?}", event.event_type());

        // Try to send if the channel is full back off
        loop {
            // Check if we have capacity to send
            if self.sender.len() < self.capacity {
                if let Ok(_) = self.sender.send(event.clone()) {
                    break;
                }
            } else {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        info!("New subscriber to events");
        let rx = self.sender.subscribe();
        info!("Subscriber count: {}", self.sender.receiver_count());
        rx
    }
}

// #[derive(Debug)]
// pub struct PubSub {
//     pub event_senders: DashMap<EventType, Box<dyn Any + Send + Sync>>,
// }

// impl PubSub {
//     pub fn new() -> Self {
//         Self {
//             event_senders: DashMap::new(),
//         }
//     }

//     pub fn subscribe<E: EventTypeOf>(&self) -> Receiver<Arc<E>> {
//         let event_type = E::event_type();
//         let sender_any = self.event_senders.entry(event_type).or_insert_with(|| {
//             let (tx, _) = broadcast::channel::<Arc<E>>(1000000);
//             info!("New subscriber to event: {:?}", event_type);
//             Box::new(tx)
//         });
//         let sender = sender_any.downcast_ref::<Sender<Arc<E>>>().expect("Type mismatch");
//         sender.subscribe()
//     }

//     pub fn publish<E: EventTypeOf>(&self, event: Arc<E>) {
//         let event_type = E::event_type();
//         debug!("Publishing event: {:?}", event_type);
//         if let Some(sender_any) = self.event_senders.get(&event_type) {
//             let sender = sender_any.downcast_ref::<Sender<Arc<E>>>().expect("Type mismatch");
//             // Check if we have any subscribers
//             if sender.receiver_count() == 0 {
//                 return;
//             }
//             if let Err(e) = sender.send(event) {
//                 error!("Failed to publish event: {:?}", e);
//             }
//         }
//     }
// }
