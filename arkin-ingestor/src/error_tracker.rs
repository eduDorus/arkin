use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Error types categorized for statistics tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Connection failed to establish
    ConnectionFailed,
    /// WebSocket protocol error
    WebSocketError,
    /// Failed to parse JSON message
    ParseError,
    /// Failed to send pong response
    PongSendFailed,
    /// Failed to send ping
    PingSendFailed,
    /// Failed to send subscription
    SubscriptionFailed,
    /// Stale connection detected
    StaleConnection,
    /// Unexpected binary message received
    UnexpectedBinary,
    /// Generic/Other error
    Other,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed => write!(f, "connection_failed"),
            Self::WebSocketError => write!(f, "websocket_error"),
            Self::ParseError => write!(f, "parse_error"),
            Self::PongSendFailed => write!(f, "pong_send_failed"),
            Self::PingSendFailed => write!(f, "ping_send_failed"),
            Self::SubscriptionFailed => write!(f, "subscription_failed"),
            Self::StaleConnection => write!(f, "stale_connection"),
            Self::UnexpectedBinary => write!(f, "unexpected_binary"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Error statistics for WebSocket connection
#[derive(Debug, Clone)]
pub struct ErrorStats {
    /// Total number of errors
    pub total_errors: u64,
    /// Error counts by category
    pub errors_by_type: Vec<(String, u64)>,
    /// Last error message details (type, message)
    pub last_error: Option<(String, String)>,
}

impl std::fmt::Display for ErrorStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Total Errors: {} | Breakdown: {}",
            self.total_errors,
            self.errors_by_type
                .iter()
                .map(|(category, count)| format!("{}: {}", category, count))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Tracks error statistics for the WebSocket client
#[allow(dead_code)]
pub struct ErrorTracker {
    total_errors: Arc<AtomicU64>,
    connection_failed: Arc<AtomicU64>,
    websocket_error: Arc<AtomicU64>,
    parse_error: Arc<AtomicU64>,
    pong_send_failed: Arc<AtomicU64>,
    ping_send_failed: Arc<AtomicU64>,
    subscription_failed: Arc<AtomicU64>,
    stale_connection: Arc<AtomicU64>,
    unexpected_binary: Arc<AtomicU64>,
    other_error: Arc<AtomicU64>,
    last_error_category: Arc<tokio::sync::Mutex<Option<String>>>,
    last_error_message: Arc<tokio::sync::Mutex<Option<String>>>,
}

impl ErrorTracker {
    pub fn new() -> Self {
        Self {
            total_errors: Arc::new(AtomicU64::new(0)),
            connection_failed: Arc::new(AtomicU64::new(0)),
            websocket_error: Arc::new(AtomicU64::new(0)),
            parse_error: Arc::new(AtomicU64::new(0)),
            pong_send_failed: Arc::new(AtomicU64::new(0)),
            ping_send_failed: Arc::new(AtomicU64::new(0)),
            subscription_failed: Arc::new(AtomicU64::new(0)),
            stale_connection: Arc::new(AtomicU64::new(0)),
            unexpected_binary: Arc::new(AtomicU64::new(0)),
            other_error: Arc::new(AtomicU64::new(0)),
            last_error_category: Arc::new(tokio::sync::Mutex::new(None)),
            last_error_message: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    pub async fn record_error(&self, category: ErrorCategory) {
        self.total_errors.fetch_add(1, Ordering::SeqCst);
        match category {
            ErrorCategory::ConnectionFailed => self.connection_failed.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::WebSocketError => self.websocket_error.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::ParseError => self.parse_error.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::PongSendFailed => self.pong_send_failed.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::PingSendFailed => self.ping_send_failed.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::SubscriptionFailed => self.subscription_failed.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::StaleConnection => self.stale_connection.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::UnexpectedBinary => self.unexpected_binary.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::Other => self.other_error.fetch_add(1, Ordering::SeqCst),
        };
    }

    pub async fn record_error_with_message(&self, category: ErrorCategory, message: String) {
        self.record_error(category).await;

        let category_str = category.to_string();
        let last_category = self.last_error_category.clone();
        let last_message = self.last_error_message.clone();

        // Spawn a task to update the error details (non-blocking)
        let mut cat = last_category.lock().await;
        *cat = Some(category_str);

        let mut msg = last_message.lock().await;
        *msg = Some(message);
    }

    pub fn get_stats(&self) -> ErrorStats {
        let mut errors_by_type = vec![
            ("connection_failed".to_string(), self.connection_failed.load(Ordering::SeqCst)),
            ("websocket_error".to_string(), self.websocket_error.load(Ordering::SeqCst)),
            ("parse_error".to_string(), self.parse_error.load(Ordering::SeqCst)),
            ("pong_send_failed".to_string(), self.pong_send_failed.load(Ordering::SeqCst)),
            ("ping_send_failed".to_string(), self.ping_send_failed.load(Ordering::SeqCst)),
            (
                "subscription_failed".to_string(),
                self.subscription_failed.load(Ordering::SeqCst),
            ),
            ("stale_connection".to_string(), self.stale_connection.load(Ordering::SeqCst)),
            ("unexpected_binary".to_string(), self.unexpected_binary.load(Ordering::SeqCst)),
            ("other".to_string(), self.other_error.load(Ordering::SeqCst)),
        ];

        // Filter out zero counts for cleaner output
        errors_by_type.retain(|(_, count)| *count > 0);

        // Get last error details (non-blocking read attempt)
        let last_error = if let Ok(cat) = self.last_error_category.try_lock() {
            if let Ok(msg) = self.last_error_message.try_lock() {
                match (cat.clone(), msg.clone()) {
                    (Some(c), Some(m)) => Some((c, m)),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        ErrorStats {
            total_errors: self.total_errors.load(Ordering::SeqCst),
            errors_by_type,
            last_error,
        }
    }
}
