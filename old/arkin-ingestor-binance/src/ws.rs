use std::sync::Arc;

use arkin_core::prelude::*;
use futures::{SinkExt, StreamExt};
use kanal::AsyncSender;
use tokio::{net::TcpStream, select, sync::Semaphore};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, error, info};
use url::Url;

use crate::{error::IngestorError, Subscription};

/// A WebSocket manager handles multiple WebSocket connections.
pub struct WebSocketManager {
    pub url: Url,

    /// Deduplicator
    pub deduplicator: Deduplicator,

    /// Limit the max number of connections.
    ///
    /// A `Semaphore` is used to limit the max number of connections. Before
    /// attempo accept a new connection, a permit is acquired from the
    /// semaphore. If none are available, the listener waits for one.
    ///
    /// When handlers complete processing a connection, the permit is returned
    /// to the semaphore.
    pub limit_connections: Arc<Semaphore>,
}

impl WebSocketManager {
    pub fn new(url: Url, connections: usize, deduplicate_lookback: usize) -> Self {
        Self {
            url,
            deduplicator: Deduplicator::new(deduplicate_lookback),
            limit_connections: Arc::new(Semaphore::new(connections)),
        }
    }

    pub async fn run(
        &mut self,
        manager_tx: AsyncSender<String>,
        subscription: Subscription,
        shutdown: CancellationToken,
    ) -> Result<(), IngestorError> {
        // Use select for new data in receiver or spawn new connection on permit
        info!("Starting WebSocket manager...");
        let (sender, receiver) = kanal::unbounded_async::<Message>();
        let websocket_tracker = TaskTracker::new();
        loop {
            select! {
                msg = receiver.recv() => {
                    let msg = match msg {
                        Ok(msg) => msg,
                        Err(e) => {
                            error!("Failed to receive message: {:?}", e);
                            continue;
                        }
                    };
                    // let bin_data = msg.into_data();
                    let data = msg.to_string();
                    if self.deduplicator.has(&data).await {
                        manager_tx.send(data).await.unwrap();
                    }
                }
                permit = self.limit_connections.clone().acquire_owned() => {
                    // This should never fail, as the semaphore is never closed.
                    if shutdown.is_cancelled() {
                        continue;
                    }
                    let permit = permit.expect("Aquire went wrong");
                    debug!("Acquired permit: {:?}", permit);
                    let mut handle = Handler::new(&self.url, sender.clone(), subscription.clone(), shutdown.clone()).await?;
                    websocket_tracker.spawn(async move {
                        if let Err(err) = handle.run().await {
                            error!("Websocket handler: {:?}", err);
                        }
                        drop(permit)
                    });

                }
                _ = shutdown.cancelled() => {
                    info!("Shutting down WebSocket manager...");
                    websocket_tracker.close();
                    websocket_tracker.wait().await;
                    break;
                }
            }
        }
        Ok(())
    }
}

/// Per-connection handler. Reads requests from `connection` or sends requests
pub struct Handler {
    id: u64,
    subscription: Subscription,
    /// The TCP connection decorated with the redis protocol encoder / decoder
    /// implemented using a buffered `TcpStream`.
    ///
    /// When `Listener` receives an inbound connection, the `TcpStream` is
    /// passed to `Connection::new`, which initializes the associated buffers.
    /// `Connection` allows the handler to operate at the "frame" level and keep
    /// the byte level protocol parsing details encapsulated in `Connection`.
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,

    /// Send messages to the WebSocket Manager
    sender: AsyncSender<Message>,

    /// Shutdown signal
    shutdown: CancellationToken,
}

impl Handler {
    pub async fn new(
        url: &Url,
        sender: AsyncSender<Message>,
        subscription: Subscription,
        shutdown: CancellationToken,
    ) -> Result<Self, IngestorError> {
        let (mut stream, _) = connect_async(url.to_string()).await.unwrap();
        // Send ping
        let ping = Message::Ping(bytes::Bytes::new());
        stream.send(ping).await;

        Ok(Self {
            id: 0,
            subscription,
            stream,
            sender,
            shutdown,
        })
    }

    /// Process a single connection.
    ///
    /// Request frames are read from the socket and processed. Responses are
    /// written back to the socket.
    ///
    /// Currently, pipelining is not implemented. Pipelining is the ability to
    /// process more than one request concurrently per connection without
    /// interleaving frames. See for more details:
    /// https://redis.io/topics/pipelining
    ///
    /// When the shutdown signal is received, the connection is processed until
    /// it reaches a safe state, at which point it is terminated.
    async fn run(&mut self) -> Result<(), IngestorError> {
        let mut sub = self.subscription.clone();
        sub.update_id(self.id);
        self.stream.send(sub.into()).await;

        loop {
            select! {
                    Some(msg) = self.stream.next() => {
                        let msg = msg.unwrap();
                        self.handle_message(msg).await?;
                    }
                    _ = self.shutdown.cancelled() => {
                        info!("Shutting down handler...");
                        self.stream.send(Message::Close(None)).await;
                        break;
                    }
            }
        }
        Ok(())
    }

    async fn handle_message(&mut self, msg: Message) -> Result<(), IngestorError> {
        match msg {
            Message::Text(text) => {
                debug!("Hanlder received text: {:?}", text);
                self.sender.send(Message::Text(text)).await;
            }
            Message::Ping(ping) => {
                debug!("Handler received ping: {:?}", ping);
                self.stream.send(Message::Pong(ping)).await;
            }
            _ => {
                debug!("Handler received other message: {:?}", msg);
            }
        }
        Ok(())
    }
}
