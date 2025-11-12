use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize)]
struct OkxTrade {
    #[serde(rename = "instId")]
    inst_id: String,
    #[serde(rename = "px")]
    price: Decimal,
    #[serde(rename = "sz")]
    quantity: Decimal,
    side: String,
    #[serde(rename = "ts")]
    timestamp_str: String,
    #[serde(rename = "tradeId")]
    trade_id: String,
}

#[derive(Deserialize)]
struct OkxTicker {
    #[serde(rename = "instId")]
    inst_id: String,
    #[serde(rename = "bidPx")]
    bid_px: Decimal,
    #[serde(rename = "bidSz")]
    bid_sz: Decimal,
    #[serde(rename = "askPx")]
    ask_px: Decimal,
    #[serde(rename = "askSz")]
    ask_sz: Decimal,
    #[serde(rename = "last")]
    last_price: Decimal,
    #[serde(rename = "high24h")]
    high_24h: Decimal,
    #[serde(rename = "low24h")]
    low_24h: Decimal,
    #[serde(rename = "vol24h")]
    vol_24h: Decimal,
    #[serde(rename = "volCcy24h")]
    vol_ccy_24h: Decimal,
    #[serde(rename = "ts")]
    timestamp_str: String,
}

#[derive(Deserialize)]
struct OkxFundingRate {
    #[serde(rename = "instId")]
    inst_id: String,
    #[serde(rename = "fundingRate")]
    funding_rate: Decimal,
    #[serde(rename = "ts")]
    timestamp_str: String,
}
