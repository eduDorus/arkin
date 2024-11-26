use std::fmt;
use std::sync::Arc;
use std::{any::Any, time::Duration};

use dashmap::DashMap;
use derive_builder::Builder;
use time::OffsetDateTime;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, error};

use strum::EnumDiscriminants;

use crate::types::Commission;
use crate::{
    Book, ExecutionOrder, ExecutionOrderId, Fill, Holding, Insight, Instrument, MarketSide, Notional, Position, Price,
    Quantity, Signal, Tick, Trade, VenueOrder, VenueOrderId, VenueOrderStatus,
};

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

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct VenueOrderState {
    #[builder(default = OffsetDateTime::now_utc())]
    pub event_time: OffsetDateTime,
    pub id: VenueOrderId,
    pub status: VenueOrderStatus,
}

impl Event for VenueOrderState {
    fn event_type() -> UpdateEventType {
        UpdateEventType::VenueOrderState
    }
}

impl From<VenueOrderState> for UpdateEvent {
    fn from(update: VenueOrderState) -> Self {
        UpdateEvent::VenueOrderState(update)
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct VenueOrderFill {
    #[builder(default = OffsetDateTime::now_utc())]
    pub event_time: OffsetDateTime,
    pub id: VenueOrderId,
    pub execution_order_id: ExecutionOrderId,
    pub instrument: Arc<Instrument>,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Commission,
}

impl VenueOrderFill {
    pub fn notional(&self) -> Notional {
        match self.side {
            MarketSide::Buy => self.price * self.quantity,
            MarketSide::Sell => self.price * -self.quantity,
        }
    }

    pub fn value(&self) -> Notional {
        self.notional() + self.commission
    }
}

impl Event for VenueOrderFill {
    fn event_type() -> UpdateEventType {
        UpdateEventType::VenueOrderFill
    }
}

impl From<VenueOrderFill> for UpdateEvent {
    fn from(update: VenueOrderFill) -> Self {
        UpdateEvent::VenueOrderFill(update)
    }
}

impl fmt::Display for VenueOrderFill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "instrument: {} side: {} price: {} quantity: {}",
            self.instrument, self.side, self.price, self.quantity,
        )
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct ExecutionOrderFill {
    #[builder(default = OffsetDateTime::now_utc())]
    pub event_time: OffsetDateTime,
    pub id: ExecutionOrderId,
    pub instrument: Arc<Instrument>,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Commission,
}

impl ExecutionOrderFill {
    pub fn notional(&self) -> Notional {
        match self.side {
            MarketSide::Buy => self.price * self.quantity,
            MarketSide::Sell => self.price * -self.quantity,
        }
    }
}

impl Event for ExecutionOrderFill {
    fn event_type() -> UpdateEventType {
        UpdateEventType::ExecutionOrderFill
    }
}

impl From<ExecutionOrderFill> for UpdateEvent {
    fn from(update: ExecutionOrderFill) -> Self {
        UpdateEvent::ExecutionOrderFill(update)
    }
}

impl From<VenueOrderFill> for ExecutionOrderFill {
    fn from(fill: VenueOrderFill) -> Self {
        ExecutionOrderFill {
            event_time: fill.event_time,
            id: fill.execution_order_id,
            instrument: fill.instrument,
            side: fill.side,
            price: fill.price,
            quantity: fill.quantity,
            commission: fill.commission,
        }
    }
}

impl fmt::Display for ExecutionOrderFill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "instrument: {} side: {} price: {} quantity: {}",
            self.instrument, self.side, self.price, self.quantity,
        )
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
    ExecutionOrderFill(ExecutionOrderFill),
    VenueOrder(VenueOrder),
    VenueOrderState(VenueOrderState),
    VenueOrderFill(VenueOrderFill),
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
