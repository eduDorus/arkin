use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Connector client error: {0}")]
    ConnectorClientError(String),

    #[error("Unauthorized access. Authentication required. {0}")]
    UnauthorizedError(String),

    #[error("Access to the requested resource is forbidden. {0}")]
    ForbiddenError(String),

    #[error("Too many requests. You are being rate-limited. {0}")]
    TooManyRequestsError(String),

    #[error("The IP address has been banned for exceeding rate limits. {0}")]
    RateLimitBanError(String),

    #[error("Internal server error: {msg} (status code: {status_code:?})")]
    ServerError {
        msg: String,
        status_code: Option<u16>,
    },

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("The requested resource was not found. {0}")]
    NotFoundError(String),

    #[error("Bad request: {0}")]
    BadRequestError(String),
}

#[derive(Debug, Error)]
pub enum WebsocketError {
    #[error("WebSocket timeout error")]
    Timeout,
    #[error("WebSocket protocol error: {0}")]
    Protocol(String),
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("WebSocket handshake error: {0}")]
    Handshake(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("No active WebSocket connection error.")]
    NotConnected,
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("No response error.")]
    NoResponse,
    #[error("Server‐side response error (code {code}): {message}")]
    ResponseError { code: i64, message: String },
}
