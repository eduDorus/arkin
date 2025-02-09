use rust_decimal::prelude::*;
use typed_builder::TypedBuilder;

use crate::http::{Credentials, Method, Request};

use super::order::{NewOrderResponseType, OrderType, Side, TimeInForce};

/// `POST /api/v3/order`
///
/// Send in a new order.
///
/// * `LIMIT_MAKER` are `LIMIT` orders that will be rejected if they would immediately match and trade as a taker.
/// * `STOP_LOSS` and `TAKE_PROFIT` will execute a `MARKET` order when the `stopPrice` is reached.
/// * Any `LIMIT` or `LIMIT_MAKER` type order can be made an iceberg order by sending an `icebergQty`.
/// * Any order with an `icebergQty` MUST have `timeInForce` set to `GTC`.
/// * `MARKET` orders using `quantity` specifies how much a user wants to buy or sell based on the market price.
/// * `MARKET` orders using `quoteOrderQty` specifies the amount the user wants to spend (when buying) or receive (when selling) of the quote asset; the correct quantity will be determined based on the market liquidity and `quoteOrderQty`.
/// * `MARKET` orders using `quoteOrderQty` will not break `LOT_SIZE` filter rules; the order will execute a quantity that will have the notional value as close as possible to `quoteOrderQty`.
/// * same `newClientOrderId` can be accepted only when the previous one is filled, otherwise the order will be rejected.
///
/// Trigger order price rules against market price for both `MARKET` and `LIMIT` versions:
///
/// * Price above market price: `STOP_LOSS` `BUY`, `TAKE_PROFIT` `SELL`
/// * Price below market price: `STOP_LOSS` `SELL`, `TAKE_PROFIT` `BUY`
///
///
/// Weight(IP): 1
///
/// # Example
///
/// ```
/// use binance_spot_connector_rust::trade::{ self, order::Side };
/// use rust_decimal_macros::dec;
///
/// let request = trade::new_order("BNBUSDT", Side::Sell, "MARKET")
///     .quantity(dec!(0.1));
/// ```
#[derive(TypedBuilder)]
pub struct NewOrderRequest {
    symbol: String,
    side: Side,
    order_type: OrderType,
    #[builder(default)]
    time_in_force: Option<TimeInForce>,
    #[builder(default)]
    quantity: Option<Decimal>,
    #[builder(default)]
    quote_order_qty: Option<Decimal>,
    #[builder(default)]
    price: Option<Decimal>,
    #[builder(default)]
    new_client_order_id: Option<String>,
    #[builder(default)]
    stop_price: Option<Decimal>,
    #[builder(default)]
    trailing_delta: Option<u64>,
    #[builder(default)]
    iceberg_qty: Option<Decimal>,
    #[builder(default)]
    new_order_resp_type: Option<NewOrderResponseType>,
    #[builder(default)]
    recv_window: Option<u64>,
    #[builder(default)]
    credentials: Option<Credentials>,
}

impl NewOrderRequest {
    pub fn time_in_force(mut self, time_in_force: TimeInForce) -> Self {
        self.time_in_force = Some(time_in_force);
        self
    }

    pub fn quantity(mut self, quantity: Decimal) -> Self {
        self.quantity = Some(quantity);
        self
    }

    pub fn quote_order_qty(mut self, quote_order_qty: Decimal) -> Self {
        self.quote_order_qty = Some(quote_order_qty);
        self
    }

    pub fn price(mut self, price: Decimal) -> Self {
        self.price = Some(price);
        self
    }

    pub fn new_client_order_id(mut self, new_client_order_id: &str) -> Self {
        self.new_client_order_id = Some(new_client_order_id.to_owned());
        self
    }

    pub fn stop_price(mut self, stop_price: Decimal) -> Self {
        self.stop_price = Some(stop_price);
        self
    }

    pub fn trailing_delta(mut self, trailing_delta: u64) -> Self {
        self.trailing_delta = Some(trailing_delta);
        self
    }

    pub fn iceberg_qty(mut self, iceberg_qty: Decimal) -> Self {
        self.iceberg_qty = Some(iceberg_qty);
        self
    }

    pub fn new_order_resp_type(mut self, new_order_resp_type: NewOrderResponseType) -> Self {
        self.new_order_resp_type = Some(new_order_resp_type);
        self
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

impl From<NewOrderRequest> for Request {
    fn from(request: NewOrderRequest) -> Request {
        let mut params = vec![
            ("symbol".to_owned(), request.symbol),
            ("side".to_owned(), request.side.to_string()),
            ("type".to_owned(), request.order_type.to_string()),
        ];

        if let Some(time_in_force) = request.time_in_force {
            params.push(("timeInForce".to_owned(), time_in_force.to_string()));
        }

        if let Some(quantity) = request.quantity {
            params.push(("quantity".to_owned(), quantity.to_string()));
        }

        if let Some(quote_order_qty) = request.quote_order_qty {
            params.push(("quoteOrderQty".to_owned(), quote_order_qty.to_string()));
        }

        if let Some(price) = request.price {
            params.push(("price".to_owned(), price.to_string()));
        }

        if let Some(new_client_order_id) = request.new_client_order_id {
            params.push(("newClientOrderId".to_owned(), new_client_order_id));
        }

        if let Some(stop_price) = request.stop_price {
            params.push(("stopPrice".to_owned(), stop_price.to_string()));
        }

        if let Some(trailing_delta) = request.trailing_delta {
            params.push(("trailingDelta".to_owned(), trailing_delta.to_string()));
        }

        if let Some(iceberg_qty) = request.iceberg_qty {
            params.push(("icebergQty".to_owned(), iceberg_qty.to_string()));
        }

        if let Some(new_order_resp_type) = request.new_order_resp_type {
            params.push(("newOrderRespType".to_owned(), new_order_resp_type.to_string()));
        }

        if let Some(recv_window) = request.recv_window {
            params.push(("recvWindow".to_owned(), recv_window.to_string()));
        }

        Request {
            path: "fapi/v1/order".to_owned(),
            method: Method::Post,
            params,
            credentials: request.credentials,
            sign: true,
        }
    }
}
