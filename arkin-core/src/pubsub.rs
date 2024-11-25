use std::fmt;
use std::sync::Arc;
use std::{any::Any, time::Duration};

use dashmap::DashMap;
use derive_builder::Builder;
use time::OffsetDateTime;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, error};

use strum::EnumDiscriminants;

use crate::{Book, ExecutionOrder, Fill, Holding, Insight, Instrument, Position, Signal, Tick, Trade, VenueOrder};

pub trait Event: fmt::Debug + Send + Sync + Clone + 'static {
    fn event_type() -> UpdateEventType;
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct IntervalTick {
    pub event_time: OffsetDateTime,
    pub instruments: Vec<Arc<Instrument>>,
    pub frequency: Duration,
}

impl Event for IntervalTick {
    fn event_type() -> UpdateEventType {
        UpdateEventType::IntervalTick
    }
}

impl From<IntervalTick> for UpdateEvent {
    fn from(tick: IntervalTick) -> Self {
        UpdateEvent::IntervalTick(tick)
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct InsightTick {
    pub event_time: OffsetDateTime,
    pub instruments: Vec<Arc<Instrument>>,
    pub insights: Vec<Insight>,
}

impl Event for InsightTick {
    fn event_type() -> UpdateEventType {
        UpdateEventType::InsightTick
    }
}

impl From<InsightTick> for UpdateEvent {
    fn from(tick: InsightTick) -> Self {
        UpdateEvent::InsightTick(tick)
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct SignalTick {
    pub event_time: OffsetDateTime,
    pub instruments: Vec<Arc<Instrument>>,
    pub signals: Vec<Signal>,
}

impl Event for SignalTick {
    fn event_type() -> UpdateEventType {
        UpdateEventType::SignalTick
    }
}

impl From<SignalTick> for UpdateEvent {
    fn from(tick: SignalTick) -> Self {
        UpdateEvent::SignalTick(tick)
    }
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(name(UpdateEventType))]
#[strum_discriminants(derive(Hash))]
pub enum UpdateEvent {
    IntervalTick(IntervalTick),
    Tick(Tick),
    Trade(Trade),
    Book(Book),
    Balance(Holding),
    Position(Position),
    Fill(Fill),
    Insight(Insight),
    InsightTick(InsightTick),
    Signal(Signal),
    SignalTick(SignalTick),
    ExecutionOrder(ExecutionOrder),
    VenueOrder(VenueOrder),
}

impl UpdateEvent {
    pub fn event_type(&self) -> UpdateEventType {
        self.into()
    }
}

#[derive(Debug)]
pub struct PubSub {
    pub event_senders: DashMap<UpdateEventType, Box<dyn Any + Send + Sync>>,
}

impl PubSub {
    pub fn new() -> Self {
        Self {
            event_senders: DashMap::new(),
        }
    }

    pub fn subscribe<E: Event>(&self) -> Receiver<E> {
        let event_type = E::event_type();
        let sender_any = self.event_senders.entry(event_type).or_insert_with(|| {
            let (tx, _) = broadcast::channel::<E>(1024);
            Box::new(tx)
        });
        let sender = sender_any.downcast_ref::<Sender<E>>().expect("Type mismatch");
        sender.subscribe()
    }

    pub fn publish<E: Event>(&self, event: E) {
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
