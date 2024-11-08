use crate::http::{Credentials, Method, Request};

/// `DELETE /fapi/v1/allOpenOrders`
///
/// Cancels all active orders on a symbol.
/// This includes OCO orders.
///
/// Weight(IP): 1
///
/// # Example
///
/// ```
/// use binance_spot_connector_rust::trade;
///
/// let request = trade::cancel_open_orders("BNBUSDT");
/// ```
pub struct CancelOpenOrders {
    symbol: String,
    recv_window: Option<u64>,
    credentials: Option<Credentials>,
}

impl CancelOpenOrders {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_owned(),
            recv_window: None,
            credentials: None,
        }
    }

    pub fn recv_window(mut self, recv_window: u64) -> Self {
        self.recv_window = Some(recv_window);
        self
    }

    pub fn credentials(mut self, credentials: &Credentials) -> Self {
        self.credentials = Some(credentials.clone());
        self
    }
}

impl From<CancelOpenOrders> for Request {
    fn from(request: CancelOpenOrders) -> Request {
        let mut params = vec![("symbol".to_owned(), request.symbol.to_string())];

        if let Some(recv_window) = request.recv_window {
            params.push(("recvWindow".to_owned(), recv_window.to_string()));
        }

        Request {
            path: "fapi/v1/allOpenOrders".to_owned(),
            method: Method::Delete,
            params,
            credentials: request.credentials,
            sign: true,
        }
    }
}
