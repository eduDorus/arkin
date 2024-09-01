use arkin_common::prelude::*;
use arkin_portfolio::PortfolioManager;
use rstest::*;
use rust_decimal_macros::dec;
use test_utils::prelude::*;
use time::OffsetDateTime;

#[rstest]
#[case::simple_long_position(
    vec![
        (strategy_crossover, perpetual_btc, Side::Buy, dec!(10.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover, perpetual_btc, Side::Sell, dec!(15.0), dec!(5.0), dec!(1.0)),
    ],
    vec![
        (strategy_crossover, perpetual_btc, PositionSide::Long, dec!(100.0), dec!(5.0), dec!(2.0), dec!(50.0)),
    ]
)]
fn test_portfolio(
    mut portfolio_manager: PortfolioManager,
    #[case] fill_params: Vec<(StrategyId, Instrument, Side, Price, Quantity, Commission)>,
    #[case] expected_positions: Vec<(StrategyId, Instrument, Price, Quantity, Commission, Notional)>,
) {
    for (strategy_id, instrument, side, price, quantity, commission) in fill_params.into_iter() {
        portfolio_manager.update_position(
            OffsetDateTime::now_utc(),
            strategy_id,
            instrument,
            side,
            price,
            quantity,
            commission,
        )
    }

    for (strategy_id, instrument, side, avg_open_price, quantity, realized_pnl) in expected_positions.into_iter() {
        let position = portfolio_manager.position(&strategy_id, &instrument).unwrap();
        assert_eq!(position.avg_open_price, avg_open_price);
        assert_eq!(position.quantity, quantity);
        assert_eq!(position.realized_pnl, realized_pnl);
    }
}
