use async_tungstenite::{
    tokio::{connect_async, ConnectStream},
    tungstenite::{handshake::client::Response, protocol::Message, Error},
    WebSocketStream,
};
use futures::{AsyncRead, AsyncWrite};
use tracing::{debug, info};

use super::Stream;

/// Binance websocket client using Tungstenite.
pub struct BinanceWebSocketClient;

impl BinanceWebSocketClient {
    pub async fn connect(url: &str) -> Result<(WebSocketState<ConnectStream>, Response), Error> {
        let (socket, response) = connect_async(url).await?;

        info!("Connected to {}", url);
        debug!("Response HTTP code: {}", response.status());
        debug!("Response headers:");
        for (ref header, _value) in response.headers() {
            debug!("* {}", header);
        }

        Ok((WebSocketState::new(socket), response))
    }

    pub async fn connect_default() -> Result<(WebSocketState<ConnectStream>, Response), Error> {
        BinanceWebSocketClient::connect("wss://fstream.binance.com/ws").await
    }

    pub async fn connect_with_listen_key(listen_key: &str) -> Result<(WebSocketState<ConnectStream>, Response), Error> {
        let url = format!("wss://fstream.binance.com/ws/{}", listen_key);
        BinanceWebSocketClient::connect(&url).await
    }
}

pub struct WebSocketState<T> {
    pub socket: WebSocketStream<T>,
    id: u64,
}

impl<T: AsyncRead + AsyncWrite + Unpin> WebSocketState<T> {
    pub fn new(socket: WebSocketStream<T>) -> Self {
        Self { socket, id: 0 }
    }

    async fn send(&mut self, method: &str, params: impl IntoIterator<Item = &str>) -> u64 {
        let mut params_str: String = params
            .into_iter()
            .map(|param| format!("\"{}\"", param))
            .collect::<Vec<String>>()
            .join(",");

        if !params_str.is_empty() {
            params_str = format!("\"params\": [{params}],", params = params_str)
        };

        let id = self.id.clone();
        self.id += 1;

        let s = format!(
            "{{\"method\":\"{method}\",{params}\"id\":{id}}}",
            method = method,
            params = params_str,
            id = id
        );
        let message = Message::Text(s.into());

        self.socket.send(message).await.unwrap();

        id
    }

    /// Sends `SUBSCRIBE` message for the given `streams`.
    ///
    /// `streams` are not validated. Invalid streams will be
    /// accepted by the server, but no data will be sent.
    /// Requests to subscribe an existing stream will be ignored
    /// by the server.
    ///
    /// Returns the message `id`. This should be used to match
    /// the request with a future response. Sent messages should
    /// not share the same message `id`.
    ///
    /// You should expect the server to respond with a similar
    /// message.
    /// ```json
    /// { "method": "SUBSCRIBE", "params": [ <streams> ], "id": <id> }
    /// ```
    pub async fn subscribe(&mut self, streams: impl IntoIterator<Item = &Stream>) -> u64 {
        self.send("SUBSCRIBE", streams.into_iter().map(|s| s.as_str())).await
    }

    /// Sends `UNSUBSCRIBE` message for the given `streams`.
    ///
    /// `streams` are not validated. Non-existing streams will be
    /// ignored by the server.
    ///
    /// Returns the message `id`. This should be used to match
    /// the request with a future response. Sent messages should
    /// not share the same message `id`.
    ///
    /// You should expect the server to respond with a similar
    /// message.
    /// ```json
    /// { "method": "UNSUBSCRIBE", "params": [ <streams> ], "id": <id> }
    /// ```
    pub async fn unsubscribe(&mut self, streams: impl IntoIterator<Item = &Stream>) -> u64 {
        self.send("UNSUBSCRIBE", streams.into_iter().map(|s| s.as_str())).await
    }

    /// Sends `LIST_SUBSCRIPTIONS` message.
    ///
    /// Returns the message `id`. This should be used to match
    /// the request with a future response. Sent messages should
    /// not share the same message `id`.
    ///
    /// You should expect the server to respond with a similar
    /// message.
    /// ```json
    /// { "method": "LIST_SUBSCRIPTIONS", "params": [ <streams> ], "id": <id> }
    /// ```
    pub async fn subscriptions(&mut self) -> u64 {
        self.send("LIST_SUBSCRIPTIONS", vec![]).await
    }

    pub async fn close(mut self) -> Result<(), Error> {
        self.socket.close(None).await
    }
}

impl<T> From<WebSocketState<T>> for WebSocketStream<T> {
    fn from(conn: WebSocketState<T>) -> WebSocketStream<T> {
        conn.socket
    }
}

impl<T> AsMut<WebSocketStream<T>> for WebSocketState<T> {
    fn as_mut(&mut self) -> &mut WebSocketStream<T> {
        &mut self.socket
    }
}
