use std::fmt;
use std::sync::Arc;
use std::{any::Any, time::Duration};

use dashmap::DashMap;
use derive_builder::Builder;
use time::OffsetDateTime;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, error};

use strum::EnumDiscriminants;

use crate::{
    Book, ExecutionOrder, Holding, Insight, Instrument, Position, Signal, Tick, Trade, VenueOrder, VenueOrderFill,
    VenueOrderId, VenueOrderStatus,
};

pub trait EventTypeOf: fmt::Debug + Send + Sync + Clone + 'static {
    fn event_type() -> EventType;
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
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
        Event::IntervalTick(tick)
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct InsightTick {
    pub event_time: OffsetDateTime,
    pub instruments: Vec<Arc<Instrument>>,
    pub insights: Vec<Insight>,
}

impl EventTypeOf for InsightTick {
    fn event_type() -> EventType {
        EventType::InsightTick
    }
}

impl From<InsightTick> for Event {
    fn from(tick: InsightTick) -> Self {
        Event::InsightTick(tick)
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct SignalTick {
    pub event_time: OffsetDateTime,
    pub instruments: Vec<Arc<Instrument>>,
    pub signals: Vec<Signal>,
}

impl EventTypeOf for SignalTick {
    fn event_type() -> EventType {
        EventType::SignalTick
    }
}

impl From<SignalTick> for Event {
    fn from(tick: SignalTick) -> Self {
        Event::SignalTick(tick)
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
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

impl From<VenueOrderState> for Event {
    fn from(update: VenueOrderState) -> Self {
        Event::VenueOrderState(update)
    }
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(name(EventType))]
#[strum_discriminants(derive(Hash))]
pub enum Event {
    IntervalTick(IntervalTick),
    Tick(Tick),
    Trade(Trade),
    Book(Book),
    BalanceUpdate(Holding),
    PositionUpdate(Position),
    Insight(Insight),
    InsightTick(InsightTick),
    Signal(Signal),
    SignalTick(SignalTick),
    ExecutionOrderNew(ExecutionOrder),
    VenueOrderNew(VenueOrder),
    VenueOrderState(VenueOrderState),
    VenueOrderFill(VenueOrderFill),
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

    pub fn subscribe<E: EventTypeOf>(&self) -> Receiver<E> {
        let event_type = E::event_type();
        let sender_any = self.event_senders.entry(event_type).or_insert_with(|| {
            let (tx, _) = broadcast::channel::<E>(1024);
            Box::new(tx)
        });
        let sender = sender_any.downcast_ref::<Sender<E>>().expect("Type mismatch");
        sender.subscribe()
    }

    pub fn publish<E: EventTypeOf>(&self, event: E) {
        let event_type = E::event_type();
        debug!("Publishing event: {:?}", event_type);
        if let Some(sender_any) = self.event_senders.get(&event_type) {
            let sender = sender_any.downcast_ref::<Sender<E>>().expect("Type mismatch");
            if let Err(e) = sender.send(event) {
                error!("Failed to publish event: {:?}", e);
            }
        }
    }
}
