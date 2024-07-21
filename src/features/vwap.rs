use super::Feature;
use crate::{
    config::VWAPConfig,
    models::{Asset, EventID, Instrument, PerpetualContract, Price, Venue},
    state::StateManager,
    utils::create_interval,
};
use std::fmt;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use tracing::info;

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct VWAP {
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub price: Price,
}

impl VWAP {
    pub fn new(instrument: Instrument, event_time: OffsetDateTime, price: Price) -> VWAP {
        VWAP {
            instrument,
            event_time,
            price,
        }
    }
}

impl fmt::Display for VWAP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.instrument, self.event_time, self.price)
    }
}

#[derive(Clone)]
#[allow(unused)]
pub struct VWAPFeature {
    state: Arc<StateManager>,
    window: Duration,
}

impl VWAPFeature {
    pub fn new(state: Arc<StateManager>, config: &VWAPConfig) -> VWAPFeature {
        let window = Duration::seconds(config.window as i64);
        VWAPFeature { state, window }
    }
}

impl Feature for VWAPFeature {
    async fn start(&self) {
        info!("Starting VWAP feature...");
        let mut interval = create_interval(Duration::seconds(5));

        loop {
            interval.tick().await;
            let now = OffsetDateTime::now_utc().replace_nanosecond(0).expect("Failed to round");
            info!("VWAP feature tick...");
            let instrument =
                Instrument::Perpetual(PerpetualContract::new(&Venue::Binance, &Asset::new("BTC"), &Asset::new("USDT")));

            let res = self.state.data.list_events(now, Duration::seconds(5), |event| {
                if matches!(event.event_type(), EventID::AggTradeUpdate) && event.instrument() == &instrument {
                    return Some(event);
                }
                None
            });

            info!("Window:");
            for event in res {
                info!("- {}: {}: {}", event.instrument(), event.event_type(), event);
            }

            // let vwap = VWAP::new(instrument, now, Price::new(Decimal::ZERO).expect("Failed to create price"));
        }
    }
}
