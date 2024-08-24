use arkin_common::prelude::*;
use arkin_strategies::prelude::*;
use rstest::*;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use test_utils::prelude::*;
use time::OffsetDateTime;

#[fixture]
pub fn crossover_strategy() -> CrossoverStrategy {
    let config = CrossoverConfig {
        id: "crossover".into(),
        price_spread_id: "spread_sma_vwap".into(),
        volume_spread_id: "spread_sma_volume".into(),
    };

    CrossoverStrategy::from_config(&config)
}

#[rstest]
#[case(dec!(0), dec!(0), dec!(0))]
#[case(dec!(1), dec!(1), dec!(-1))]
#[case(dec!(-1), dec!(1), dec!(1))]
#[case(dec!(1), dec!(-1), dec!(0))]
#[case(dec!(-1), dec!(-1), dec!(0))]
#[case(dec!(1), dec!(0), dec!(0))]
#[case(dec!(-1), dec!(0), dec!(0))]
#[case(dec!(0), dec!(-1), dec!(0))]
#[case(dec!(0), dec!(-1), dec!(0))]

pub fn crossover(
    crossover_strategy: CrossoverStrategy,
    instrument: Instrument,
    event_time: OffsetDateTime,
    #[case] spread_price: Decimal,
    #[case] spread_volume: Decimal,
    #[case] expected: Decimal,
) {
    let features = vec![
        Insight::new("spread_sma_vwap".into(), instrument.clone(), event_time, spread_price),
        Insight::new("spread_sma_volume".into(), instrument, event_time, spread_volume),
    ];
    let res = crossover_strategy.calculate(&features);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].signal, Weight::from(expected));
}
