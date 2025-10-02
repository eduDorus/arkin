use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BinanceSpotStreamEvent {
    pub stream: String,
    pub data: BinanceSpotBookTickerData,
}

/// Best bid/ask ticker data from Binance.
///
/// Contains the current best prices and available quantities
/// for immediate execution on both bid and ask sides.
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceSpotBookTickerData {
    /// Update ID
    #[serde(rename = "u")]
    pub update_id: u64,
    /// Trading pair symbol
    #[serde(rename = "s")]
    pub instrument: String,
    /// Best bid price
    #[serde(rename = "b")]
    pub bid_price: Decimal,
    /// Bid quantity
    #[serde(rename = "B")]
    pub bid_quantity: Decimal,
    /// Best ask price
    #[serde(rename = "a")]
    pub ask_price: Decimal,
    /// Ask quantity
    #[serde(rename = "A")]
    pub ask_quantity: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binance_spot_ticker() {
        let json_data = r#"{"stream":"btcusdt@bookTicker","data":{"u":76861015827,"s":"BTCUSDT","b":"109257.64000000","B":"0.77163000","a":"109257.65000000","A":"6.27706000"}}"#;
        let _ = serde_json::from_str::<BinanceSpotStreamEvent>(json_data).unwrap();
    }
}
