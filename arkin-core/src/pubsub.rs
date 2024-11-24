use std::any::Any;
use std::fmt;

use dashmap::DashMap;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::error;

use strum::EnumDiscriminants;

use crate::{Book, ExecutionOrder, Fill, Holding, Insight, Position, Signal, Tick, Trade, VenueOrder};

pub trait Event: fmt::Debug + Send + Sync + Clone + 'static {
    fn event_type() -> UpdateEventType;
}

#[derive(Debug, Clone)]
pub struct InsightTick;

impl Event for InsightTick {
    fn event_type() -> UpdateEventType {
        UpdateEventType::InsightTick
    }
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(name(UpdateEventType))]
#[strum_discriminants(derive(Hash))]
pub enum UpdateEvent {
    Tick(Tick),
    Trade(Trade),
    Book(Book),
    Balance(Holding),
    Position(Position),
    Fill(Fill),
    Insight(Insight),
    InsightTick,
    Signal(Signal),
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
        if let Some(sender_any) = self.event_senders.get(&event_type) {
            let sender = sender_any.downcast_ref::<Sender<E>>().expect("Type mismatch");
            if let Err(e) = sender.send(event) {
                error!("Failed to publish event: {:?}", e);
            }
        }
    }
}
