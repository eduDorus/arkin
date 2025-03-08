use rust_decimal::Decimal;
use serde::Deserialize;
use time::OffsetDateTime;

use arkin_core::prelude::*;

use super::user_models::BinancePositionSide;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionDetail {
    pub symbol: String,
    pub position_side: BinancePositionSide,
    pub position_amt: Decimal,
    pub entry_price: Decimal,
    pub break_even_price: Decimal,
    pub mark_price: Decimal,
    pub un_realized_profit: Decimal,
    pub liquidation_price: Decimal,
    pub isolated_margin: Decimal,
    pub notional: Decimal,
    pub margin_asset: String,
    pub isolated_wallet: Decimal,
    pub initial_margin: Decimal,
    pub maint_margin: Decimal,
    pub position_initial_margin: Decimal,
    pub open_order_initial_margin: Decimal,
    pub adl: i64,
    pub bid_notional: Decimal,
    pub ask_notional: Decimal,
    #[serde(with = "custom_serde::timestamp")]
    pub update_time: OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use serde_json::from_str;

    #[test]
    fn test_one_way_mode_positions() {
        let data = r#"
        [
          {
                "symbol": "ADAUSDT",
                "positionSide": "BOTH",
                "positionAmt": "30",
                "entryPrice": "0.385",
                "breakEvenPrice": "0.385077",
                "markPrice": "0.41047590",
                "unRealizedProfit": "0.76427700",
                "liquidationPrice": "0",
                "isolatedMargin": "0",
                "notional": "12.31427700",
                "marginAsset": "USDT",
                "isolatedWallet": "0",
                "initialMargin": "0.61571385",
                "maintMargin": "0.08004280",
                "positionInitialMargin": "0.61571385",
                "openOrderInitialMargin": "0",
                "adl": 2,
                "bidNotional": "0",
                "askNotional": "0",
                "updateTime": 1720736417660
          }
        ]
        "#;
        let positions: Vec<PositionDetail> = from_str(data).unwrap();
        assert_eq!(positions.len(), 1);
        let pos = &positions[0];
        assert_eq!(pos.symbol, "ADAUSDT");
        assert_eq!(pos.position_side, BinancePositionSide::Both);
        assert_eq!(pos.un_realized_profit, dec!(0.76427700));
        assert_eq!(pos.adl, 2);
    }

    #[test]
    fn test_hedge_mode_positions() {
        let data = r#"
        [
          {
                "symbol": "ADAUSDT",
                "positionSide": "LONG",
                "positionAmt": "30",
                "entryPrice": "0.385",
                "breakEvenPrice": "0.385077",
                "markPrice": "0.41047590",
                "unRealizedProfit": "0.76427700",
                "liquidationPrice": "0",
                "isolatedMargin": "0",
                "notional": "12.31427700",
                "marginAsset": "USDT",
                "isolatedWallet": "0",
                "initialMargin": "0.61571385",
                "maintMargin": "0.08004280",
                "positionInitialMargin": "0.61571385",
                "openOrderInitialMargin": "0",
                "adl": 2,
                "bidNotional": "0",
                "askNotional": "0",
                "updateTime": 1720736417660
          },
          {
                "symbol": "COMPUSDT",
                "positionSide": "SHORT",
                "positionAmt": "-1.000",
                "entryPrice": "70.92841",
                "breakEvenPrice": "70.900038636",
                "markPrice": "49.72023376",
                "unRealizedProfit": "21.20817624",
                "liquidationPrice": "2260.56757210",
                "isolatedMargin": "0",
                "notional": "-49.72023376",
                "marginAsset": "USDT",
                "isolatedWallet": "0",
                "initialMargin": "2.48601168",
                "maintMargin": "0.49720233",
                "positionInitialMargin": "2.48601168",
                "openOrderInitialMargin": "0",
                "adl": 2,
                "bidNotional": "0",
                "askNotional": "0",
                "updateTime": 1708943511656
          }
        ]
        "#;

        let positions: Vec<PositionDetail> = from_str(data).unwrap();
        assert_eq!(positions.len(), 2);

        let pos1 = &positions[0];
        assert_eq!(pos1.symbol, "ADAUSDT");
        assert_eq!(pos1.position_side, BinancePositionSide::Long);
        assert_eq!(pos1.un_realized_profit, dec!(0.76427700));

        let pos2 = &positions[1];
        assert_eq!(pos2.symbol, "COMPUSDT");
        assert_eq!(pos2.position_side, BinancePositionSide::Short);
        assert_eq!(pos2.un_realized_profit, dec!(21.20817624));
        assert_eq!(pos2.initial_margin, dec!(2.48601168));
    }
}
