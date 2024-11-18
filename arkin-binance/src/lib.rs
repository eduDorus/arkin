mod http;
mod utils;
mod ws;

mod usdm;

pub use http::{BinanceHttpClient, BinanceHttpClientError, Credentials, Method, Request, Response};
pub use ws::{BinanceWebSocketClient, Stream};

pub use usdm::market_stream::*;
pub use usdm::trade::*;

pub mod prelude {
    pub use crate::usdm::market_stream::*;
    pub use crate::usdm::models::*;
    pub use crate::usdm::trade::*;
    pub use crate::ws::{BinanceWebSocketClient, Stream, WebSocketState};
    pub use crate::{BinanceHttpClient, BinanceHttpClientError, Credentials, Method, Request, Response};
}
