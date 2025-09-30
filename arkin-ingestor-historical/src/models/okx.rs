use rust_decimal::Decimal;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use time::UtcDateTime;

use arkin_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxRoot {
    pub arg: Arg,
    pub data: Vec<Data>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Arg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub instrument: String,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Data {
    #[serde(rename = "instId")]
    pub instrument: String,
    #[serde(rename = "ts", with = "custom_serde::timestamp")]
    pub transaction_time: UtcDateTime,
    #[serde(rename = "tradeId")]
    pub trade_id: String,
    #[serde(rename = "px")]
    pub price: Decimal,
    #[serde(rename = "sz")]
    pub quantity: Decimal,
    pub side: String,
    #[serde_as(as = "DisplayFromStr")]
    pub count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_okx_spot_trade() {
        let json_data = r#"{"arg":{"channel":"trades","instId":"ETH-BTC"},"data":[{"instId":"ETH-BTC","tradeId":"55597951","px":"0.03567","sz":"0.0205","side":"buy","ts":"1735689652105","count":"1"}]}"#;
        let _ = serde_json::from_str::<OkxRoot>(json_data).unwrap();
    }

    #[test]

    fn test_okx_perp_trade() {
        let json_data = r#"{"arg":{"channel":"trades","instId":"BTC-USDT-SWAP"},"data":[{"instId":"BTC-USDT-SWAP","tradeId":"1216801608","px":"93630","sz":"2.1","side":"buy","ts":"1735689659701","count":"1"}]}"#;
        let _ = serde_json::from_str::<OkxRoot>(json_data).unwrap();
    }
}
