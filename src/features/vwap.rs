use super::Feature;
use crate::{
    config::VWAPConfig,
    models::{Asset, EventID, Instrument, PerpetualContract, Price, Venue},
    state::StateManager,
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

        let mut rx = self.state.listen_feature_frequency(Duration::seconds(5));

        while (rx.recv().await).is_ok() {
            info!("VWAP feature tick...");
            let instrument =
                Instrument::Perpetual(PerpetualContract::new(&Venue::Binance, &Asset::new("BTC"), &Asset::new("USDT")));
            let res = self
                .state
                .data
                .list_events(OffsetDateTime::now_utc(), Duration::seconds(5), |event| {
                    if matches!(event.event_type(), EventID::TradeUpdate) && event.instrument() == &instrument {
                        return Some(event);
                    }
                    None
                });
            info!("Window:");
            for event in res {
                info!("- {}: {}: {}", event.instrument(), event.event_type(), event);
            }
        }
    }
}
