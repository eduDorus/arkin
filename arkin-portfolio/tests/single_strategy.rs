use std::sync::Arc;

use rust_decimal_macros::dec;
use test_log::test;

use arkin_core::prelude::*;
use arkin_portfolio::prelude::*;

#[test(tokio::test)]
async fn test_single_strategy_long_position() {
    let pubsub = Arc::new(PubSub::new());

    // Create Portfolio
    let portfolio = Arc::new(
        SingleStrategyPortfolioBuilder::default()
            .pubsub(pubsub.clone())
            .build()
            .expect("Failed to build SimplePortfolio"),
    );

    // Create instrument
    let instrument = test_inst_binance_btc_usdt_perp();

    // Create balance
    let balance = HoldingBuilder::default()
        .asset(instrument.quote_asset.clone())
        .balance(dec!(10000))
        .build()
        .unwrap();
    portfolio
        .balance_update(balance.clone())
        .await
        .expect("Failed to update balance");

    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(10000));

    // Create fill
    let fill = ExecutionOrderFillBuilder::default()
        .id(ExecutionOrderId::new_v4())
        .instrument(instrument.clone())
        .side(MarketSide::Buy)
        .price(dec!(100.0))
        .quantity(dec!(1.0))
        .commission(dec!(2.0))
        .build()
        .unwrap();

    portfolio.fill_update(fill.clone()).await.expect("Failed to update fill");
    let positions = portfolio.list_open_positions().await;
    assert_eq!(positions.len(), 1);

    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(9898));

    let asset_capital = portfolio.capital(&instrument.quote_asset).await;
    assert_eq!(asset_capital, dec!(9998));

    // Create fill
    let fill = ExecutionOrderFillBuilder::default()
        .id(ExecutionOrderId::new_v4())
        .instrument(instrument.clone())
        .side(MarketSide::Sell)
        .price(dec!(120.0))
        .quantity(dec!(0.5))
        .commission(dec!(2.0))
        .build()
        .unwrap();

    portfolio.fill_update(fill.clone()).await.expect("Failed to update fill");
    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(9956));

    let asset_capital = portfolio.capital(&instrument.quote_asset).await;
    assert_eq!(asset_capital, dec!(10006));

    // Create fill
    let fill = ExecutionOrderFillBuilder::default()
        .id(ExecutionOrderId::new_v4())
        .instrument(instrument.clone())
        .side(MarketSide::Sell)
        .price(dec!(110.0))
        .quantity(dec!(0.5))
        .commission(dec!(2.0))
        .build()
        .unwrap();

    portfolio.fill_update(fill.clone()).await.expect("Failed to update fill");
    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(10009));

    let asset_capital = portfolio.capital(&instrument.quote_asset).await;
    assert_eq!(asset_capital, dec!(10009));

    let open_positions = portfolio.list_open_positions().await;
    assert_eq!(open_positions.len(), 0);

    let realized_pnl = portfolio.pnl_instrument(&instrument).await;
    assert_eq!(realized_pnl, dec!(15));

    let commission = portfolio.commission_instrument(&instrument).await;
    assert_eq!(commission, dec!(6));
}

#[test(tokio::test)]
async fn test_single_strategy_short_position() {
    let pubsub = Arc::new(PubSub::new());

    // Create Portfolio
    let portfolio = Arc::new(
        SingleStrategyPortfolioBuilder::default()
            .pubsub(pubsub.clone())
            .build()
            .expect("Failed to build SimplePortfolio"),
    );

    // Create instrument
    let instrument = test_inst_binance_btc_usdt_perp();

    // Create balance
    let balance = HoldingBuilder::default()
        .asset(instrument.quote_asset.clone())
        .balance(dec!(10000))
        .build()
        .unwrap();
    portfolio
        .balance_update(balance.clone())
        .await
        .expect("Failed to update balance");

    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(10000));

    // Create fill
    let fill = ExecutionOrderFillBuilder::default()
        .id(ExecutionOrderId::new_v4())
        .instrument(instrument.clone())
        .side(MarketSide::Sell)
        .price(dec!(100.0))
        .quantity(dec!(1.0))
        .commission(dec!(2.0))
        .build()
        .unwrap();

    portfolio.fill_update(fill.clone()).await.expect("Failed to update fill");
    let positions = portfolio.list_open_positions().await;
    assert_eq!(positions.len(), 1);

    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(10098));

    let asset_capital = portfolio.capital(&instrument.quote_asset).await;
    assert_eq!(asset_capital, dec!(9998));

    // Create fill
    let fill = ExecutionOrderFillBuilder::default()
        .id(ExecutionOrderId::new_v4())
        .instrument(instrument.clone())
        .side(MarketSide::Buy)
        .price(dec!(80.0))
        .quantity(dec!(0.5))
        .commission(dec!(2.0))
        .build()
        .unwrap();

    portfolio.fill_update(fill.clone()).await.expect("Failed to update fill");
    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(10056));

    let asset_capital = portfolio.capital(&instrument.quote_asset).await;
    assert_eq!(asset_capital, dec!(10006));

    // Create fill
    let fill = ExecutionOrderFillBuilder::default()
        .id(ExecutionOrderId::new_v4())
        .instrument(instrument.clone())
        .side(MarketSide::Buy)
        .price(dec!(90.0))
        .quantity(dec!(0.5))
        .commission(dec!(2.0))
        .build()
        .unwrap();

    portfolio.fill_update(fill.clone()).await.expect("Failed to update fill");
    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(10009));

    let asset_capital = portfolio.capital(&instrument.quote_asset).await;
    assert_eq!(asset_capital, dec!(10009));

    let open_positions = portfolio.list_open_positions().await;
    assert_eq!(open_positions.len(), 0);

    let realized_pnl = portfolio.pnl_instrument(&instrument).await;
    assert_eq!(realized_pnl, dec!(15));

    let commission = portfolio.commission_instrument(&instrument).await;
    assert_eq!(commission, dec!(6));
}

#[test(tokio::test)]
async fn test_single_strategy_swap_position() {
    let pubsub = Arc::new(PubSub::new());

    // Create Portfolio
    let portfolio = Arc::new(
        SingleStrategyPortfolioBuilder::default()
            .pubsub(pubsub.clone())
            .build()
            .expect("Failed to build SimplePortfolio"),
    );

    // Create instrument
    let instrument = test_inst_binance_btc_usdt_perp();

    // Create balance
    let balance = HoldingBuilder::default()
        .asset(instrument.quote_asset.clone())
        .balance(dec!(10000))
        .build()
        .unwrap();
    portfolio
        .balance_update(balance.clone())
        .await
        .expect("Failed to update balance");

    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(10000));

    // Create fill
    let fill = ExecutionOrderFillBuilder::default()
        .id(ExecutionOrderId::new_v4())
        .instrument(instrument.clone())
        .side(MarketSide::Buy)
        .price(dec!(100.0))
        .quantity(dec!(1.0))
        .commission(dec!(2.0))
        .build()
        .unwrap();

    portfolio.fill_update(fill.clone()).await.expect("Failed to update fill");
    let positions = portfolio.list_open_positions().await;
    assert_eq!(positions.len(), 1);

    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(9898));

    let asset_capital = portfolio.capital(&instrument.quote_asset).await;
    assert_eq!(asset_capital, dec!(9998));

    // Create fill
    let fill = ExecutionOrderFillBuilder::default()
        .id(ExecutionOrderId::new_v4())
        .instrument(instrument.clone())
        .side(MarketSide::Sell)
        .price(dec!(120.0))
        .quantity(dec!(2.0))
        .commission(dec!(2.0))
        .build()
        .unwrap();

    portfolio.fill_update(fill.clone()).await.expect("Failed to update fill");

    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(10136));

    let asset_capital = portfolio.capital(&instrument.quote_asset).await;
    assert_eq!(asset_capital, dec!(10016));

    // Create fill
    let fill = ExecutionOrderFillBuilder::default()
        .id(ExecutionOrderId::new_v4())
        .instrument(instrument.clone())
        .side(MarketSide::Buy)
        .price(dec!(100.0))
        .quantity(dec!(1.0))
        .commission(dec!(2.0))
        .build()
        .unwrap();

    portfolio.fill_update(fill.clone()).await.expect("Failed to update fill");
    let portfolio_balance = portfolio.balance(&instrument.quote_asset).await.unwrap();
    assert_eq!(portfolio_balance.balance, dec!(10034));

    let asset_capital = portfolio.capital(&instrument.quote_asset).await;
    assert_eq!(asset_capital, dec!(10034));

    let open_positions = portfolio.list_open_positions().await;
    assert_eq!(open_positions.len(), 0);

    let realized_pnl = portfolio.pnl_instrument(&instrument).await;
    assert_eq!(realized_pnl, dec!(40));

    let commission = portfolio.commission_instrument(&instrument).await;
    assert_eq!(commission, dec!(6));
}
