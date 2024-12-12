#![allow(dead_code)]
use rust_decimal::Decimal;

use arkin_core::prelude::*;

use serde::Deserialize;
use time::OffsetDateTime;

#[derive(Debug, Deserialize)]
#[serde(tag = "e")]
pub enum BinanceUSDMUserStreamEvent {
    #[serde(rename = "ORDER_TRADE_UPDATE")]
    OrderTradeUpdate {
        #[serde(rename = "E", with = "custom_serde::timestamp")]
        event_time: OffsetDateTime,
        #[serde(rename = "T", with = "custom_serde::timestamp")]
        transaction_time: OffsetDateTime,
        #[serde(rename = "o")]
        order: OrderData,
    },
    #[serde(rename = "ACCOUNT_UPDATE")]
    AccountUpdate {
        #[serde(rename = "E", with = "custom_serde::timestamp")]
        event_time: OffsetDateTime,
        #[serde(rename = "T", with = "custom_serde::timestamp")]
        transaction_time: OffsetDateTime,
        #[serde(rename = "a")]
        account: AccountUpdateData,
    },
    #[serde(rename = "TRADE_LITE")]
    TradeLite {
        #[serde(rename = "E", with = "custom_serde::timestamp")]
        event_time: OffsetDateTime,
        #[serde(rename = "T", with = "custom_serde::timestamp")]
        transaction_time: OffsetDateTime,
        #[serde(rename = "s")]
        symbol: String,
        #[serde(rename = "q")]
        original_quantity: String,
        #[serde(rename = "p")]
        original_price: String,
        #[serde(rename = "m")]
        is_maker_side: bool,
        #[serde(rename = "c")]
        client_order_id: String,
        #[serde(rename = "S")]
        side: String,
        #[serde(rename = "L")]
        last_filled_price: String,
        #[serde(rename = "l")]
        last_filled_quantity: String,
        #[serde(rename = "t")]
        trade_id: i64,
        #[serde(rename = "i")]
        order_id: i64,
    },
    #[serde(rename = "MARGIN_CALL")]
    MarginCall {
        #[serde(rename = "E", with = "custom_serde::timestamp")]
        event_time: OffsetDateTime,
        #[serde(rename = "cw")]
        cross_wallet_balance: String,
        #[serde(rename = "p")]
        positions: Vec<MarginCallPosition>,
    },
}

#[derive(Debug, Deserialize)]
pub struct OrderData {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "c")]
    client_order_id: String,
    #[serde(rename = "S")]
    side: String,
    #[serde(rename = "o")]
    order_type: String,
    #[serde(rename = "f")]
    time_in_force: String,
    #[serde(rename = "q")]
    original_quantity: Decimal,
    #[serde(rename = "p")]
    original_price: Decimal,
    #[serde(rename = "ap")]
    average_price: Decimal,
    #[serde(rename = "sp")]
    stop_price: Decimal,
    #[serde(rename = "x")]
    execution_type: String,
    #[serde(rename = "X")]
    order_status: String,
    #[serde(rename = "i")]
    order_id: i64,
    #[serde(rename = "l")]
    last_filled_quantity: Decimal,
    #[serde(rename = "z")]
    filled_accumulated_quantity: Decimal,
    #[serde(rename = "L")]
    last_filled_price: Decimal,
    #[serde(rename = "N")]
    commission_asset: Option<String>,
    #[serde(rename = "n")]
    commission: Option<Decimal>,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    order_trade_time: OffsetDateTime,
    #[serde(rename = "t")]
    trade_id: i64,
    #[serde(rename = "b")]
    bids_notional: Decimal,
    #[serde(rename = "a")]
    asks_notional: Decimal,
    #[serde(rename = "m")]
    is_maker_side: bool,
    #[serde(rename = "R")]
    is_reduce_only: bool,
    #[serde(rename = "wt")]
    stop_price_working_type: String,
    #[serde(rename = "ot")]
    original_order_type: String,
    #[serde(rename = "ps")]
    position_side: String,
    #[serde(rename = "cp")]
    close_all: bool,
    #[serde(rename = "AP")]
    activation_price: Option<Decimal>,
    #[serde(rename = "cr")]
    callback_rate: Option<Decimal>,
    #[serde(rename = "pP")]
    price_protect: bool,
    #[serde(rename = "si")]
    #[serde(skip)]
    ignore_si: i64,
    #[serde(rename = "ss")]
    #[serde(skip)]
    ignore_ss: i64,
    #[serde(rename = "rp")]
    realized_profit: Decimal,
    #[serde(rename = "V")]
    stp_mode: String,
    #[serde(rename = "pm")]
    price_match_mode: String,
    #[serde(rename = "gtd")]
    gtd_time: i64,
}

#[derive(Debug, Deserialize)]
pub struct AccountUpdateData {
    #[serde(rename = "m")]
    event_reason_type: String,
    #[serde(rename = "B")]
    balances: Vec<Balance>,
    #[serde(rename = "P")]
    positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
pub struct Balance {
    #[serde(rename = "a")]
    asset: String,
    #[serde(rename = "wb")]
    wallet_balance: Decimal,
    #[serde(rename = "cw")]
    cross_wallet_balance: Decimal,
    #[serde(rename = "bc")]
    balance_change: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct Position {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "pa")]
    position_amount: Decimal,
    #[serde(rename = "ep")]
    entry_price: Decimal,
    #[serde(rename = "bep")]
    breakeven_price: Decimal,
    #[serde(rename = "cr")]
    accumulated_realized: Decimal,
    #[serde(rename = "up")]
    unrealized_pnl: Decimal,
    #[serde(rename = "mt")]
    margin_type: String,
    #[serde(rename = "iw")]
    isolated_wallet: Decimal,
    #[serde(rename = "ps")]
    position_side: String,
}

#[derive(Debug, Deserialize)]
pub struct MarginCallPosition {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "ps")]
    position_side: String,
    #[serde(rename = "pa")]
    position_amount: Decimal,
    #[serde(rename = "mt")]
    margin_type: String,
    #[serde(rename = "iw")]
    isolated_wallet: Decimal,
    #[serde(rename = "mp")]
    mark_price: Decimal,
    #[serde(rename = "up")]
    unrealized_pnl: Decimal,
    #[serde(rename = "mm")]
    maintenance_margin_required: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_trade_update_new() {
        let data = r#"
        {
          "e": "ORDER_TRADE_UPDATE",
          "T": 1733989303533,
          "E": 1733989303534,
          "o": {
            "s": "ETHUSDT",
            "c": "web_FOkuMm812fLFmAIwwjFs",
            "S": "SELL",
            "o": "MARKET",
            "f": "GTC",
            "q": "0.006",
            "p": "0",
            "ap": "0",
            "sp": "0",
            "x": "NEW",
            "X": "NEW",
            "i": 8389765792666894988,
            "l": "0",
            "z": "0",
            "L": "0",
            "n": "0",
            "N": "USDT",
            "T": 1733989303533,
            "t": 0,
            "b": "0",
            "a": "0",
            "m": false,
            "R": true,
            "wt": "CONTRACT_PRICE",
            "ot": "MARKET",
            "ps": "BOTH",
            "cp": false,
            "rp": "0",
            "pP": false,
            "si": 0,
            "ss": 0,
            "V": "EXPIRE_MAKER",
            "pm": "NONE",
            "gtd": 0
          }
        }"#;

        let parsed: BinanceUSDMUserStreamEvent = serde_json::from_str(data).unwrap();
        if let BinanceUSDMUserStreamEvent::OrderTradeUpdate {
            event_time,
            transaction_time,
            order,
        } = parsed
        {
            assert_eq!(
                event_time,
                OffsetDateTime::from_unix_timestamp_nanos(1733989303534 * 1_000_000).unwrap()
            );
            assert_eq!(
                transaction_time,
                OffsetDateTime::from_unix_timestamp_nanos(1733989303533 * 1_000_000).unwrap()
            );
            assert_eq!(order.symbol, "ETHUSDT");
            assert_eq!(order.side, "SELL");
            assert!(order.is_reduce_only);
        } else {
            panic!("Not an OrderTradeUpdate event");
        }
    }

    #[test]
    fn test_account_update() {
        let data = r#"
        {
          "e": "ACCOUNT_UPDATE",
          "T": 1733988745973,
          "E": 1733988745974,
          "a": {
            "B": [
              {
                "a": "USDT",
                "wb": "199.98822332",
                "cw": "199.98822332",
                "bc": "0"
              }
            ],
            "P": [
              {
                "s": "ETHUSDT",
                "pa": "0.006",
                "ep": "3925.56",
                "cr": "0",
                "up": "-0.00003100",
                "mt": "cross",
                "iw": "0",
                "ps": "BOTH",
                "ma": "USDT",
                "bep": "3927.52278"
              }
            ],
            "m": "ORDER"
          }
        }"#;

        let parsed: BinanceUSDMUserStreamEvent = serde_json::from_str(data).unwrap();
        if let BinanceUSDMUserStreamEvent::AccountUpdate {
            event_time,
            transaction_time,
            account,
        } = parsed
        {
            assert_eq!(
                event_time,
                OffsetDateTime::from_unix_timestamp_nanos(1733988745974 * 1_000_000).unwrap()
            );
            assert_eq!(
                transaction_time,
                OffsetDateTime::from_unix_timestamp_nanos(1733988745973 * 1_000_000).unwrap()
            );
            assert_eq!(account.event_reason_type, "ORDER");
            assert_eq!(account.balances.len(), 1);
            assert_eq!(account.balances[0].asset, "USDT");
            assert_eq!(account.positions.len(), 1);
            assert_eq!(account.positions[0].symbol, "ETHUSDT");
        } else {
            panic!("Not an AccountUpdate event");
        }
    }

    #[test]
    fn test_trade_lite() {
        let data = r#"
        {
          "e": "TRADE_LITE",
          "E": 1733988745974,
          "T": 1733988745973,
          "s": "ETHUSDT",
          "q": "0.006",
          "p": "0.00",
          "m": false,
          "c": "29d41430-9b98-4310-a471-8769b5dfb512",
          "S": "BUY",
          "L": "3925.56",
          "l": "0.006",
          "t": 4830037678,
          "i": 8389765792662662763
        }
        "#;

        let parsed: BinanceUSDMUserStreamEvent = serde_json::from_str(data).unwrap();
        if let BinanceUSDMUserStreamEvent::TradeLite {
            event_time,
            transaction_time,
            symbol,
            original_quantity,
            original_price,
            is_maker_side,
            client_order_id,
            side,
            last_filled_price,
            last_filled_quantity,
            trade_id,
            order_id,
        } = parsed
        {
            assert_eq!(
                event_time,
                OffsetDateTime::from_unix_timestamp_nanos(1733988745974 * 1_000_000).unwrap()
            );
            assert_eq!(
                transaction_time,
                OffsetDateTime::from_unix_timestamp_nanos(1733988745973 * 1_000_000).unwrap()
            );
            assert_eq!(symbol, "ETHUSDT");
            assert_eq!(original_quantity, "0.006");
            assert_eq!(original_price, "0.00");
            assert!(!is_maker_side);
            assert_eq!(client_order_id, "29d41430-9b98-4310-a471-8769b5dfb512");
            assert_eq!(side, "BUY");
            assert_eq!(last_filled_price, "3925.56");
            assert_eq!(last_filled_quantity, "0.006");
            assert_eq!(trade_id, 4830037678);
            assert_eq!(order_id, 8389765792662662763);
        } else {
            panic!("Not a TradeLite event");
        }
    }
}
