use rust_decimal::Decimal;
use serde::Deserialize;
use time::UtcDateTime;

use arkin_core::prelude::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceDetails {
    pub account_alias: String,
    pub asset: String,
    pub balance: Decimal,
    pub cross_wallet_balance: Decimal,
    pub cross_un_pnl: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
    pub margin_available: bool,
    #[serde(with = "custom_serde::timestamp")]
    pub update_time: UtcDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use serde_json::from_str;

    #[test]
    fn test_account_balance() {
        let data = r#"
        [
          {
            "accountAlias": "SgsR",
            "asset": "USDT",
            "balance": "122607.35137903",
            "crossWalletBalance": "23.72469206",
            "crossUnPnl": "0.00000000",
            "availableBalance": "23.72469206",
            "maxWithdrawAmount": "23.72469206",
            "marginAvailable": true,
            "updateTime": 1617939110373
          }
        ]
        "#;

        let balances: Vec<BalanceDetails> = from_str(data).unwrap();
        assert_eq!(balances.len(), 1);
        let bal = &balances[0];
        assert_eq!(bal.account_alias, "SgsR");
        assert_eq!(bal.asset, "USDT");
        assert_eq!(bal.balance, dec!(122607.35137903));
        assert!(bal.margin_available);
        assert_eq!(
            bal.update_time,
            UtcDateTime::from_unix_timestamp_nanos(1617939110373 * 1_000_000).unwrap()
        );
    }
}
