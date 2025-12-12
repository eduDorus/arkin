use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use dashmap::DashMap;
use kanal::{AsyncReceiver, AsyncSender};
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::prelude::{Publisher, Runnable, Subscriber};
use crate::{CoreCtx, Event, EventFilter, EventType, InsightsTick, PubSubTrait, ServiceCtx};

#[derive(Debug, Clone)]
pub struct PubSubPublisher {
    event_queue: Arc<Mutex<BinaryHeap<Reverse<Event>>>>,
}

#[async_trait]
impl Publisher for PubSubPublisher {
    async fn publish(&self, event: Event) {
        debug!(target: "publisher", "publishing event {}", event);
        let mut queue = self.event_queue.lock().await;
        queue.push(Reverse(event));
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
        if self.ack
            && let Err(e) = self.ack_tx.send(()).await
        {
            error!("Failed to acknowledge event: {}", e);
        }
    }
}

pub struct ChannelPubSub {
    event_queue: Arc<Mutex<BinaryHeap<Reverse<Event>>>>,
    next_id: AtomicU64,
    subscribers: DashMap<u64, AsyncSender<Event>>,
    event_subscriptions: DashMap<EventType, Vec<u64>>,
    subscribers_acknowledge: bool,
    subscribers_acknowledge_channel: (AsyncSender<()>, AsyncReceiver<()>),
}

impl ChannelPubSub {
    pub fn new(ack: bool) -> Arc<Self> {
        Self {
            event_queue: Arc::new(Mutex::new(BinaryHeap::<Reverse<Event>>::new())),
            next_id: AtomicU64::new(0),
            subscribers: DashMap::new(),
            event_subscriptions: DashMap::new(),
            subscribers_acknowledge: ack,
            subscribers_acknowledge_channel: kanal::bounded_async(10240),
        }
        .into()
    }
    fn get_next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn subscribe(&self, filter: EventFilter) -> Arc<PubSubSubscriber> {
        info!(target: "pubsub", "new subscriber");
        let (tx, rx) = kanal::bounded_async(10240);

        // Update the subscriber list
        let id = self.get_next_id();
        self.subscribers.insert(id, tx);

        // Update the event subscriptions
        let event_types = filter.event_types();
        for event in event_types {
            self.event_subscriptions.entry(event).or_default().push(id);
        }

        PubSubSubscriber {
            rx,
            ack: self.subscribers_acknowledge,
            ack_tx: self.subscribers_acknowledge_channel.0.clone(),
        }
        .into()
    }

    pub async fn publish(&self, event: Event) {
        debug!(target: "pubsub", "publishing event {}", event);
        let mut queue = self.event_queue.lock().await;
        queue.push(Reverse(event));
    }

    pub async fn publish_batch(&self, events: Vec<Event>) {
        if events.is_empty() {
            return;
        }

        let mut queue = self.event_queue.lock().await;
        queue.reserve(events.len());
        queue.extend(events.into_iter().map(Reverse));
        // let mut queue = self.event_queue.lock().await;
        // for event in events {
        //     queue.push(Reverse(event));
        // }
    }

    pub fn publisher(&self) -> Arc<PubSubPublisher> {
        info!(target: "pubsub", "new publisher");
        PubSubPublisher {
            event_queue: self.event_queue.clone(),
        }
        .into()
    }

    async fn broadcast_event(&self, event: Event) {
        let mut ack_counter = 0;
        let event_type = event.event_type();

        // Get subscriber ids (DashMap returns a ref, not a Vec)
        let subscriber_ids: Vec<u64> = self
            .event_subscriptions
            .get(&event_type)
            .map(|v| v.value().clone())
            .unwrap_or_default();

        if !event_type.is_market_data() {
            debug!(target: "pubsub", "sending event {} to {} subscribers", event_type,  subscriber_ids.len());
        }

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
            if timeout(Duration::from_millis(1000), async {
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
}

async fn event_processor_task(pubsub: Arc<ChannelPubSub>, service_ctx: Arc<ServiceCtx>, core_ctx: Arc<CoreCtx>) {
    info!(target: "pubsub", "starting event processor task");

    let shutdown = service_ctx.get_shutdown_token();
    loop {
        // In simulation mode, wait at barrier before processing each window
        // This ensures all ingestors have loaded their data before we process
        if let Some(barrier) = core_ctx.simulation_barrier.read().await.as_ref() {
            info!(target: "pubsub", "event processor waiting at barrier");
            barrier.release_ingestors().await;
            barrier.pubsub_confirm_and_wait().await;
            info!(target: "pubsub", "event processor released from barrier");
        }

        let mut processed_any = false;

        // Drain and process all available events from the queue
        loop {
            let event_opt = pubsub.event_queue.lock().await.pop();

            if let Some(Reverse(event)) = event_opt {
                processed_any = true;

                if !event.event_type().is_market_data() {
                    debug!(target: "pubsub", "processing event: {}", event);
                }

                // Advance time in simulation
                if !core_ctx.time.is_live().await {
                    debug!(target: "pubsub", "advancing time to {}", event.timestamp());
                    core_ctx.time.advance_time_to(event.timestamp()).await;
                    // Post tick events
                    let intervals = core_ctx.time.check_interval().await;
                    if !intervals.is_empty() {
                        for ts in intervals {
                            let tick = InsightsTick::builder()
                                .event_time(ts)
                                .frequency(Duration::from_secs(60))
                                .build();
                            debug!(target: "pubsub", "posting insight tick event: {}", tick.event_time);
                            pubsub.broadcast_event(Event::InsightsTick(tick.into())).await;
                        }
                    }
                }

                pubsub.broadcast_event(event).await;
            } else {
                // No more events available, break out of drain loop
                break;
            }
        }

        if shutdown.is_cancelled() {
            info!(target: "pubsub", "shutdown signal received, stopping event processor task");
            break;
        }

        if !processed_any {
            debug!(target: "pubsub", "No events processed, waiting...");
            select! {
                _ = shutdown.cancelled() => break,
                _ = tokio::time::sleep(Duration::from_millis(1)) => {},
            }
        }
    }
    info!(target: "pubsub", "event processor task has stopped");
}

#[async_trait]
impl Runnable for ChannelPubSub {
    async fn get_tasks(
        self: Arc<Self>,
        service_ctx: Arc<ServiceCtx>,
        core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        vec![Box::pin(event_processor_task(
            self.clone(),
            service_ctx.clone(),
            core_ctx.clone(),
        ))]
    }
}

#[async_trait]
impl PubSubTrait for ChannelPubSub {
    fn subscribe(&self, filter: EventFilter) -> Arc<dyn Subscriber> {
        self.subscribe(filter)
    }

    fn publisher(&self) -> Arc<dyn Publisher> {
        self.publisher()
    }
}
