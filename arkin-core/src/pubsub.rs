use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use kanal::{AsyncReceiver, AsyncSender};
use time::OffsetDateTime;
use tokio::sync::{Barrier, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use typed_builder::TypedBuilder;

use crate::{Event, RunnableService, SimulationClock};

#[derive(Clone)]
pub struct PubSubHandle {
    pub tx: AsyncSender<Event>,
    pub rx: AsyncReceiver<(Event, Arc<Barrier>)>,
    pub rx_ack: AsyncSender<String>,
    pub clock: Arc<dyn SimulationClock>,
}

impl PubSubHandle {
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
    pub tx: AsyncSender<Event>,
    pub clock: Arc<dyn SimulationClock>,
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
    pub rx: AsyncReceiver<(Event, Arc<Barrier>)>,
    pub rx_ack: AsyncSender<String>,
    pub clock: Arc<dyn SimulationClock>,
}

impl PubSubSubscriber {
    pub async fn current_time(&self) -> OffsetDateTime {
        self.clock.get_current_time().await
    }
}

#[derive(TypedBuilder)]
pub struct PubSub {
    #[builder(default = RwLock::new(Vec::new()))]
    event_receivers: RwLock<Vec<AsyncReceiver<Event>>>,
    #[builder(default = RwLock::new(Vec::new()))]
    subscribers: RwLock<Vec<AsyncSender<(Event, Arc<Barrier>)>>>,
    #[builder(default = RwLock::new(kanal::bounded_async(1024)))]
    subscribers_acknowledge: RwLock<(AsyncSender<String>, AsyncReceiver<String>)>,
    clock: Arc<dyn SimulationClock>,
}

impl PubSub {
    pub async fn subscriber(&self) -> PubSubSubscriber {
        let (tx, rx) = kanal::bounded_async(1024);
        self.subscribers.write().await.push(tx);
        PubSubSubscriber {
            rx,
            rx_ack: self.subscribers_acknowledge.read().await.0.clone(),
            clock: self.clock.clone(),
        }
    }

    pub async fn publisher(&self) -> PubSubPublisher {
        let (tx, rx) = kanal::bounded_async(1024);
        self.event_receivers.write().await.push(rx);
        PubSubPublisher {
            tx,
            clock: self.clock.clone(),
        }
    }

    pub async fn handle(&self) -> PubSubHandle {
        let (publisher_tx, publisher_rx) = kanal::bounded_async(1024);
        let (receiver_tx, receiver_rx) = kanal::bounded_async(1024);
        self.event_receivers.write().await.push(publisher_rx);
        self.subscribers.write().await.push(receiver_tx);
        PubSubHandle {
            tx: publisher_tx,
            rx: receiver_rx,
            rx_ack: self.subscribers_acknowledge.read().await.0.clone(),
            clock: self.clock.clone(),
        }
    }

    async fn notify_subscribers(&self, event: Event) -> Arc<Barrier> {
        let subscribers = self.subscribers.read().await;
        info!("Notifying {} subscribers", subscribers.len());
        let barrier = Arc::new(Barrier::new(subscribers.len() + 1));
        for subscriber in subscribers.iter() {
            if let Err(e) = subscriber.send((event.clone(), barrier.clone())).await {
                error!("Failed to notify subscriber: {}", e);
            }
        }
        barrier
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
                    info!("Processing event: {:?}", event);
                }
                self.clock.advance_time(event.timestamp()).await;
                info!("Publishing Event: {}", event);
                let barrier = self.notify_subscribers(event.clone()).await;
                // info!("Waiting for subscribers to process event");
                // Wait max 1s for subscribers to process the event
                let timeout = tokio::time::timeout(Duration::from_secs(5), barrier.wait()).await;
                match timeout {
                    Ok(_) => {} // info!("Subscribers processed event"),
                    Err(_) => {
                        error!("Timeout waiting for subscribers to process event: {:?}", event);
                        barrier.wait().await; // Ensure barrier is released
                    }
                }
            } else {
                // Check for shutdown
                if shutdown.is_cancelled() {
                    println!("PubSub shutting down...");
                    break;
                }

                if self.clock.is_finished().await {
                    println!("PubSub finished processing all events.");
                    // Notify subscribers that processing is finished
                    let barrier = self.notify_subscribers(Event::Finished).await;
                    barrier.wait().await;
                    break;
                }

                // If it is the final day advance the clock if there are no events by 1 sec a time
                if self.clock.is_final_hour().await {
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
