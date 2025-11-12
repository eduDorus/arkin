use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize)]
struct BinanceAggregateTrade {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "a")]
    trade_id: u64,
    #[serde(rename = "p")]
    price: Decimal,
    #[serde(rename = "q")]
    quantity: Decimal,
    #[serde(rename = "T")]
    trade_time: u64,
    #[serde(rename = "m")]
    is_buyer_maker: bool,
}
