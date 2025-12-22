use std::str::FromStr;
use std::{sync::Arc, time::Duration};

use anyhow::Result;

use async_trait::async_trait;
use kanal::{AsyncReceiver, AsyncSender};
use redis::{aio::ConnectionManager, Client, PushInfo, PushKind, Value};
use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{
    CoreCtx, Event, EventFilter, EventType, PersistenceReader, PubSubTrait, Publisher, Runnable, ServiceCtx, Subscriber,
};

const CONN_COUNT: usize = 1;

#[derive(Clone)]
pub struct RedisPubSub {
    client: Client,
    pub_tx: AsyncSender<Event>,
    persistence: Arc<dyn PersistenceReader>,
}

impl RedisPubSub {
    pub fn new(persistence: Arc<dyn PersistenceReader>) -> Result<Arc<Self>> {
        let client = Client::open("redis://100.104.175.55/?protocol=resp3")?;

        // Dedicated pub task: One shared con, fire-and-forget loop
        let (pub_tx, pub_rx) = kanal::unbounded_async::<Event>(); // Tune buffer; drop on full

        for _ in 0..CONN_COUNT {
            let client_clone = client.clone();
            let rx = pub_rx.clone();
            tokio::spawn(async move {
                let config = redis::aio::ConnectionManagerConfig::new()
                    .set_min_delay(Duration::from_millis(100))
                    .set_max_delay(Duration::from_secs(10))
                    .set_number_of_retries(0)
                    .set_response_timeout(Some(Duration::from_secs(1)))
                    .set_connection_timeout(Some(Duration::from_secs(5)));

                let mut manager = match ConnectionManager::new_with_config(client_clone, config).await {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Initial Redis manager connect failed: {}", e);
                        return; // Or retry loop here
                    }
                };

                let mut buffer = Vec::with_capacity(1000);
                loop {
                    // Wait for the first event
                    match rx.recv().await {
                        Ok(event) => buffer.push(event),
                        Err(_) => break, // Channel closed
                    }

                    // Try to fill buffer with available events up to 1000
                    while buffer.len() < 1000 {
                        match rx.try_recv() {
                            Ok(Some(event)) => buffer.push(event),
                            Ok(None) => break, // Empty
                            Err(_) => break,   // Closed
                        }
                    }

                    let mut pipe = redis::pipe();
                    let mut count = 0;
                    for event in buffer.drain(..) {
                        let Some(data) = event.to_msgpack() else {
                            warn!("Failed to serialize event for Redis: {}", event);
                            continue;
                        };
                        let channel = event.event_type().to_string();
                        pipe.publish(channel, data);
                        count += 1;
                    }

                    if count > 0 {
                        if let Err(e) = pipe.query_async::<()>(&mut manager).await {
                            warn!("Redis pipeline publish error: {}", e);
                        }
                    }
                }
            });
        }

        Ok(Arc::new(Self {
            client,
            pub_tx,
            persistence,
        }))
    }

    pub async fn publish(&self, event: Event) {
        debug!("Publishing to Redis: {}", event);
        if let Err(e) = self.pub_tx.try_send(event) {
            warn!("Redis pubsub publish buffer full, dropping event: {}", e);
        }
    }

    pub fn subscribe(&self, filter: EventFilter) -> Arc<dyn Subscriber> {
        let (tx, rx) = kanal::unbounded_async();
        let client = self.client.clone();
        let persistence = self.persistence.clone();
        let event_types: Vec<String> = filter.event_types().iter().map(|et| et.to_string()).collect();
        info!(target: "redis_pubsub", "Subscribing to Redis channels: {:?}", event_types);
        tokio::spawn(async move {
            loop {
                let (ptx, mut prx) = mpsc::unbounded_channel();
                let config = redis::AsyncConnectionConfig::new().set_push_sender(ptx);

                match client.get_multiplexed_async_connection_with_config(&config).await {
                    Ok(mut sub_con) => {
                        if let Err(e) = sub_con.subscribe(event_types.clone()).await {
                            error!("Redis subscribe failed: {}", e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }

                        info!("Redis subscribed to channels");

                        while let Some(push) = prx.recv().await {
                            match push {
                                PushInfo {
                                    kind: PushKind::Message,
                                    data,
                                } => {
                                    if data.len() >= 2 {
                                        if let Value::BulkString(bytes) = &data[1] {
                                            let channel = if let Value::BulkString(ch) = &data[0] {
                                                std::str::from_utf8(ch).unwrap_or("")
                                            } else {
                                                ""
                                            };
                                            let event_type = match EventType::from_str(channel) {
                                                Ok(et) => et,
                                                Err(_) => {
                                                    warn!("Unknown event type: {}", channel);
                                                    continue;
                                                }
                                            };
                                            if let Some(event) =
                                                Event::from_msgpack(&event_type, bytes, persistence.clone()).await
                                            {
                                                debug!("Received: {}", event);
                                                let _ = tx.send(event).await;
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        warn!("Redis subscription stream ended, reconnecting...");
                    }
                    Err(e) => {
                        error!("Redis connection failed: {}", e);
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
        Arc::new(RedisSubscriber { rx })
    }

    pub fn publisher(&self) -> Arc<RedisPublisher> {
        Arc::new(RedisPublisher {
            pubsub: Arc::new(self.clone()),
        })
    }
}

#[async_trait]
impl Runnable for RedisPubSub {
    async fn get_tasks(
        self: Arc<Self>,
        _service_ctx: Arc<ServiceCtx>,
        _core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        vec![]
    }
}

pub struct RedisPublisher {
    pubsub: Arc<RedisPubSub>,
}

#[async_trait]
impl Publisher for RedisPublisher {
    async fn publish(&self, event: Event) {
        self.pubsub.publish(event).await;
    }
}

pub struct RedisSubscriber {
    rx: AsyncReceiver<Event>,
}

#[async_trait]
impl Subscriber for RedisSubscriber {
    async fn recv(&self) -> Option<Event> {
        self.rx.recv().await.ok()
    }
    fn needs_ack(&self) -> bool {
        false
    }
    async fn send_ack(&self) {}
}

#[async_trait]
impl PubSubTrait for RedisPubSub {
    fn subscribe(&self, filter: EventFilter) -> Arc<dyn Subscriber> {
        self.subscribe(filter)
    }

    fn publisher(&self) -> Arc<dyn Publisher> {
        self.publisher()
    }

    async fn publish(&self, event: Event) {
        self.publish(event).await;
    }
}
