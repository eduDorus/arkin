use arkin_core::prelude::*;

use crate::types::{BinanceMarketType, OrderParams};

pub fn instrument_to_market_type(instrument: &Instrument) -> BinanceMarketType {
    match instrument.instrument_type {
        InstrumentType::Spot => BinanceMarketType::Spot,
        InstrumentType::Perpetual | InstrumentType::InversePerpetual => BinanceMarketType::Usdm,
        // For other types, default to spot for now
        _ => BinanceMarketType::Spot,
    }
}

pub fn venue_order_to_params(order: &VenueOrder, market_type: &BinanceMarketType) -> OrderParams {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let (order_type, time_in_force, quantity, quote_order_qty, price, stop_price) =
        match (order.order_type, market_type) {
            (VenueOrderType::Market, _) => {
                // For market orders, we use quantity
                (
                    "MARKET".to_string(),
                    None,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    None,
                    None,
                )
            }
            (VenueOrderType::Limit, _) => {
                let tif = match order.time_in_force {
                    VenueOrderTimeInForce::Gtc => Some("GTC".to_string()),
                    VenueOrderTimeInForce::Ioc => Some("IOC".to_string()),
                    VenueOrderTimeInForce::Fok => Some("FOK".to_string()),
                    VenueOrderTimeInForce::Gtx => Some("GTX".to_string()),
                    VenueOrderTimeInForce::Gtd => Some("GTD".to_string()),
                };
                (
                    "LIMIT".to_string(),
                    tif,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                    None,
                )
            }
            (VenueOrderType::StopMarket, BinanceMarketType::Spot) => {
                // For spot stop market orders - price is used as stopPrice
                (
                    "STOP_LOSS".to_string(),
                    None,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    None,
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                )
            }
            (VenueOrderType::StopMarket, BinanceMarketType::Margin) => {
                // For margin stop market orders - same as spot
                (
                    "STOP_LOSS".to_string(),
                    None,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    None,
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                )
            }
            (VenueOrderType::StopMarket, BinanceMarketType::Usdm) => {
                // For futures stop market orders - price is used as stopPrice
                (
                    "STOP_MARKET".to_string(),
                    None,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    None,
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                )
            }
            (VenueOrderType::StopLimit, BinanceMarketType::Spot) => {
                let tif = match order.time_in_force {
                    VenueOrderTimeInForce::Gtc => Some("GTC".to_string()),
                    VenueOrderTimeInForce::Ioc => Some("IOC".to_string()),
                    VenueOrderTimeInForce::Fok => Some("FOK".to_string()),
                    VenueOrderTimeInForce::Gtx => Some("GTX".to_string()),
                    VenueOrderTimeInForce::Gtd => Some("GTD".to_string()),
                };
                // For stop limit orders, price is used as both limit price and stop price (since we don't have separate fields)
                (
                    "STOP_LOSS_LIMIT".to_string(),
                    tif,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                )
            }
            (VenueOrderType::StopLimit, BinanceMarketType::Margin) => {
                let tif = match order.time_in_force {
                    VenueOrderTimeInForce::Gtc => Some("GTC".to_string()),
                    VenueOrderTimeInForce::Ioc => Some("IOC".to_string()),
                    VenueOrderTimeInForce::Fok => Some("FOK".to_string()),
                    VenueOrderTimeInForce::Gtx => Some("GTX".to_string()),
                    VenueOrderTimeInForce::Gtd => Some("GTD".to_string()),
                };
                // For margin stop limit orders - same as spot
                (
                    "STOP_LOSS_LIMIT".to_string(),
                    tif,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                )
            }
            (VenueOrderType::StopLimit, BinanceMarketType::Usdm) => {
                let tif = match order.time_in_force {
                    VenueOrderTimeInForce::Gtc => Some("GTC".to_string()),
                    VenueOrderTimeInForce::Ioc => Some("IOC".to_string()),
                    VenueOrderTimeInForce::Fok => Some("FOK".to_string()),
                    VenueOrderTimeInForce::Gtx => Some("GTX".to_string()),
                    VenueOrderTimeInForce::Gtd => Some("GTD".to_string()),
                };
                // For USDM stop limit orders - use STOP_LOSS_LIMIT instead of STOP
                (
                    "STOP_LOSS_LIMIT".to_string(),
                    tif,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                )
            }
            (VenueOrderType::TakeProfit, _) => (
                "TAKE_PROFIT".to_string(),
                None,
                Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                None,
                None,
                Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
            ),
            (VenueOrderType::TakeProfitMarket, BinanceMarketType::Usdm) => (
                "TAKE_PROFIT_MARKET".to_string(),
                None,
                Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                None,
                None,
                Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
            ),
            (VenueOrderType::TakeProfitMarket, BinanceMarketType::Spot) => {
                // Spot doesn't support TAKE_PROFIT_MARKET, fallback to TAKE_PROFIT
                (
                    "TAKE_PROFIT".to_string(),
                    None,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    None,
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                )
            }
            (VenueOrderType::TakeProfitMarket, BinanceMarketType::Margin) => {
                // Margin doesn't support TAKE_PROFIT_MARKET, fallback to TAKE_PROFIT
                (
                    "TAKE_PROFIT".to_string(),
                    None,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    None,
                    Some(format!("{:.*}", order.instrument.price_precision as usize, order.price)),
                )
            }
            (VenueOrderType::TrailingStopMarket, BinanceMarketType::Usdm) => (
                "TRAILING_STOP_MARKET".to_string(),
                None,
                Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                None,
                None,
                None,
            ),
            (VenueOrderType::TrailingStopMarket, BinanceMarketType::Spot) => {
                // Spot doesn't support TRAILING_STOP_MARKET, fallback to MARKET
                (
                    "MARKET".to_string(),
                    None,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    None,
                    None,
                )
            }
            (VenueOrderType::TrailingStopMarket, BinanceMarketType::Margin) => {
                // Margin doesn't support TRAILING_STOP_MARKET, fallback to MARKET
                (
                    "MARKET".to_string(),
                    None,
                    Some(format!("{:.*}", order.instrument.quantity_precision as usize, order.quantity)),
                    None,
                    None,
                    None,
                )
            }
        };

    // Set position_side and reduce_only for USDM orders
    let (position_side, reduce_only) = match market_type {
        BinanceMarketType::Usdm => (Some("BOTH".to_string()), Some("false".to_string())),
        _ => (None, None),
    };

    OrderParams {
        symbol: order.instrument.venue_symbol.clone(),
        side: match order.side {
            MarketSide::Buy => "BUY".to_string(),
            MarketSide::Sell => "SELL".to_string(),
        }
        .to_string(),
        order_type,
        time_in_force,
        quantity,
        quote_order_qty,
        price,
        stop_price,
        position_side,
        reduce_only,
        timestamp,
        recv_window: Some(5000),
        new_client_order_id: Some(order.id.to_string()),
    }
}
