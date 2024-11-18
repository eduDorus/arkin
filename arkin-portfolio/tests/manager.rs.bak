use arkin_core::prelude::*;
use arkin_portfolio::PortfolioManager;
use rstest::*;
use rust_decimal_macros::dec;
use test_utils::prelude::*;
use time::OffsetDateTime;

#[rstest]
#[case::win_long_position(
    vec![
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(10.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Sell, dec!(15.0), dec!(5.0), dec!(1.0)),
    ],
    vec![
        (strategy_crossover(), perpetual_btc(), PositionSide::Long, dec!(10.0), dec!(5.0), dec!(2.0), dec!(25.0)),
    ]
)]
#[case::win_short_position_profit(
    vec![
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Sell, dec!(15.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(10.0), dec!(5.0), dec!(1.0)),
    ],
    vec![
        (strategy_crossover(), perpetual_btc(), PositionSide::Short, dec!(15.0), dec!(5.0), dec!(2.0), dec!(25.0)),
    ]
)]
#[case::loss_long_position(
    vec![
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(20.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Sell, dec!(15.0), dec!(5.0), dec!(1.0)),
    ],
    vec![
        (strategy_crossover(), perpetual_btc(), PositionSide::Long, dec!(20.0), dec!(5.0), dec!(2.0), dec!(-25.0)),
    ]
)]
#[case::loss_short_position_profit(
    vec![
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Sell, dec!(15.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(20.0), dec!(5.0), dec!(1.0)),
    ],
    vec![
        (strategy_crossover(), perpetual_btc(), PositionSide::Short, dec!(15.0), dec!(5.0), dec!(2.0), dec!(-25.0)),
    ]
)]
#[case::avg_into_long_position(
    vec![
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(10.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(15.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(20.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Sell, dec!(20.0), dec!(30.0), dec!(1.0)),
    ],
    vec![
        (strategy_crossover(), perpetual_btc(), PositionSide::Long, dec!(15.0), dec!(0.0), dec!(4.0), dec!(150.0)),
    ]
)]
#[case::avg_into_short_position(
    vec![
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Sell, dec!(10.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Sell, dec!(15.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Sell, dec!(20.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(10.0), dec!(30.0), dec!(1.0)),
    ],
    vec![
        (strategy_crossover(), perpetual_btc(), PositionSide::Short, dec!(15.0), dec!(0.0), dec!(4.0), dec!(150.0)),
    ]
)]
#[case::flip_side(
    vec![
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(10.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Sell, dec!(15.0), dec!(20.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(10.0), dec!(10.0), dec!(1.0)),
    ],
    vec![
        (strategy_crossover(), perpetual_btc(), PositionSide::Short, dec!(15.0), dec!(0.0), dec!(1.5), dec!(50.0)),
    ]
)]
#[case::multi_instrument(
    vec![
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(10.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_eth(), ExecutionOrderSide::Sell, dec!(10.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(20.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_eth(), ExecutionOrderSide::Sell, dec!(20.0), dec!(10.0), dec!(1.0)),
    ],
    vec![
        (strategy_crossover(), perpetual_btc(), PositionSide::Long, dec!(15.), dec!(20.), dec!(2.), dec!(0.)),
        (strategy_crossover(), perpetual_eth(), PositionSide::Short, dec!(15.), dec!(20.), dec!(2.), dec!(0.)),
    ]
)]
#[case::multi_instrument_multi_strategy(
    vec![
        (strategy_crossover(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(10.0), dec!(10.0), dec!(1.0)),
        (strategy_crossover(), perpetual_eth(), ExecutionOrderSide::Sell, dec!(10.0), dec!(10.0), dec!(1.0)),
        (strategy_predator(), perpetual_btc(), ExecutionOrderSide::Buy, dec!(10.0), dec!(10.0), dec!(1.0)),
        (strategy_predator(), perpetual_eth(), ExecutionOrderSide::Sell, dec!(10.0), dec!(10.0), dec!(1.0)),
    ],
    vec![
        (strategy_crossover(), perpetual_btc(), PositionSide::Long, dec!(10.), dec!(10.), dec!(1.), dec!(0.)),
        (strategy_crossover(), perpetual_eth(), PositionSide::Short, dec!(10.), dec!(10.), dec!(1.), dec!(0.)),
        (strategy_predator(), perpetual_btc(), PositionSide::Long, dec!(10.), dec!(10.), dec!(1.), dec!(0.)),
        (strategy_predator(), perpetual_eth(), PositionSide::Short, dec!(10.), dec!(10.), dec!(1.), dec!(0.)),
    ]
)]
fn test_portfolio(
    portfolio_manager: PortfolioManager,
    #[case] fill_params: Vec<(StrategyId, Instrument, ExecutionOrderSide, Price, Quantity, Commission)>,
    #[case] expected_positions: Vec<(StrategyId, Instrument, PositionSide, Price, Quantity, Commission, Notional)>,
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

    for (strategy_id, instrument, side, avg_open_price, quantity, commission, realized_pnl) in
        expected_positions.into_iter()
    {
        let position = portfolio_manager.open_position(&strategy_id, &instrument).unwrap();
        assert_eq!(position.avg_open_price, avg_open_price);
        assert_eq!(position.side, side);
        assert_eq!(position.quantity, quantity);
        assert_eq!(position.commission, commission);
        assert_eq!(position.realized_pnl, realized_pnl);
    }
}
