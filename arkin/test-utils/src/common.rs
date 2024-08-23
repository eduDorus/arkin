use arkin_common::prelude::*;
use rstest::*;
use time::OffsetDateTime;

#[fixture]
pub fn instrument() -> Instrument {
    Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into())
}

#[fixture]
pub fn event_time() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}
