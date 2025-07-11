use serde::Deserialize;
use serde_this_or_that::{as_f64, as_u64};
use time::UtcDateTime;

use crate::utils::custom_serde;

// https://api.tardis.dev/v1/exchanges
// {
//     "id": "okex-swap",
//     "name": "OKX Swap",
//     "enabled": true,
//     "supportsDatasets": true,
//     "availableSince": "2019-03-30T00:00:00.000Z",
//     "availableChannels": [
//         "swap/trade",
//         "swap/depth",
//         "swap/depth_l2_tbt",
//         "swap/ticker",
//         "swap/funding_rate",
//         "swap/mark_price",
//         "swap/liquidation",
//         "index/ticker",
//         "system/status",
//         "information/sentiment",
//         "information/long_short_ratio",
//         "information/margin",
//         "trades",
//         "trades-all",
//         "books-l2-tbt",
//         "bbo-tbt",
//         "books",
//         "tickers",
//         "open-interest",
//         "mark-price",
//         "price-limit",
//         "funding-rate",
//         "status",
//         "instruments",
//         "index-tickers",
//         "long-short-account-ratio",
//         "taker-volume",
//         "liquidations",
//         "public-block-trades",
//         "public-struc-block-trades",
//         "liquidation-orders"
//     ]
// },

#[derive(Debug, Deserialize)]
pub struct OkexSwapsTrade {
    pub arg: OkexSwapsTradeArg,
    pub data: Vec<OkexSwapsTradeData>,
}

#[derive(Debug, Deserialize)]
pub struct OkexSwapsTradeArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub instrument: String,
}

#[derive(Debug, Deserialize)]
pub struct OkexSwapsTradeData {
    #[serde(rename = "instId")]
    pub instrument: String,
    #[serde(rename = "tradeId", deserialize_with = "as_u64")]
    pub trade_id: u64,
    #[serde(rename = "px", deserialize_with = "as_f64")]
    pub price: f64,
    #[serde(rename = "sz", deserialize_with = "as_u64")]
    pub quantity: u64,
    pub side: String,
    #[serde(rename = "ts", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
}

// https://www.okx.com/docs-v5/en/#order-book-trading-market-data-ws-order-book-channel
#[derive(Debug, Deserialize)]
pub struct OkexSwapsBook {
    pub arg: OkexSwapsBookArg,
    pub action: String,
    pub data: Vec<OkexSwapsBookData>,
}

#[derive(Debug, Deserialize)]
pub struct OkexSwapsBookArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub instrument: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkexSwapsBookData {
    pub asks: Vec<OkexSwapsBookUpdate>,
    pub bids: Vec<OkexSwapsBookUpdate>,
    #[serde(rename = "ts", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    pub checksum: i32,
    pub seq_id: i64,
    pub prev_seq_id: i64,
}

// An example of the array of asks and bids values: ["411.8", "10", "0", "4"]
// - "411.8" is the depth price
// - "10" is the quantity at the price (number of contracts for derivatives, quantity in base currency for Spot and Spot Margin)
// - "0" is part of a deprecated feature and it is always "0"
// - "4" is the number of orders at the price.
#[derive(Debug, Deserialize)]
pub struct OkexSwapsBookUpdate {
    #[serde(deserialize_with = "as_f64")]
    pub price: f64,
    #[serde(deserialize_with = "as_u64")]
    pub quantity: u64,
    #[serde(deserialize_with = "as_f64")]
    pub deprecated_feature: f64,
    #[serde(deserialize_with = "as_u64")]
    pub num_orders: u64,
}

#[derive(Debug, Deserialize)]
pub struct OkexSwapsOpenInterest {
    pub arg: OkexSwapsOpenInterestArg,
    pub data: Vec<OkexSwapsOpenInterestData>,
}

#[derive(Debug, Deserialize)]
pub struct OkexSwapsOpenInterestArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub instrument: String,
}

#[derive(Debug, Deserialize)]
pub struct OkexSwapsOpenInterestData {
    #[serde(rename = "ts", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    #[serde(rename = "instId")]
    pub instrument: String,
    #[serde(rename = "instType")]
    pub instrument_type: String,
    #[serde(rename = "oi", deserialize_with = "as_u64")]
    pub open_interest: u64,
    #[serde(rename = "oiCcy", deserialize_with = "as_f64")]
    pub open_interest_currency: f64,
}

#[derive(Debug, Deserialize)]
pub struct OkexSwapsFundingRate {
    pub arg: OkexSwapsFundingRateArg,
    pub data: Vec<OkexSwapsFundingRateData>,
}

#[derive(Debug, Deserialize)]
pub struct OkexSwapsFundingRateArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub instrument: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkexSwapsFundingRateData {
    #[serde(rename = "ts", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    #[serde(rename = "instId")]
    pub instrument: String,
    #[serde(rename = "instType")]
    pub instrument_type: String,
    #[serde(deserialize_with = "as_f64")]
    pub funding_rate: f64,
    #[serde(with = "custom_serde::timestamp")]
    pub funding_time: UtcDateTime,
    #[serde(deserialize_with = "as_f64")]
    pub max_funding_rate: f64,
    #[serde(deserialize_with = "as_f64")]
    pub min_funding_rate: f64,
    #[serde(deserialize_with = "as_f64")]
    pub next_funding_rate: f64,
    #[serde(with = "custom_serde::timestamp")]
    pub next_funding_time: UtcDateTime,
    #[serde(deserialize_with = "as_f64")]
    pub sett_funding_rate: f64,
    pub sett_state: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn test_okex_swap_trade() {
        let json_data = r#"{"arg":{"channel":"trades-all","instId":"FITFI-USDT-SWAP"},"data":[{"instId":"FITFI-USDT-SWAP","tradeId":"38380467","px":"0.007173","sz":"44","side":"buy","ts":"1701388800105"}]}"#;
        let _ = serde_json::from_str::<OkexSwapsTrade>(json_data).unwrap();
    }

    #[tokio::test]
    #[traced_test]
    async fn test_okex_swap_book() {
        let json_data = r#"{"arg":{"channel":"books","instId":"STX-USDT-SWAP"},"action":"update","data":[{"asks":[["0.692","463","0","9"],["0.6921","584","0","10"],["0.6923","591","0","10"],["0.6924","1385","0","9"],["0.6927","2801","0","11"]],"bids":[["0.6915","44","0","4"],["0.6906","2344","0","9"],["0.6903","197","0","7"],["0.6902","203","0","9"],["0.69","397","0","12"],["0.6895","192","0","6"],["0.6893","4088","0","6"]],"ts":"1701388800001","checksum":945112414,"seqId":1314875837,"prevSeqId":1314875825}]}"#;
        let _ = serde_json::from_str::<OkexSwapsBook>(json_data).unwrap();
    }

    #[tokio::test]
    #[traced_test]
    async fn test_okex_swap_open_interest() {
        let json_data = r#"{"arg":{"channel":"open-interest","instId":"CETUS-USDT-SWAP"},"data":[{"instId":"CETUS-USDT-SWAP","instType":"SWAP","oi":"4065123","oiCcy":"40651230","ts":"1701388800898"}]}"#;
        let _ = serde_json::from_str::<OkexSwapsOpenInterest>(json_data).unwrap();
    }

    #[tokio::test]
    #[traced_test]
    async fn test_okex_swap_funding_rate() {
        let json_data = r#"{"arg":{"channel":"funding-rate","instId":"ZIL-USDT-SWAP"},"data":[{"fundingRate":"0.0001841086622606","fundingTime":"1701417600000","instId":"ZIL-USDT-SWAP","instType":"SWAP","maxFundingRate":"0.015","minFundingRate":"-0.015","nextFundingRate":"0.0002285191956124","nextFundingTime":"1701446400000","settFundingRate":"0.0001137114217030","settState":"processing","ts":"1701388800035"}]}"#;
        let _ = serde_json::from_str::<OkexSwapsFundingRate>(json_data).unwrap();
    }
}
