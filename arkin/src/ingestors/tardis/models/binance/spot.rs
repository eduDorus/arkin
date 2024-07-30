use rust_decimal::Decimal;
use serde::Deserialize;
use time::OffsetDateTime;

use crate::utils::custom_serde;

#[derive(Debug, Deserialize)]
pub struct BinanceSpotAggTrade {
    pub stream: String,
    pub data: BinanceSpotAggTradeData,
}

#[derive(Debug, Deserialize)]
pub struct BinanceSpotAggTradeData {
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: OffsetDateTime,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: OffsetDateTime,
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "s")]
    pub instrument: String,
    #[serde(rename = "a")]
    pub agg_trade_id: u64,
    #[serde(rename = "f")]
    pub first_trade_id: u64,
    #[serde(rename = "l")]
    pub last_trade_id: u64,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "q")]
    pub quantity: Decimal,
    #[serde(rename = "m")]
    pub maker: bool, // The true = sell, false = buy
    #[serde(rename = "M")]
    pub ignore: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binance_futures_agg_trade_1() {
        let json_data = r#"{"stream":"btcusdt@aggTrade","data":{"e":"aggTrade","E":1704894291955,"s":"BTCUSDT","a":2822096941,"p":"45083.19000000","q":"0.00017000","f":3362935125,"l":3362935125,"T":1704894291954,"m":true,"M":true}}"#;
        let _ = serde_json::from_str::<BinanceSpotAggTrade>(json_data).unwrap();
    }

    #[test]
    fn test_binance_futures_agg_trade_2() {
        let json_data = r#"{"stream":"btcusdt@aggTrade","data":{"e":"aggTrade","E":1704895018786,"s":"BTCUSDT","a":2822117879,"p":"45128.00000000","q":"0.01303000","f":3362959266,"l":3362959266,"T":1704895018785,"m":false,"M":true}}"#;
        let _ = serde_json::from_str::<BinanceSpotAggTrade>(json_data).unwrap();
    }
}
