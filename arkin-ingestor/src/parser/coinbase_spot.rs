use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct CoinbaseTrade {
    product_id: String,
    trade_id: String,
    price: Decimal,
    size: Decimal,
    time: String,
    side: String,
}
