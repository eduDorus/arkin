use std::fmt;
use std::sync::Arc;
use std::{any::Any, time::Duration};

use dashmap::DashMap;
use time::OffsetDateTime;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, error, info};
use typed_builder::TypedBuilder;

use strum::EnumDiscriminants;

use crate::{
    Book, ExecutionOrder, Holding, Insight, Instrument, Position, Signal, Tick, Trade, VenueOrder, VenueOrderFill,
    VenueOrderId, VenueOrderStatus,
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

pub struct SignalTick {
    pub event_time: OffsetDateTime,
    pub instruments: Vec<Arc<Instrument>>,
    pub signals: Vec<Arc<Signal>>,
}

impl EventTypeOf for SignalTick {
    fn event_type() -> EventType {
        EventType::SignalTick
    }
}

impl From<Arc<SignalTick>> for Event {
    fn from(tick: Arc<SignalTick>) -> Self {
        Event::SignalTick(tick)
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct VenueOrderState {
    #[builder(default = OffsetDateTime::now_utc())]
    pub event_time: OffsetDateTime,
    pub id: VenueOrderId,
    pub status: VenueOrderStatus,
}

impl EventTypeOf for VenueOrderState {
    fn event_type() -> EventType {
        EventType::VenueOrderState
    }
}

impl From<Arc<VenueOrderState>> for Event {
    fn from(update: Arc<VenueOrderState>) -> Self {
        Event::VenueOrderState(update)
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
    BalanceUpdate(Arc<Holding>),
    PositionUpdate(Arc<Position>),
    Insight(Arc<Insight>),
    InsightTick(Arc<InsightTick>),
    Signal(Arc<Signal>),
    SignalTick(Arc<SignalTick>),
    ExecutionOrderNew(Arc<ExecutionOrder>),
    VenueOrderNew(Arc<VenueOrder>),
    VenueOrderState(Arc<VenueOrderState>),
    VenueOrderFill(Arc<VenueOrderFill>),
}

impl Event {
    pub fn event_type(&self) -> EventType {
        self.into()
    }
}

#[derive(Debug)]
pub struct PubSub {
    pub event_senders: DashMap<EventType, Box<dyn Any + Send + Sync>>,
}

impl PubSub {
    pub fn new() -> Self {
        Self {
            event_senders: DashMap::new(),
        }
    }

    pub fn subscribe<E: EventTypeOf>(&self) -> Receiver<Arc<E>> {
        let event_type = E::event_type();
        let sender_any = self.event_senders.entry(event_type).or_insert_with(|| {
            let (tx, _) = broadcast::channel::<Arc<E>>(1000000);
            info!("New subscriber to event: {:?}", event_type);
            Box::new(tx)
        });
        let sender = sender_any.downcast_ref::<Sender<Arc<E>>>().expect("Type mismatch");
        sender.subscribe()
    }

    pub fn publish<E: EventTypeOf>(&self, event: Arc<E>) {
        let event_type = E::event_type();
        debug!("Publishing event: {:?}", event_type);
        if let Some(sender_any) = self.event_senders.get(&event_type) {
            let sender = sender_any.downcast_ref::<Sender<Arc<E>>>().expect("Type mismatch");
            if let Err(e) = sender.send(event) {
                error!("Failed to publish event: {:?}", e);
            }
        }
    }
}
