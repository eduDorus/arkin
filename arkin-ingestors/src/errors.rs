use thiserror::Error;

#[derive(Error, Debug)]
pub enum IngestorError {
    #[error("Channel send error: {0}")]
    ChannelSendError(#[from] flume::SendError<async_tungstenite::tungstenite::Message>),

    #[error("Channel receive error: {0}")]
    ChannelReceiveError(#[from] flume::RecvError),

    #[error("Websocket error: {0}")]
    WebSocketError(#[from] async_tungstenite::tungstenite::Error),

    #[error("Acuireing the lock failed: {0}")]
    LockError(#[from] tokio::sync::AcquireError),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    #[error(transparent)]
    PersistenceError(#[from] arkin_persistence::PersistenceError),

    #[error("Error in the persistence service: {0}")]
    PersistenceServiceError(String),

    #[error("Error in configuration: {0}")]
    ConfigError(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
