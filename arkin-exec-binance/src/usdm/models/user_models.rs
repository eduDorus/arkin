use rust_decimal::prelude::*;

use arkin_core::prelude::*;

use serde::Deserialize;
use time::UtcDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinancePositionSide {
    Long,
    Short,
    Both,
}

// impl From<BinancePositionSide> for PositionSide {
//     fn from(side: BinancePositionSide) -> Self {
//         match side {
//             BinancePositionSide::Long => PositionSide::Long,
//             BinancePositionSide::Short => PositionSide::Short,
//             BinancePositionSide::Both => ,
//         }
//     }
// }

// All uppercase
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountUpdateReason {
    Deposit,
    Withdraw,
    Order,
    FundingFee,
    WithdrawReject,
    Adjustment,
    InsuranceClear,
    AdminDeposit,
    AdminWithdraw,
    MarginTransfer,
    MarginTypeChange,
    AssetTransfer,
    OptionsPremiumFee,
    OptionsSettleProfit,
    AutoExchange,
    CoinSwapDeposit,
    CoinSwapWithdraw,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinanceOrderSide {
    Buy,
    Sell,
}

impl From<BinanceOrderSide> for MarketSide {
    fn from(side: BinanceOrderSide) -> Self {
        match side {
            BinanceOrderSide::Buy => MarketSide::Buy,
            BinanceOrderSide::Sell => MarketSide::Sell,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinanceOrderType {
    Limit,
    Market,
    Stop,
    StopMarket,
    TakeProfit,
    TakeProfitMarket,
    TrailingStopMarket,
}

impl From<BinanceOrderType> for VenueOrderType {
    fn from(order_type: BinanceOrderType) -> Self {
        match order_type {
            BinanceOrderType::Limit => VenueOrderType::Limit,
            BinanceOrderType::Market => VenueOrderType::Market,
            BinanceOrderType::Stop => VenueOrderType::StopLimit,
            BinanceOrderType::StopMarket => VenueOrderType::StopMarket,
            BinanceOrderType::TakeProfit => VenueOrderType::TakeProfit,
            BinanceOrderType::TakeProfitMarket => VenueOrderType::TakeProfitMarket,
            BinanceOrderType::TrailingStopMarket => VenueOrderType::TrailingStopMarket,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinanceExecutionType {
    New,
    Canceled,
    Calculated, // The order is being calculated liquidation execution
    Expired,
    Trade,
    Amendment, // The order is being amended/modified
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinanceOrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
    ExpiredInMatch,
}

impl From<BinanceOrderStatus> for VenueOrderStatus {
    fn from(status: BinanceOrderStatus) -> Self {
        match status {
            BinanceOrderStatus::New => VenueOrderStatus::New,
            BinanceOrderStatus::PartiallyFilled => VenueOrderStatus::PartiallyFilled,
            BinanceOrderStatus::Filled => VenueOrderStatus::Filled,
            BinanceOrderStatus::Canceled => VenueOrderStatus::Cancelled,
            BinanceOrderStatus::Rejected => VenueOrderStatus::Rejected,
            BinanceOrderStatus::Expired => VenueOrderStatus::Expired,
            BinanceOrderStatus::ExpiredInMatch => VenueOrderStatus::Expired,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinanceTimeInForce {
    Gtc,
    Ioc,
    Fok,
    Gtx,
}

impl From<BinanceTimeInForce> for VenueOrderTimeInForce {
    fn from(time_in_force: BinanceTimeInForce) -> Self {
        match time_in_force {
            BinanceTimeInForce::Gtc => VenueOrderTimeInForce::Gtc,
            BinanceTimeInForce::Ioc => VenueOrderTimeInForce::Ioc,
            BinanceTimeInForce::Fok => VenueOrderTimeInForce::Fok,
            BinanceTimeInForce::Gtx => VenueOrderTimeInForce::Gtx,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BinanceWorkingType {
    MarkPrice,
    ContractPrice,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "e")]
pub enum BinanceUSDMUserStreamEvent {
    #[serde(rename = "ORDER_TRADE_UPDATE")]
    OrderTradeUpdate {
        #[serde(rename = "E", with = "custom_serde::timestamp")]
        event_time: UtcDateTime,
        #[serde(rename = "T", with = "custom_serde::timestamp")]
        transaction_time: UtcDateTime,
        #[serde(rename = "o")]
        order: OrderData,
    },
    #[serde(rename = "ACCOUNT_UPDATE")]
    AccountUpdate {
        #[serde(rename = "E", with = "custom_serde::timestamp")]
        event_time: UtcDateTime,
        #[serde(rename = "T", with = "custom_serde::timestamp")]
        transaction_time: UtcDateTime,
        #[serde(rename = "a")]
        account: AccountUpdateData,
    },
    #[serde(rename = "TRADE_LITE")]
    TradeLite {
        #[serde(rename = "E", with = "custom_serde::timestamp")]
        event_time: UtcDateTime,
        #[serde(rename = "T", with = "custom_serde::timestamp")]
        transaction_time: UtcDateTime,
        #[serde(rename = "s")]
        symbol: String,
        #[serde(rename = "q")]
        original_quantity: Decimal,
        #[serde(rename = "p")]
        original_price: Decimal,
        #[serde(rename = "m")]
        is_maker_side: bool,
        #[serde(rename = "c")]
        client_order_id: String,
        #[serde(rename = "S")]
        side: BinanceOrderSide,
        #[serde(rename = "L")]
        last_filled_price: Decimal,
        #[serde(rename = "l")]
        last_filled_quantity: Decimal,
        #[serde(rename = "t")]
        trade_id: i64,
        #[serde(rename = "i")]
        order_id: i64,
    },
    #[serde(rename = "MARGIN_CALL")]
    MarginCall {
        #[serde(rename = "E", with = "custom_serde::timestamp")]
        event_time: UtcDateTime,
        #[serde(rename = "cw")]
        cross_wallet_balance: String,
        #[serde(rename = "p")]
        positions: Vec<MarginCallPosition>,
    },
}

#[derive(Debug, Deserialize)]
pub struct OrderData {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub client_order_id: String,
    #[serde(rename = "S")]
    pub side: BinanceOrderSide,
    #[serde(rename = "o")]
    pub order_type: BinanceOrderType,
    #[serde(rename = "f")]
    pub time_in_force: BinanceTimeInForce,
    #[serde(rename = "q")]
    pub original_quantity: Decimal,
    #[serde(rename = "p")]
    pub original_price: Decimal,
    #[serde(rename = "ap")]
    pub average_price: Decimal,
    #[serde(rename = "sp")]
    pub stop_price: Decimal,
    #[serde(rename = "x")]
    pub execution_type: BinanceExecutionType,
    #[serde(rename = "X")]
    pub order_status: BinanceOrderStatus,
    #[serde(rename = "i")]
    pub order_id: i64,
    #[serde(rename = "l")]
    pub last_filled_quantity: Decimal,
    #[serde(rename = "z")]
    pub filled_accumulated_quantity: Decimal,
    #[serde(rename = "L")]
    pub last_filled_price: Decimal,
    #[serde(rename = "N")]
    pub commission_asset: Option<String>,
    #[serde(rename = "n")]
    pub commission: Option<Decimal>,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub order_trade_time: UtcDateTime,
    #[serde(rename = "t")]
    pub trade_id: i64,
    #[serde(rename = "b")]
    pub bids_notional: Decimal,
    #[serde(rename = "a")]
    pub asks_notional: Decimal,
    #[serde(rename = "m")]
    pub is_maker_side: bool,
    #[serde(rename = "R")]
    pub is_reduce_only: bool,
    #[serde(rename = "wt")]
    pub stop_price_working_type: BinanceWorkingType,
    #[serde(rename = "ot")]
    pub original_order_type: BinanceOrderType,
    #[serde(rename = "ps")]
    pub position_side: BinancePositionSide,
    #[serde(rename = "cp")]
    pub close_all: bool,
    #[serde(rename = "AP")]
    pub activation_price: Option<Decimal>,
    #[serde(rename = "cr")]
    pub callback_rate: Option<Decimal>,
    #[serde(rename = "pP")]
    pub price_protect: bool,
    #[serde(rename = "si")]
    #[serde(skip)]
    pub ignore_si: i64,
    #[serde(rename = "ss")]
    #[serde(skip)]
    pub ignore_ss: i64,
    #[serde(rename = "rp")]
    pub realized_profit: Decimal,
    #[serde(rename = "V")]
    pub stp_mode: String,
    #[serde(rename = "pm")]
    pub price_match_mode: String,
    #[serde(rename = "gtd")]
    pub gtd_time: i64,
}

#[derive(Debug, Deserialize)]
pub struct AccountUpdateData {
    #[serde(rename = "m")]
    pub event_reason_type: AccountUpdateReason,
    #[serde(rename = "B")]
    pub balances: Vec<Balance>,
    #[serde(rename = "P")]
    pub positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
pub struct Balance {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "wb")]
    pub wallet_balance: Decimal,
    #[serde(rename = "cw")]
    pub cross_wallet_balance: Decimal,
    #[serde(rename = "bc")]
    pub balance_change: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct Position {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "pa")]
    pub position_amount: Decimal,
    #[serde(rename = "ep")]
    pub entry_price: Decimal,
    #[serde(rename = "bep")]
    pub breakeven_price: Decimal,
    #[serde(rename = "cr")]
    pub accumulated_realized: Decimal,
    #[serde(rename = "up")]
    pub unrealized_pnl: Decimal,
    #[serde(rename = "mt")]
    pub margin_type: String,
    #[serde(rename = "iw")]
    pub isolated_wallet: Decimal,
    #[serde(rename = "ps")]
    pub position_side: BinancePositionSide,
}

#[derive(Debug, Deserialize)]
pub struct MarginCallPosition {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "ps")]
    pub position_side: String,
    #[serde(rename = "pa")]
    pub position_amount: Decimal,
    #[serde(rename = "mt")]
    pub margin_type: String,
    #[serde(rename = "iw")]
    pub isolated_wallet: Decimal,
    #[serde(rename = "mp")]
    pub mark_price: Decimal,
    #[serde(rename = "up")]
    pub unrealized_pnl: Decimal,
    #[serde(rename = "mm")]
    pub maintenance_margin_required: Decimal,
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
                UtcDateTime::from_unix_timestamp_nanos(1733989303534 * 1_000_000).unwrap()
            );
            assert_eq!(
                transaction_time,
                UtcDateTime::from_unix_timestamp_nanos(1733989303533 * 1_000_000).unwrap()
            );
            assert_eq!(order.symbol, "ETHUSDT");
            assert_eq!(order.side, BinanceOrderSide::Sell);
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
                UtcDateTime::from_unix_timestamp_nanos(1733988745974 * 1_000_000).unwrap()
            );
            assert_eq!(
                transaction_time,
                UtcDateTime::from_unix_timestamp_nanos(1733988745973 * 1_000_000).unwrap()
            );
            assert_eq!(account.event_reason_type, AccountUpdateReason::Order);
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
                UtcDateTime::from_unix_timestamp_nanos(1733988745974 * 1_000_000).unwrap()
            );
            assert_eq!(
                transaction_time,
                UtcDateTime::from_unix_timestamp_nanos(1733988745973 * 1_000_000).unwrap()
            );
            assert_eq!(symbol, "ETHUSDT");
            assert_eq!(original_quantity, dec!(0.006));
            assert_eq!(original_price, dec!(0.00));
            assert!(!is_maker_side);
            assert_eq!(client_order_id, "29d41430-9b98-4310-a471-8769b5dfb512");
            assert_eq!(side, BinanceOrderSide::Buy);
            assert_eq!(last_filled_price, dec!(3925.56));
            assert_eq!(last_filled_quantity, dec!(0.006));
            assert_eq!(trade_id, 4830037678);
            assert_eq!(order_id, 8389765792662662763);
        } else {
            panic!("Not a TradeLite event");
        }
    }
}
