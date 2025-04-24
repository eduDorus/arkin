use std::sync::Arc;

use arkin_accounting::prelude::*;
use arkin_core::prelude::*;
use rust_decimal_macros::dec;
use test_log::test;
use time::macros::datetime;
use tracing::info;

#[test(tokio::test)]
async fn setup_wallets() {
    // Setup Simulation Clock
    let clock = Arc::new(SimulationModeClock::new(
        datetime!(2025-03-01 00:00:00).assume_utc(),
        datetime!(2025-03-02 00:00:00).assume_utc(),
    ));

    // Setup Pubsub
    let pubsub = Arc::new(PubSub::builder().clock(clock).build());

    // Setup Accounting
    let accounting = LedgerAccounting::builder()
        .pubsub(pubsub.handle("LedgerAccounting").await)
        .build();

    // Deposit initial funds
    let strategy_1 = test_strategy_1();
    let personal_venue = test_personal_venue();
    let binance_venue = test_binance_venue();
    let inst_btc = test_inst_binance_btc_usdt_perp();
    let inst_eth = test_inst_binance_eth_usdt_perp();
    let usdt = test_usdt_asset();
    let initial_amount = dec!(100_000);

    accounting
        .deposit(
            &personal_venue,
            &binance_venue,
            &usdt.clone().into(),
            initial_amount,
            &AccountType::Margin,
        )
        .await
        .expect("Failed to deposit initial funds");

    // Check if we have the right amount of funds
    let balances = accounting.margin_balance(&binance_venue).await;
    let usdt_balance = balances.get(&usdt).expect("Failed to get balance").clone();
    assert_eq!(usdt_balance, initial_amount);

    let asset_balance = accounting.asset_margin_balance(&binance_venue, &usdt).await;
    assert_eq!(asset_balance, initial_amount);
    info!("Initial funds deposited successfully");

    // Margin Trade
    let trade_amount = dec!(1);
    let trade_price = dec!(86450);
    let margin_rate = dec!(0.05);
    let commission = dec!(0.0005);
    accounting
        .margin_trade(
            MarketSide::Buy,
            strategy_1.clone(),
            inst_btc.clone(),
            None,
            trade_amount,
            trade_price,
            margin_rate,
            commission,
        )
        .await
        .expect("Failed to execute margin trade");

    // Check balances
    let first_trade_margin = trade_amount * trade_price * margin_rate;
    let first_trade_commission = trade_amount * trade_price * commission;

    let balances = accounting.margin_balance(&binance_venue).await;
    let usdt_balance = balances.get(&usdt).expect("Failed to get balance").clone();
    assert_eq!(
        usdt_balance,
        initial_amount - first_trade_commission,
        "USDT balance after margin trade is incorrect"
    );

    let available_balance = accounting.available_margin_balance(&binance_venue).await;
    let usdt_available_balance = available_balance.get(&usdt).expect("Failed to get balance").clone();
    assert_eq!(
        usdt_available_balance,
        initial_amount - first_trade_margin - first_trade_commission,
        "Available USDT balance after margin trade is incorrect"
    );

    let asset_balance = accounting.asset_margin_balance(&binance_venue, &usdt).await;
    let available_asset_balance = accounting.asset_available_margin_balance(&binance_venue, &usdt).await;
    assert_eq!(
        asset_balance,
        initial_amount - first_trade_commission,
        "Asset balance after margin trade is incorrect"
    );
    assert_eq!(
        available_asset_balance,
        initial_amount - first_trade_margin - first_trade_commission,
        "Available asset balance after margin trade is incorrect"
    );

    // Second strategy trade
    let trade_amount = dec!(1);
    let trade_price = dec!(3410);
    let margin_rate = dec!(0.05);
    let commission = dec!(0.0005);
    accounting
        .margin_trade(
            MarketSide::Buy,
            strategy_1.clone(),
            inst_eth.clone(),
            None,
            trade_amount,
            trade_price,
            margin_rate,
            commission,
        )
        .await
        .expect("Failed to execute margin trade");

    // Check balances
    let second_trade_margin = trade_amount * trade_price * margin_rate;
    let second_trade_commission = trade_amount * trade_price * commission;

    let balances = accounting.margin_balance(&binance_venue).await;
    let usdt_balance = balances.get(&usdt).expect("Failed to get balance").clone();
    assert_eq!(
        usdt_balance,
        initial_amount - first_trade_commission - second_trade_commission,
        "USDT balance after margin trade is incorrect"
    );

    let available_balance = accounting.available_margin_balance(&binance_venue).await;
    let usdt_available_balance = available_balance.get(&usdt).expect("Failed to get balance").clone();
    assert_eq!(
        usdt_available_balance,
        initial_amount - first_trade_margin - first_trade_commission - second_trade_margin - second_trade_commission,
        "Available USDT balance after margin trade is incorrect"
    );

    let asset_balance = accounting.asset_margin_balance(&binance_venue, &usdt).await;
    assert_eq!(
        asset_balance,
        initial_amount - first_trade_commission - second_trade_commission,
        "Asset balance after margin trade is incorrect"
    );

    let available_asset_balance = accounting.asset_available_margin_balance(&binance_venue, &usdt).await;
    assert_eq!(
        available_asset_balance,
        initial_amount - first_trade_margin - first_trade_commission - second_trade_margin - second_trade_commission,
        "Available asset balance after margin trade is incorrect"
    );
}
