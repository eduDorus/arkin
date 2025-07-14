use rust_decimal::Decimal;
use serde::Deserialize;
use time::UtcDateTime;

use arkin_core::prelude::*;

use super::user_models::BinancePositionSide;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSnapshot {
    pub total_initial_margin: Decimal,
    pub total_maint_margin: Decimal,
    pub total_wallet_balance: Decimal,
    pub total_unrealized_profit: Decimal,
    pub total_margin_balance: Decimal,
    pub total_position_initial_margin: Decimal,
    pub total_open_order_initial_margin: Decimal,
    pub total_cross_wallet_balance: Decimal,
    pub total_cross_un_pnl: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
    pub assets: Vec<AssetInfo>,
    pub positions: Vec<PositionInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetInfo {
    pub asset: String,
    pub wallet_balance: Decimal,
    pub unrealized_profit: Decimal,
    pub margin_balance: Decimal,
    pub maint_margin: Decimal,
    pub initial_margin: Decimal,
    pub position_initial_margin: Decimal,
    pub open_order_initial_margin: Decimal,
    pub cross_wallet_balance: Decimal,
    pub cross_un_pnl: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
    #[serde(default)]
    pub margin_available: Option<bool>,
    #[serde(with = "custom_serde::timestamp")]
    pub update_time: UtcDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionInfo {
    pub symbol: String,
    pub position_side: BinancePositionSide,
    pub position_amt: Decimal,
    pub unrealized_profit: Decimal,
    pub isolated_margin: Decimal,
    pub notional: Decimal,
    pub isolated_wallet: Decimal,
    pub initial_margin: Decimal,
    pub maint_margin: Decimal,
    #[serde(with = "custom_serde::timestamp")]
    pub update_time: UtcDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use serde_json::from_str;

    #[test]
    fn test_single_asset_mode() {
        let data = r#"
        {
            "totalInitialMargin": "0.00000000",
            "totalMaintMargin": "0.00000000",
            "totalWalletBalance": "103.12345678",
            "totalUnrealizedProfit": "0.00000000",
            "totalMarginBalance": "103.12345678",
            "totalPositionInitialMargin": "0.00000000",
            "totalOpenOrderInitialMargin": "0.00000000",
            "totalCrossWalletBalance": "103.12345678",
            "totalCrossUnPnl": "0.00000000",
            "availableBalance": "103.12345678",
            "maxWithdrawAmount": "103.12345678",
            "assets": [
                {
                    "asset": "USDT",
                    "walletBalance": "23.72469206",
                    "unrealizedProfit": "0.00000000",
                    "marginBalance": "23.72469206",
                    "maintMargin": "0.00000000",
                    "initialMargin": "0.00000000",
                    "positionInitialMargin": "0.00000000",
                    "openOrderInitialMargin": "0.00000000",
                    "crossWalletBalance": "23.72469206",
                    "crossUnPnl": "0.00000000",
                    "availableBalance": "23.72469206",
                    "maxWithdrawAmount": "23.72469206",
                    "updateTime": 1625474304765
                },
                {
                    "asset": "USDC",
                    "walletBalance": "103.12345678",
                    "unrealizedProfit": "0.00000000",
                    "marginBalance": "103.12345678",
                    "maintMargin": "0.00000000",
                    "initialMargin": "0.00000000",
                    "positionInitialMargin": "0.00000000",
                    "openOrderInitialMargin": "0.00000000",
                    "crossWalletBalance": "103.12345678",
                    "crossUnPnl": "0.00000000",
                    "availableBalance": "126.72469206",
                    "maxWithdrawAmount": "103.12345678",
                    "updateTime": 1625474304765
                }
            ],
            "positions": [
                {
                    "symbol": "BTCUSDT",
                    "positionSide": "BOTH",
                    "positionAmt": "1.000",
                    "unrealizedProfit": "0.00000000",
                    "isolatedMargin": "0.00000000",
                    "notional": "0",
                    "isolatedWallet": "0",
                    "initialMargin": "0",
                    "maintMargin": "0",
                    "updateTime": 0
                }
            ]
        }"#;

        let snapshot: AccountSnapshot = from_str(data).unwrap();
        assert_eq!(snapshot.total_wallet_balance, dec!(103.12345678));
        assert_eq!(snapshot.assets.len(), 2);
        assert!(snapshot.assets[0].margin_available.is_none()); // Single asset mode: no marginAvailable field
        assert_eq!(snapshot.positions.len(), 1);
        assert_eq!(snapshot.positions[0].symbol, "BTCUSDT");
    }

    #[test]
    fn test_multi_asset_mode() {
        let data = r#"
        {
            "totalInitialMargin": "0.00000000",
            "totalMaintMargin": "0.00000000",
            "totalWalletBalance": "126.72469206",
            "totalUnrealizedProfit": "0.00000000",
            "totalMarginBalance": "126.72469206",
            "totalPositionInitialMargin": "0.00000000",
            "totalOpenOrderInitialMargin": "0.00000000",
            "totalCrossWalletBalance": "126.72469206",
            "totalCrossUnPnl": "0.00000000",
            "availableBalance": "126.72469206",
            "maxWithdrawAmount": "126.72469206",
            "assets": [
                {
                    "asset": "USDT",
                    "walletBalance": "23.72469206",
                    "unrealizedProfit": "0.00000000",
                    "marginBalance": "23.72469206",
                    "maintMargin": "0.00000000",
                    "initialMargin": "0.00000000",
                    "positionInitialMargin": "0.00000000",
                    "openOrderInitialMargin": "0.00000000",
                    "crossWalletBalance": "23.72469206",
                    "crossUnPnl": "0.00000000",
                    "availableBalance": "126.72469206",
                    "maxWithdrawAmount": "23.72469206",
                    "marginAvailable": true,
                    "updateTime": 1625474304765
                },
                {
                    "asset": "BUSD",
                    "walletBalance": "103.12345678",
                    "unrealizedProfit": "0.00000000",
                    "marginBalance": "103.12345678",
                    "maintMargin": "0.00000000",
                    "initialMargin": "0.00000000",
                    "positionInitialMargin": "0.00000000",
                    "openOrderInitialMargin": "0.00000000",
                    "crossWalletBalance": "103.12345678",
                    "crossUnPnl": "0.00000000",
                    "availableBalance": "126.72469206",
                    "maxWithdrawAmount": "103.12345678",
                    "marginAvailable": true,
                    "updateTime": 1625474304765
                }
            ],
            "positions": [
                {
                    "symbol": "BTCUSDT",
                    "positionSide": "BOTH",
                    "positionAmt": "1.000",
                    "unrealizedProfit": "0.00000000",
                    "isolatedMargin": "0.00000000",
                    "notional": "0",
                    "isolatedWallet": "0",
                    "initialMargin": "0",
                    "maintMargin": "0",
                    "updateTime": 0
                }
            ]
        }"#;

        let snapshot: AccountSnapshot = from_str(data).unwrap();
        assert_eq!(snapshot.total_wallet_balance, dec!(126.72469206));
        assert_eq!(snapshot.assets.len(), 2);
        assert!(snapshot.assets[0].margin_available.unwrap());
        assert!(snapshot.assets[1].margin_available.unwrap());
        assert_eq!(snapshot.positions.len(), 1);
        assert_eq!(snapshot.positions[0].symbol, "BTCUSDT");
    }
}
