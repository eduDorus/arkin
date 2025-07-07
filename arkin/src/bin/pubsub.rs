use arkin_core::{Event, InsightsTick};
use kanal::{AsyncReceiver, AsyncSender};
use time::OffsetDateTime;
use tokio::{
    select,
    sync::{Barrier, RwLock},
};
use tokio_util::sync::CancellationToken;

use std::{cmp::Reverse, collections::BinaryHeap, sync::Arc, time::Duration};

pub const MESSAGE_SIZE: u64 = 1_800_000;

#[tokio::main]
pub async fn main() {
    let shutdown = CancellationToken::new();
    let pubsub = Arc::new(PubSub::new(shutdown.clone()));
    let mut component_1 = Component {
        id: 1,
        name: "Component1".to_string(),
        pubsub: pubsub.handle().await,
        shutdown: shutdown.clone(),
    };

    let mut component_2 = Component {
        id: 2,
        name: "Component2".to_string(),
        pubsub: pubsub.handle().await,
        shutdown: shutdown.clone(),
    };

    // Start the components
    let component_1_handle = tokio::spawn(async move {
        component_1.start().await;
    });

    let component_2_handle = tokio::spawn(async move {
        component_2.start().await;
    });

    // Publish events
    let handle_1 = pubsub.publisher().await;
    tokio::spawn(async move {
        let mut i = 0;
        loop {
            let event = InsightsTick {
                event_time: OffsetDateTime::now_utc() - Duration::from_secs(i),
                instruments: vec![],
                frequency: Duration::from_secs(1),
            };
            if let Err(e) = handle_1.send(event.into()).await {
                println!("Failed to send event: {e:?}");
            }
            i += 1;
            if i > MESSAGE_SIZE {
                break;
            }
        }
    });

    let handle_2 = pubsub.publisher().await;
    tokio::spawn(async move {
        let mut i = 0;
        loop {
            let event = InsightsTick {
                event_time: OffsetDateTime::now_utc() - Duration::from_secs(i),
                instruments: vec![],
                frequency: Duration::from_secs(1),
            };
            if let Err(e) = handle_2.send(event.into()).await {
                println!("Failed to send event: {e:?}");
            }
            i += 1;
            if i > MESSAGE_SIZE {
                break;
            }
        }
    });

    // Start the pubsub
    let pubsub_handle = tokio::spawn(async move {
        pubsub.start().await;
    });

    // Shutdown the components
    // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("Shutting down components...");
    // shutdown.cancel();
    println!("Waiting for components to finish...");
    let timer = std::time::Instant::now();
    let _ = tokio::join!(component_1_handle, component_2_handle);
    println!("Components have finished.");
    println!("Time taken: {:?}", timer.elapsed());
    shutdown.cancel();
    // Check if all components are shut down
    pubsub_handle.await.unwrap();
    // println!("Pubsub finished {}", pubsub_handle.is_finished());
    println!("All components have been shut down.");
}

pub struct Component {
    pub id: i64,
    pub name: String,
    pub pubsub: PubSubHandler,
    pub shutdown: CancellationToken,
}

impl Component {
    pub async fn start(&mut self) {
        let mut counter = 0;
        loop {
            select! {
                Ok((_event, barrier)) = self.pubsub.rx.recv() => {
                    counter += 1;
                    // Process the event
                    barrier.wait().await; // Notify that processing is done
                    if counter >= 2 *MESSAGE_SIZE {
                        println!("Component {} processed {} events, shutting down...", self.id, counter);
                        break;
                    }
                }
                _ = self.shutdown.cancelled() => {
                    // Handle shutdown or other conditions
                    println!("Component {} shutting down...", self.id);
                    break;
                }
            }
        }
    }
}

pub struct PubSubHandler {
    pub tx: AsyncSender<Event>,
    pub rx: AsyncReceiver<(Event, Arc<Barrier>)>,
}

pub struct PubSub {
    event_receivers: RwLock<Vec<AsyncReceiver<Event>>>,
    subscribers: RwLock<Vec<AsyncSender<(Event, Arc<Barrier>)>>>,
    shutdown: CancellationToken,
}

impl PubSub {
    pub fn new(shutdown: CancellationToken) -> Self {
        Self {
            event_receivers: RwLock::new(Vec::new()),
            subscribers: RwLock::new(Vec::new()),
            shutdown,
        }
    }

    pub async fn subscriber(&self) -> AsyncReceiver<(Event, Arc<Barrier>)> {
        let (tx, rx) = kanal::bounded_async(1024);
        self.subscribers.write().await.push(tx);
        rx
    }

    pub async fn publisher(&self) -> AsyncSender<Event> {
        let (tx, rx) = kanal::bounded_async(1024);
        self.event_receivers.write().await.push(rx);
        tx
    }

    pub async fn handle(&self) -> PubSubHandler {
        let (publisher_tx, publisher_rx) = kanal::bounded_async(1024);
        let (receiver_tx, receiver_rx) = kanal::bounded_async(1024);
        self.event_receivers.write().await.push(publisher_rx);
        self.subscribers.write().await.push(receiver_tx);
        PubSubHandler {
            tx: publisher_tx,
            rx: receiver_rx,
        }
    }

    async fn notify_subscribers(&self, event: Event) -> Arc<Barrier> {
        let subscribers = self.subscribers.read().await;
        let barrier = Arc::new(Barrier::new(subscribers.len() + 1));
        for subscriber in subscribers.iter() {
            let _ = subscriber.send((event.clone(), barrier.clone())).await;
        }
        barrier
    }

    pub async fn start(&self) {
        let mut event_queue = BinaryHeap::with_capacity(MESSAGE_SIZE as usize);
        loop {
            // Check for shutdown
            if self.shutdown.is_cancelled() {
                println!("PubSub shutting down...");
                break;
            }
            // Collect new events
            for receiver in self.event_receivers.read().await.iter() {
                // Max consume 10 events at a time
                let mut count = 0;
                while let Ok(Some(event)) = receiver.try_recv() {
                    count += 1;
                    event_queue.push(Reverse(event));
                    if count >= 10 {
                        break;
                    }
                }
            }

            if let Some(Reverse(event)) = event_queue.pop() {
                let barrier = self.notify_subscribers(event.clone()).await;
                barrier.wait().await;
            } else {
                // No events to process, sleep for a bit
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
    }
}
