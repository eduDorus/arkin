use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use dashmap::DashMap;
use kanal::{AsyncReceiver, AsyncSender};
use strum::{IntoDiscriminant, IntoEnumIterator};
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::{debug, error, info, instrument, warn};

use crate::utils::PeekableReceiver;
use crate::{Event, EventType, InsightsTick, Publisher, Runnable, ServiceCtx, Subscriber, SystemTime};

pub enum EventFilter {
    All,
    AllWithoutMarket,
    Persistable,
    PersistableNoMarket,
    Insights,
    Events(Vec<EventType>),
}

#[derive(Debug, Clone)]
pub struct PubSubPublisher {
    tx: AsyncSender<Event>,
}

#[async_trait]
impl Publisher for PubSubPublisher {
    async fn publish(&self, event: Event) {
        debug!(target: "publisher", "publishing event {}", event);
        if let Err(e) = self.tx.send(event.into()).await {
            error!("Failed to publish event: {}", e);
        }
    }
}

#[derive(Debug, Clone)]
pub struct PubSubSubscriber {
    rx: AsyncReceiver<Event>,
    ack: bool,
    ack_tx: AsyncSender<()>,
}

#[async_trait]
impl Subscriber for PubSubSubscriber {
    async fn recv(&self) -> Option<Event> {
        match self.rx.recv().await {
            Ok(event) => Some(event),
            Err(e) => {
                error!("Failed to receive event: {}", e);
                None
            }
        }
    }

    fn needs_ack(&self) -> bool {
        self.ack
    }

    async fn send_ack(&self) {
        if self.ack {
            if let Err(e) = self.ack_tx.send(()).await {
                error!("Failed to acknowledge event: {}", e);
            }
        }
    }
}

pub struct PubSub {
    time: Arc<dyn SystemTime>,
    event_queue: Mutex<BinaryHeap<Reverse<Event>>>,
    publishers: DashMap<u64, PeekableReceiver<Event>>,
    next_id: AtomicU64,
    subscribers: DashMap<u64, AsyncSender<Event>>,
    event_subscriptions: DashMap<EventType, Vec<u64>>,
    subscribers_acknowledge: bool,
    subscribers_acknowledge_channel: (AsyncSender<()>, AsyncReceiver<()>),
}

impl PubSub {
    pub fn new(time: Arc<dyn SystemTime>, ack: bool) -> Arc<Self> {
        Self {
            time: time,
            event_queue: Mutex::new(BinaryHeap::<Reverse<Event>>::new()),
            publishers: DashMap::new(),
            next_id: AtomicU64::new(0),
            subscribers: DashMap::new(),
            event_subscriptions: DashMap::new(),
            subscribers_acknowledge: ack,
            subscribers_acknowledge_channel: kanal::bounded_async(1024),
        }
        .into()
    }
    fn get_next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn subscribe(&self, filter: EventFilter) -> Arc<PubSubSubscriber> {
        info!(target: "pubsub", "new subscriber");
        let (tx, rx) = kanal::bounded_async(1);

        // Update the subscriber list
        let id = self.get_next_id();
        self.subscribers.insert(id, tx);

        // Update the event subscriptions
        match filter {
            EventFilter::All => {
                for event_type in EventType::iter() {
                    self.event_subscriptions.entry(event_type).or_default().push(id);
                }
            }
            EventFilter::AllWithoutMarket => {
                for event_type in EventType::iter() {
                    if !event_type.is_market_data() {
                        self.event_subscriptions.entry(event_type).or_default().push(id);
                    }
                }
            }
            EventFilter::Persistable => {
                for event_type in EventType::iter() {
                    if event_type.is_persistable() {
                        self.event_subscriptions.entry(event_type).or_default().push(id);
                    }
                }
            }
            EventFilter::PersistableNoMarket => {
                for event_type in EventType::iter() {
                    if event_type.is_persistable_no_market() {
                        self.event_subscriptions.entry(event_type).or_default().push(id);
                    }
                }
            }
            EventFilter::Insights => {
                for event_type in EventType::iter() {
                    if event_type.is_insight() {
                        self.event_subscriptions.entry(event_type).or_default().push(id);
                    }
                }
            }
            EventFilter::Events(events) => {
                for event_type in events {
                    self.event_subscriptions.entry(event_type).or_default().push(id);
                }
            }
        }

        PubSubSubscriber {
            rx,
            ack: self.subscribers_acknowledge,
            ack_tx: self.subscribers_acknowledge_channel.0.clone(),
        }
        .into()
    }

    pub fn publisher(&self) -> Arc<PubSubPublisher> {
        info!(target: "pubsub", "new publisher");
        let (tx, rx) = kanal::bounded_async(100000);

        // Update publisher list
        let id = self.get_next_id();
        self.publishers.insert(id, PeekableReceiver::new(rx));
        PubSubPublisher { tx }.into()
    }

    async fn broadcast_event(&self, event: Event) {
        let mut ack_counter = 0;
        let event_type = event.discriminant();

        // Get subscriber ids (DashMap returns a ref, not a Vec)
        let subscriber_ids: Vec<u64> = self
            .event_subscriptions
            .get(&event_type)
            .map(|v| v.value().clone())
            .unwrap_or_default();
        debug!(target: "pubsub", "sending event {} to {} subscribers", event_type,  subscriber_ids.len());

        // Send to subscribers and check for closed connections (DashMap version)
        let mut to_remove = Vec::new();
        for id in subscriber_ids {
            if let Some(sender) = self.subscribers.get(&id) {
                match sender.try_send(event.clone()) {
                    Ok(_) => ack_counter += 1,
                    Err(_) => {
                        info!(target: "pubsub", "subscriber closed connection, will be removed");
                        to_remove.push(id);
                    }
                }
            } else {
                to_remove.push(id);
            }
        }

        // Remove dead subscribers (DashMap is already thread-safe)
        if !to_remove.is_empty() {
            for id in to_remove {
                info!(target: "pubsub", "subscriber {} disconnected, removing...", id);
                self.subscribers.remove(&id);
                for mut entry in self.event_subscriptions.iter_mut() {
                    let ids = entry.value_mut();
                    if let Some(pos) = ids.iter().position(|&x| x == id) {
                        ids.swap_remove(pos);
                    }
                }
            }
        }

        // Wait for acknowledgements
        if self.subscribers_acknowledge {
            let mut ack_received = 0;
            if timeout(Duration::from_secs(1), async {
                while ack_received < ack_counter {
                    debug!(target: "pubsub", "{} waiting for ack ({}/{})", event_type, ack_received, ack_counter);
                    if let Ok(_) = self.subscribers_acknowledge_channel.1.recv().await {
                        ack_received += 1;
                        debug!(target: "pubsub", "{} received ack ({}/{})", event_type, ack_received, ack_counter);
                    } else {
                        warn!(target: "pubsub", "ack channel closed early, missing {}/{} acks for {}, moving on", ack_counter - ack_received, ack_counter, event);
                        break;
                    }
                }
            })
            .await
            .is_err()
            {
                warn!(target: "pubsub", "timeout waiting {} for ack {} of {}, moving on", event, ack_received, ack_counter);
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn event_collector_task(&self, ctx: Arc<ServiceCtx>) {
        info!(target: "pubsub", "starting event collector task");
        while ctx.is_running().await {
            let mut collected_any = false;
            for mut receiver in self.publishers.iter_mut() {
                // Peek if there is a element and if it is within 24h
                if let Some(peeked) = receiver.value_mut().peek() {
                    debug!(target: "pubsub", "found event");
                    // TODO: This is not optimal
                    if peeked.timestamp() > self.time.now().await + Duration::from_secs(86400) {
                        debug!(target: "pubsub", "timestamp to big, we continue");
                        continue;
                    }
                } else {
                    debug!(target: "pubsub", "No events we continue");
                    continue;
                }
                if let Some(event) = receiver.take() {
                    collected_any = true;

                    let mut lock = self.event_queue.lock().await;
                    lock.push(Reverse(event));
                    if lock.len() > 10000000 {
                        drop(lock);
                        warn!(target: "pubsub", "event queue is full, waiting 1s");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
            if !collected_any {
                debug!(target: "pubsub", "No events collected, waiting...");
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
        info!(target: "pubsub", "event collector task has stopped");
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn event_processor_task(&self, ctx: Arc<ServiceCtx>) {
        info!(target: "pubsub", "starting event processor task");
        while ctx.is_running().await {
            // let length = self.event_queue.lock().await.len();
            // info!(target: "pubsub", "event queue length: {}", length);
            if let Some(Reverse(event)) = self.event_queue.lock().await.pop() {
                //  I think we don't need this anymore
                if !event.event_type().is_market_data() {
                    info!(target: "pubsub", "processing event: {}", event);
                }

                // Advance time in simulation
                if !self.time.is_live().await {
                    debug!(target: "pubsub", "advancing time to {}", event.timestamp());
                    self.time.advance_time_to(event.timestamp()).await;
                }

                // Post tick events
                let intervals = self.time.check_interval().await;
                if !intervals.is_empty() {
                    for ts in intervals {
                        let tick = InsightsTick::builder()
                            .event_time(ts)
                            .frequency(Duration::from_secs(60))
                            .build();
                        let tick_event = Event::InsightsTick(tick.into());
                        self.broadcast_event(tick_event).await;
                    }
                }
                self.broadcast_event(event).await;
            } else {
                debug!(target: "pubsub", "No events processed, waiting...");
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
        info!(target: "pubsub", "event processor task has stopped");
    }
}

#[async_trait]
impl Runnable for PubSub {
    fn identifier(&self) -> &str {
        "pubsub"
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn start_tasks(self: Arc<Self>, ctx: Arc<ServiceCtx>) {
        info!(target: "pubsub", "starting tasks");
        let exec = self.clone();
        let ctx_clone = ctx.clone();
        ctx.spawn(async move { exec.event_collector_task(ctx_clone).await });

        let exec = self.clone();
        let ctx_clone = ctx.clone();
        ctx.spawn(async move { exec.event_processor_task(ctx_clone).await });
        info!(target: "pubsub", "tasks started");
    }
}
