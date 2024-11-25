use arkin_core::{test_utils::test_inst_binance_btc_usdt_perp, FillBuilder, MarketSide, PositionBuilder, PositionSide};
use arkin_portfolio::{Portfolio, SingleStrategyPortfolioBuilder};
use rust_decimal::prelude::*;
use test_log::test;
use uuid::Uuid;

#[test(tokio::test)]
async fn test_single_strategy_position_update() {
    // Create Portfolio
    let portfolio = SingleStrategyPortfolioBuilder::default()
        .build()
        .expect("Failed to build SimplePortfolio");

    // Create Position
    let instrument = test_inst_binance_btc_usdt_perp();
    let position = PositionBuilder::default()
        .instrument(instrument.clone())
        .side(PositionSide::Long)
        .avg_open_price(Decimal::from_f64(100.0).unwrap())
        .quantity(Decimal::from_f64(1.0).unwrap())
        .commission(Decimal::from_f64(0.0).unwrap())
        .build()
        .unwrap();

    // Update Position
    portfolio
        .position_update(position.clone())
        .await
        .expect("Failed to update position");

    let positions = portfolio.positions().await;
    assert_eq!(positions.len(), 1);
    assert_eq!(positions.get(&instrument).unwrap(), &position);
}

#[test(tokio::test)]
async fn test_single_strategy_portfolio_fill_update() {
    // Create Portfolio
    let portfolio = SingleStrategyPortfolioBuilder::default()
        .build()
        .expect("Failed to build SimplePortfolio");

    // Create Fill
    let instrument = test_inst_binance_btc_usdt_perp();
    let fill = FillBuilder::default()
        .venue_order_id(Uuid::new_v4())
        .instrument(instrument.clone())
        .side(MarketSide::Buy)
        .price(Decimal::from_f64(100.0).unwrap())
        .quantity(Decimal::from_f64(1.0).unwrap())
        .commission(Decimal::from_f64(2.0).unwrap())
        .build()
        .unwrap();

    // Update Fill
    portfolio.fill_update(fill.clone()).await.expect("Failed to update fill");

    let positions = portfolio.positions().await;
    let new_position = positions.get(&instrument).unwrap();
    assert_eq!(positions.len(), 1);
    assert_eq!(new_position.instrument, instrument);
    assert_eq!(new_position.avg_open_price, fill.price);
    assert_eq!(new_position.quantity, fill.quantity);
    assert_eq!(new_position.commission, fill.commission);

    // Update Fill
    let fill = FillBuilder::default()
        .venue_order_id(Uuid::new_v4())
        .instrument(instrument.clone())
        .side(MarketSide::Sell)
        .price(Decimal::from_f64(101.0).unwrap())
        .quantity(Decimal::from_f64(1.0).unwrap())
        .commission(Decimal::from_f64(2.0).unwrap())
        .build()
        .unwrap();

    portfolio.fill_update(fill.clone()).await.expect("Failed to update fill");

    let positions = portfolio.positions().await;
    let new_position = positions.get(&instrument).unwrap();
    assert_eq!(positions.len(), 1);
    assert_eq!(new_position.instrument, instrument);
    assert_eq!(new_position.avg_open_price, Decimal::from_f64(100.0).unwrap());
    assert_eq!(new_position.avg_close_price, Some(Decimal::from_f64(101.0).unwrap()));
    assert_eq!(new_position.quantity, Decimal::from_f64(0.0).unwrap());
    assert_eq!(new_position.commission, Decimal::from_f64(4.0).unwrap());
    assert_eq!(new_position.realized_pnl, Decimal::from_f64(1.0).unwrap());
}
