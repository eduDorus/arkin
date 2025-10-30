use serde::{Deserialize, Serialize};
use std::fmt;

use crate::registry::WsChannel;

/// Subscription error types
#[derive(Clone, Debug)]
pub enum SubscriptionError {
    InvalidChannelForVenue(String),
    SerializationFailed(String),
    NoInstruments,
}

impl fmt::Display for SubscriptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubscriptionError::InvalidChannelForVenue(msg) => write!(f, "Invalid channel for venue: {}", msg),
            SubscriptionError::SerializationFailed(msg) => write!(f, "Serialization failed: {}", msg),
            SubscriptionError::NoInstruments => write!(f, "No instruments provided"),
        }
    }
}

impl std::error::Error for SubscriptionError {}

/// Core subscription trait
pub trait SubscriptionRequest: Serialize {
    fn build(channels: &[WsChannel], instruments: &[String]) -> Result<Self, SubscriptionError>
    where
        Self: Sized;
}

// ============================================================================
// BINANCE SUBSCRIPTION
// ============================================================================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinanceSubscription {
    pub method: String,
    pub params: Vec<String>,
    pub id: u64,
}

impl BinanceSubscription {
    pub fn new(params: Vec<String>) -> Self {
        Self {
            method: "SUBSCRIBE".to_string(),
            params,
            id: 1,
        }
    }
}

impl SubscriptionRequest for BinanceSubscription {
    fn build(channels: &[WsChannel], instruments: &[String]) -> Result<Self, SubscriptionError> {
        if instruments.is_empty() {
            return Err(SubscriptionError::NoInstruments);
        }

        let mut params = Vec::new();
        for channel in channels {
            let channel_str = match channel {
                WsChannel::AggTrades => "@aggTrade",
                WsChannel::TopOfBook => "@bookTicker",
                WsChannel::OrderBook => "@depth",
                WsChannel::FundingRate => "@forceOrder",
                WsChannel::MarkPriceKlines => "@markPrice_kline",
                WsChannel::IndexPriceKlines => "@indexPrice_kline",
                _ => {
                    return Err(SubscriptionError::InvalidChannelForVenue(format!(
                        "Channel {:?} not supported for Binance",
                        channel
                    )))
                }
            };

            for instrument in instruments {
                params.push(format!("{}{}", instrument.to_lowercase(), channel_str));
            }
        }

        Ok(BinanceSubscription::new(params))
    }
}

// ============================================================================
// OKX SUBSCRIPTION
// ============================================================================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OkxSubscription {
    pub op: String,
    pub args: Vec<OkxArg>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OkxArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub inst_id: String,
}

impl OkxSubscription {
    pub fn new(args: Vec<OkxArg>) -> Self {
        Self {
            op: "subscribe".to_string(),
            args,
        }
    }
}

impl SubscriptionRequest for OkxSubscription {
    fn build(channels: &[WsChannel], instruments: &[String]) -> Result<Self, SubscriptionError> {
        if instruments.is_empty() {
            return Err(SubscriptionError::NoInstruments);
        }

        let mut args = Vec::new();
        for channel in channels {
            let channel_str = match channel {
                WsChannel::Trades => "trades",
                WsChannel::TopOfBook => "tickers",
                WsChannel::OrderBook => "books",
                WsChannel::FundingRate => "funding-rate",
                WsChannel::MarkPriceKlines => "mark-price",
                WsChannel::IndexPriceKlines => "index-tickers",
                WsChannel::OpenInterest => "open-interest",
                _ => {
                    return Err(SubscriptionError::InvalidChannelForVenue(format!(
                        "Channel {:?} not supported for OKX",
                        channel
                    )))
                }
            };

            for instrument in instruments {
                args.push(OkxArg {
                    channel: channel_str.to_string(),
                    inst_id: instrument.clone(),
                });
            }
        }

        Ok(OkxSubscription::new(args))
    }
}

// ============================================================================
// BYBIT SUBSCRIPTION
// ============================================================================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BybitSubscription {
    pub op: String,
    pub args: Vec<String>,
}

impl BybitSubscription {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            op: "subscribe".to_string(),
            args,
        }
    }
}

impl SubscriptionRequest for BybitSubscription {
    fn build(channels: &[WsChannel], instruments: &[String]) -> Result<Self, SubscriptionError> {
        if instruments.is_empty() {
            return Err(SubscriptionError::NoInstruments);
        }

        let mut args = Vec::new();
        for channel in channels {
            let channel_prefix = match channel {
                WsChannel::Trades => "publicTrade.",
                WsChannel::TopOfBook => "tickers.",
                WsChannel::OrderBook => "orderbook.1.",
                WsChannel::MarkPriceKlines => "kline.1.",
                WsChannel::OpenInterest => "openInterest.",
                WsChannel::FundingRate => "funding.",
                _ => {
                    return Err(SubscriptionError::InvalidChannelForVenue(format!(
                        "Channel {:?} not supported for Bybit",
                        channel
                    )))
                }
            };

            for instrument in instruments {
                args.push(format!("{}{}", channel_prefix, instrument));
            }
        }

        Ok(BybitSubscription::new(args))
    }
}

// ============================================================================
// COINBASE SUBSCRIPTION
// ============================================================================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinbaseSubscription {
    #[serde(rename = "type")]
    pub subscription_type: String,
    pub product_ids: Vec<String>,
    pub channels: Vec<String>,
}

impl CoinbaseSubscription {
    pub fn new(product_ids: Vec<String>, channels: Vec<String>) -> Self {
        Self {
            subscription_type: "subscribe".to_string(),
            product_ids,
            channels,
        }
    }
}

impl SubscriptionRequest for CoinbaseSubscription {
    fn build(channels: &[WsChannel], instruments: &[String]) -> Result<Self, SubscriptionError> {
        if instruments.is_empty() {
            return Err(SubscriptionError::NoInstruments);
        }

        let mut channel_strs = Vec::new();
        for channel in channels {
            let channel_str = match channel {
                WsChannel::Trades => "matches",
                WsChannel::TopOfBook => "ticker",
                WsChannel::OrderBook => "level2",
                _ => {
                    return Err(SubscriptionError::InvalidChannelForVenue(format!(
                        "Channel {:?} not supported for Coinbase",
                        channel
                    )))
                }
            };
            channel_strs.push(channel_str.to_string());
        }

        Ok(CoinbaseSubscription::new(instruments.to_vec(), channel_strs))
    }
}

// ============================================================================
// DERIBIT SUBSCRIPTION
// ============================================================================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeribitSubscription {
    pub jsonrpc: String,
    pub method: String,
    pub params: DeribitParams,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeribitParams {
    pub channels: Vec<String>,
}

impl DeribitSubscription {
    pub fn new(channels: Vec<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: "public/subscribe".to_string(),
            params: DeribitParams { channels },
            id: 1,
        }
    }
}

impl SubscriptionRequest for DeribitSubscription {
    fn build(channels: &[WsChannel], instruments: &[String]) -> Result<Self, SubscriptionError> {
        if instruments.is_empty() {
            return Err(SubscriptionError::NoInstruments);
        }

        let mut channel_strs = Vec::new();
        for channel in channels {
            for instrument in instruments {
                let channel_str = match channel {
                    WsChannel::Trades => format!("trades.perpetual.{}.raw", instrument),
                    WsChannel::TopOfBook => format!("ticker.{}.raw", instrument),
                    WsChannel::OrderBook => format!("book.{}.raw", instrument),
                    WsChannel::MarkPriceKlines => format!("mark_price.{}.raw", instrument),
                    WsChannel::FundingRate => format!("funding_rate.{}.raw", instrument),
                    WsChannel::OpenInterest => format!("open_interest.{}.raw", instrument),
                    WsChannel::IndexPriceKlines => format!("deribit_price_index.{}", instrument),
                    _ => {
                        return Err(SubscriptionError::InvalidChannelForVenue(format!(
                            "Channel {:?} not supported for Deribit",
                            channel
                        )))
                    }
                };
                channel_strs.push(channel_str);
            }
        }

        Ok(DeribitSubscription::new(channel_strs))
    }
}

// ============================================================================
// HYPERLIQUID SUBSCRIPTION
// ============================================================================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HyperliquidSubscription {
    pub action: String,
    pub type_field: String,
    pub coin: String,
}

impl SubscriptionRequest for HyperliquidSubscription {
    fn build(channels: &[WsChannel], instruments: &[String]) -> Result<Self, SubscriptionError> {
        if instruments.is_empty() {
            return Err(SubscriptionError::NoInstruments);
        }

        // Hyperliquid uses coin-based subscription, combine all coins
        // For simplicity, we'll use the first instrument
        let coin = instruments[0].clone();

        let channel_type = match channels.first() {
            Some(WsChannel::Trades) => "trades",
            Some(WsChannel::OrderBook) => "l2Book",
            Some(WsChannel::FundingRate) => "funding",
            Some(WsChannel::MarkPriceKlines) => "premiumIndex",
            _ => {
                return Err(SubscriptionError::InvalidChannelForVenue(format!(
                    "Channel {:?} not supported for Hyperliquid",
                    channels.first()
                )))
            }
        };

        Ok(HyperliquidSubscription {
            action: "subscribe".to_string(),
            type_field: channel_type.to_string(),
            coin,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binance_subscription() {
        let sub = BinanceSubscription::build(
            &[WsChannel::Trades, WsChannel::TopOfBook],
            &["btcusdt".to_string(), "ethusdt".to_string()],
        )
        .unwrap();

        assert_eq!(sub.method, "SUBSCRIBE");
        assert_eq!(sub.params.len(), 4);
        assert!(sub.params.contains(&"btcusdt@aggTrade".to_string()));
        assert!(sub.params.contains(&"ethusdt@bookTicker".to_string()));
    }

    #[test]
    fn test_okx_subscription() {
        let sub =
            OkxSubscription::build(&[WsChannel::Trades, WsChannel::TopOfBook], &["BTC-USDT".to_string()]).unwrap();

        assert_eq!(sub.op, "subscribe");
        assert_eq!(sub.args.len(), 2);
        assert_eq!(sub.args[0].channel, "trades");
        assert_eq!(sub.args[0].inst_id, "BTC-USDT");
    }

    #[test]
    fn test_bybit_subscription() {
        let sub = BybitSubscription::build(&[WsChannel::Trades], &["BTCUSDT".to_string()]).unwrap();

        assert_eq!(sub.op, "subscribe");
        assert_eq!(sub.args[0], "publicTrade.BTCUSDT");
    }

    #[test]
    fn test_no_instruments_error() {
        let result = BinanceSubscription::build(&[WsChannel::Trades], &[]);
        assert!(matches!(result, Err(SubscriptionError::NoInstruments)));
    }
}
