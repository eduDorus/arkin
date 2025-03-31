use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Duration;
use std::{cmp::Reverse, collections::HashMap};

use async_trait::async_trait;
use kanal::{AsyncReceiver, AsyncSender};
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use typed_builder::TypedBuilder;

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
    #[builder(default = RwLock::new(Vec::new()))]
    event_receivers: RwLock<Vec<AsyncReceiver<Event>>>,
    #[builder(default = RwLock::new(HashMap::new()))]
    subscribers: RwLock<HashMap<String, AsyncSender<Event>>>,
    #[builder(default = RwLock::new(kanal::bounded_async(1)))]
    subscribers_acknowledge: RwLock<(AsyncSender<()>, AsyncReceiver<()>)>,
    clock: Arc<dyn SimulationClock>,
}

impl PubSub {
    pub async fn subscriber(&self, name: &str) -> PubSubSubscriber {
        let (tx, rx) = kanal::bounded_async(100000);
        self.subscribers.write().await.insert(name.to_string(), tx);
        PubSubSubscriber {
            rx,
            rx_ack: self.subscribers_acknowledge.read().await.0.clone(),
            clock: self.clock.clone(),
        }
    }

    pub async fn publisher(&self) -> PubSubPublisher {
        let (tx, rx) = kanal::bounded_async(100000);
        self.event_receivers.write().await.push(rx);
        PubSubPublisher {
            tx,
            clock: self.clock.clone(),
        }
    }

    pub async fn handle(&self, name: &str) -> PubSubHandle {
        let (publisher_tx, publisher_rx) = kanal::bounded_async(100000);
        let (receiver_tx, receiver_rx) = kanal::bounded_async(100000);
        self.event_receivers.write().await.push(publisher_rx);
        self.subscribers.write().await.insert(name.to_string(), receiver_tx);
        PubSubHandle {
            tx: publisher_tx,
            rx: receiver_rx,
            rx_ack: self.subscribers_acknowledge.read().await.0.clone(),
            clock: self.clock.clone(),
        }
    }

    async fn notify_subscribers(&self, event: Event) {
        let subscribers = self.subscribers.read().await;
        // info!("Notifying {} subscribers", subscribers.len());
        for (_name, tx) in subscribers.iter() {
            // info!("Notifying subscriber: {}", name);
            if let Err(e) = tx.send(event.clone()).await {
                error!("Failed to notify subscriber: {}", e);
            }
        }
        let mut subscribers_ack_count = subscribers.len();
        loop {
            let ack = self.subscribers_acknowledge.read().await.1.recv().await;
            if let Ok(()) = ack {
                // info!("Subscriber acknowledged event: {}", ack);
                subscribers_ack_count -= 1;
                if subscribers_ack_count == 0 {
                    break;
                }
            }
        }
    }

    async fn notify_subscribers_no_ack(&self, event: Event) {
        let subscribers = self.subscribers.read().await;
        // info!("Notifying {} subscribers", subscribers.len());
        for (name, tx) in subscribers.iter() {
            info!("Notifying subscriber no ack: {}", name);
            if let Err(e) = tx.send(event.clone()).await {
                error!("Failed to notify subscriber: {}", e);
            }
        }
    }
}

#[async_trait]
impl RunnableService for PubSub {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        let mut event_queue = BinaryHeap::new();
        loop {
            // Collect new events
            for receiver in self.event_receivers.read().await.iter() {
                while let Ok(Some(event)) = receiver.try_recv() {
                    event_queue.push(Reverse(event));
                }
            }

            if let Some(Reverse(event)) = event_queue.pop() {
                if !event.is_market_data() {
                    info!("Processing event: {}", event);
                }
                self.clock.advance_time(event.timestamp()).await;
                self.notify_subscribers(event.clone()).await;
            } else {
                // Check for shutdown
                if shutdown.is_cancelled() {
                    info!("PubSub shutting down...");
                    self.notify_subscribers_no_ack(Event::Finished).await;
                    break;
                }

                if self.clock.is_finished().await {
                    info!("PubSub finished processing all events.");
                    // Notify subscribers that processing is finished
                    self.notify_subscribers_no_ack(Event::Finished).await;
                    break;
                }

                // If it is the final day advance the clock if there are no events by 1 sec a time
                if self.clock.is_final_hour().await {
                    info!("Final hour reached, advancing clock by 1 second.");
                    let current_time = self.clock.get_current_time().await;
                    let next_time = current_time + Duration::from_secs(1);
                    self.clock.advance_time(next_time).await;
                }

                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
        info!("PubSub service stopped.");
        Ok(())
    }
}
