use arkin_core::{MarketSide, VenueOrderTimeInForce, VenueOrderType};
use rust_decimal::Decimal;
use strum::Display;

#[derive(Copy, Clone, Display)]
#[strum(serialize_all = "UPPERCASE")]
pub enum OrderType {
    Market,
    Limit,
}

impl From<VenueOrderType> for OrderType {
    fn from(order_type: VenueOrderType) -> Self {
        match order_type {
            VenueOrderType::Market => OrderType::Market,
            VenueOrderType::Limit => OrderType::Limit,
            VenueOrderType::StopMarket => unimplemented!("Stop market is not supported"),
            VenueOrderType::TakeProfit => unimplemented!("Take profit is not supported"),
            VenueOrderType::Stop => unimplemented!("Stop is not supported"),
            VenueOrderType::TakeProfitMarket => unimplemented!("Take Profit not supported"),
            VenueOrderType::TrailingStopMarket => unimplemented!("Trailing Stop Market is not supported"),
        }
    }
}
#[derive(Copy, Clone, Display)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

impl From<arkin_core::MarketSide> for Side {
    fn from(side: MarketSide) -> Self {
        match side {
            MarketSide::Buy => Side::Buy,
            MarketSide::Sell => Side::Sell,
        }
    }
}

#[derive(Copy, Clone, Display)]
#[strum(serialize_all = "UPPERCASE")]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
}

impl From<VenueOrderTimeInForce> for TimeInForce {
    fn from(time_in_force: VenueOrderTimeInForce) -> Self {
        match time_in_force {
            VenueOrderTimeInForce::Gtc => TimeInForce::Gtc,
            VenueOrderTimeInForce::Ioc => TimeInForce::Ioc,
            VenueOrderTimeInForce::Fok => TimeInForce::Fok,
            VenueOrderTimeInForce::Gtx => unimplemented!("GTX is not supported"),
            VenueOrderTimeInForce::Gtd => unimplemented!("GTD is not supported"),
        }
    }
}

#[derive(Copy, Clone, Display)]
#[strum(serialize_all = "UPPERCASE")]
pub enum NewOrderResponseType {
    Ack,
    Result,
    Full,
}

#[derive(Copy, Clone, Display)]
pub enum CancelReplaceMode {
    #[strum(serialize = "STOP_ON_FAILURE")]
    StopOnFailure,
    #[strum(serialize = "ALLOW_FAILURE")]
    AllowFailure,
}

pub struct WorkingMandatoryParams {
    pub working_type: String,
    pub working_side: Side,
    pub working_price: Decimal,
    pub working_quantity: Decimal,
}

impl WorkingMandatoryParams {
    pub fn new(working_type: &str, working_side: Side, working_price: Decimal, working_quantity: Decimal) -> Self {
        Self {
            working_type: working_type.to_owned(),
            working_side,
            working_price,
            working_quantity,
        }
    }
}
