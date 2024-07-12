use std::sync::Arc;

use anyhow::Result;
use flume::Sender;
use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::TcpStream,
    select,
    sync::{OwnedSemaphorePermit, Semaphore},
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info};
use url::Url;

use crate::utils::Deduplicator;

use super::binance::Subscription;

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

    pub async fn run(&mut self, manager_tx: Sender<String>, subscription: Subscription) -> Result<()> {
        // Use select for new data in receiver or spawn new connection on permit
        info!("Starting WebSocket manager...");
        let (sender, receiver) = flume::unbounded::<Message>();

        loop {
            select! {
                msg = receiver.recv_async() => {
                    let msg = msg?;
                    // let bin_data = msg.into_data();
                    let data = msg.to_string();
                    if self.deduplicator.check(&data) {
                        manager_tx.send_async(data).await.unwrap();
                    }
                },
                permit = self.limit_connections.clone().acquire_owned() => {
                    // This should never fail, as the semaphore is never closed.
                    let permit = permit?;
                    debug!("Acquired permit: {:?}", permit);
                    match self.start_handler(permit, sender.clone(), subscription.clone()).await {
                        Ok(_) => info!("Started new handler"),
                        Err(e) => error!("Failed to start new handler: {:?}", e),
                    }
                }
            }
        }
    }

    async fn start_handler(
        &self,
        permit: OwnedSemaphorePermit,
        sender: Sender<Message>,
        subscription: Subscription,
    ) -> Result<()> {
        let mut handle = Handler::new(&self.url, sender, subscription).await?;
        tokio::spawn(async move {
            if let Err(err) = handle.run().await {
                error!("Websocket handler: {:?}", err);
            }
            drop(permit)
        });
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
    sender: Sender<Message>,
}

impl Handler {
    pub async fn new(url: &Url, sender: Sender<Message>, subscription: Subscription) -> Result<Self> {
        let (stream, _) = connect_async(url.to_string()).await?;
        Ok(Self {
            id: 0,
            subscription,
            stream,
            sender,
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
    async fn run(&mut self) -> Result<()> {
        let mut sub = self.subscription.clone();
        sub.update_id(self.id);
        self.stream.send(sub.into()).await?;

        while let Some(msg) = self.stream.next().await {
            let msg = msg?;
            self.handle_message(msg).await?;
        }
        Ok(())
    }

    async fn handle_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Text(text) => {
                debug!("Hanlder received text: {:?}", text);
                self.sender.send_async(Message::Text(text)).await?;
            }
            Message::Ping(ping) => {
                debug!("Handler received ping: {:?}", ping);
                self.stream.send(Message::Pong(ping)).await?;
                debug!("Sent pong");
            }
            _ => {
                debug!("Handler received other message: {:?}", msg);
            }
        }
        Ok(())
    }
}
