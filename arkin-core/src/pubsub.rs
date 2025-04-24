use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Duration;
use std::{cmp::Reverse, collections::HashMap};

use async_trait::async_trait;
use kanal::{AsyncReceiver, AsyncSender};
use time::OffsetDateTime;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use typed_builder::TypedBuilder;

use crate::utils::PeekableReceiver;
use crate::{Event, RunnableService, SimulationClock};

#[derive(Clone)]
pub struct PubSubHandle {
    tx: AsyncSender<Event>,
    rx: AsyncReceiver<Event>,
    rx_ack: AsyncSender<()>,
    clock: Arc<dyn SimulationClock>,
}

impl PubSubHandle {
    pub async fn recv(&self) -> Option<Event> {
        match self.rx.recv().await {
            Ok(event) => Some(event),
            Err(e) => {
                error!("Failed to receive event: {}", e);
                None
            }
        }
    }

    pub async fn ack(&self) {
        if let Err(e) = self.rx_ack.send(()).await {
            error!("Failed to acknowledge event: {}", e);
        }
    }

    pub async fn publish<E>(&self, event: E)
    where
        E: Into<Event>,
    {
        let event = event.into();
        if let Err(e) = self.tx.send(event).await {
            error!("Failed to publish event: {}", e);
        }
    }

    pub async fn current_time(&self) -> OffsetDateTime {
        self.clock.get_current_time().await
    }
}

#[derive(Clone)]
pub struct PubSubPublisher {
    tx: AsyncSender<Event>,
    clock: Arc<dyn SimulationClock>,
}

impl PubSubPublisher {
    pub async fn publish<E>(&self, event: E)
    where
        E: Into<Event>,
    {
        let event = event.into();
        if let Err(e) = self.tx.send(event).await {
            error!("Failed to publish event: {}", e);
        }
    }

    pub async fn current_time(&self) -> OffsetDateTime {
        self.clock.get_current_time().await
    }
}

#[derive(Clone)]
pub struct PubSubSubscriber {
    rx: AsyncReceiver<Event>,
    rx_ack: AsyncSender<()>,
    clock: Arc<dyn SimulationClock>,
}

impl PubSubSubscriber {
    pub async fn recv(&self) -> Option<Event> {
        match self.rx.recv().await {
            Ok(event) => Some(event),
            Err(e) => {
                error!("Failed to receive event: {}", e);
                None
            }
        }
    }

    pub async fn ack(&self) {
        if let Err(e) = self.rx_ack.send(()).await {
            error!("Failed to acknowledge event: {}", e);
        }
    }

    pub async fn current_time(&self) -> OffsetDateTime {
        self.clock.get_current_time().await
    }
}

#[derive(TypedBuilder)]
pub struct PubSub {
    #[builder(default = Arc::new(RwLock::new(Vec::new())))]
    event_receivers: Arc<RwLock<Vec<PeekableReceiver<Event>>>>,
    #[builder(default = Arc::new(RwLock::new(HashMap::new())))]
    subscribers: Arc<RwLock<HashMap<String, AsyncSender<Event>>>>,
    #[builder(default = Arc::new(RwLock::new(kanal::bounded_async(128))))]
    subscribers_acknowledge: Arc<RwLock<(AsyncSender<()>, AsyncReceiver<()>)>>,
    clock: Arc<dyn SimulationClock>,
}

impl PubSub {
    pub async fn subscriber(&self, name: &str) -> PubSubSubscriber {
        let (tx, rx) = kanal::bounded_async(1);
        self.subscribers.write().await.insert(name.to_string(), tx);
        PubSubSubscriber {
            rx,
            rx_ack: self.subscribers_acknowledge.read().await.0.clone(),
            clock: self.clock.clone(),
        }
    }

    pub async fn publisher(&self) -> PubSubPublisher {
        let (tx, rx) = kanal::bounded_async(100000);
        self.event_receivers.write().await.push(PeekableReceiver::new(rx));
        PubSubPublisher {
            tx,
            clock: self.clock.clone(),
        }
    }

    pub async fn handle(&self, name: &str) -> PubSubHandle {
        let (publisher_tx, publisher_rx) = kanal::bounded_async(100000);
        let (receiver_tx, receiver_rx) = kanal::bounded_async(1);
        self.event_receivers.write().await.push(PeekableReceiver::new(publisher_rx));
        self.subscribers.write().await.insert(name.to_string(), receiver_tx);
        PubSubHandle {
            tx: publisher_tx,
            rx: receiver_rx,
            rx_ack: self.subscribers_acknowledge.read().await.0.clone(),
            clock: self.clock.clone(),
        }
    }
}

#[async_trait]
impl RunnableService for PubSub {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        // Shared event queue
        let event_queue = Arc::new(Mutex::new(BinaryHeap::<Reverse<Event>>::new()));

        // Event Collector Task
        let collector_event_queue = event_queue.clone();
        let collector_event_receivers = self.event_receivers.clone();
        let collector_clock = self.clock.clone();
        let collector_shutdown = shutdown.clone();
        tokio::spawn(async move {
            loop {
                if collector_shutdown.is_cancelled() {
                    break;
                }
                let mut receivers = collector_event_receivers.write().await;
                for receiver in receivers.iter_mut() {
                    // Peek if there is a element and if it is within 24h
                    if let Some(peeked) = receiver.peek() {
                        if peeked.timestamp() > collector_clock.get_current_time().await + Duration::from_secs(86400) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                    if let Some(event) = receiver.next() {
                        let mut lock = collector_event_queue.lock().await;
                        lock.push(Reverse(event));
                        if lock.len() > 100000000 {
                            drop(lock);
                            error!("Event queue is full, waiting 5s");
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
                // tokio::time::sleep(Duration::from_micros(1)).await;
            }
            info!("Event collector task stopped.");
        });

        // Event Processor Task
        let processor_clock = self.clock.clone();
        let processor_subscribers = self.subscribers.clone();
        let processor_subscribers_acknowledge = self.subscribers_acknowledge.clone();
        let processor_event_queue = event_queue.clone();
        let processor_shutdown = shutdown.clone();
        tokio::spawn(async move {
            loop {
                if processor_shutdown.is_cancelled() {
                    let subscribers = processor_subscribers.read().await;
                    for (_name, tx) in subscribers.iter() {
                        if let Err(e) = tx.send(Event::Finished).await {
                            error!("Failed to notify subscriber: {}", e);
                        }
                    }
                    break;
                }
                if let Some(Reverse(event)) = processor_event_queue.lock().await.pop() {
                    if !event.is_market_data() {
                        info!("Processing event: {}", event);
                    }
                    if !processor_clock.is_live().await {
                        processor_clock.advance_time(event.timestamp()).await;
                    }
                    let subscribers = processor_subscribers.read().await;
                    for (_name, tx) in subscribers.iter() {
                        if let Err(e) = tx.send(event.clone()).await {
                            error!("Failed to notify subscriber: {}", e);
                        }
                    }
                    let mut subscribers_ack_count = subscribers.len();
                    loop {
                        let ack = processor_subscribers_acknowledge.read().await.1.recv().await;
                        if let Ok(()) = ack {
                            subscribers_ack_count -= 1;
                            if subscribers_ack_count == 0 {
                                break;
                            }
                        }
                    }
                } else {
                    if processor_clock.is_finished().await {
                        info!("PubSub finished processing all events.");
                        let subscribers = processor_subscribers.read().await;
                        for (_name, tx) in subscribers.iter() {
                            if let Err(e) = tx.send(Event::Finished).await {
                                error!("Failed to notify subscriber: {}", e);
                            }
                        }
                        break;
                    }
                    if processor_clock.is_final_hour().await {
                        info!("Final hour reached, advancing clock by 1 second.");
                        let current_time = processor_clock.get_current_time().await;
                        let next_time = current_time + Duration::from_secs(1);
                        processor_clock.advance_time(next_time).await;
                    }
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
            info!("Event processor task stopped.");
        });

        // Wait for shutdown signal
        shutdown.cancelled().await;
        info!("PubSub service stopped.");
        Ok(())
    }
}
