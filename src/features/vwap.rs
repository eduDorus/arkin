use super::Feature;
use crate::{
    config::VWAPConfig,
    models::{Asset, Instrument, PerpetualContract, Price, Venue},
    state::StateManager,
};
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
            let instrument = PerpetualContract::new(&Venue::Binance, &Asset::new("BTC"), &Asset::new("USDT"));
            let trades = self.state.data.list_market(
                &Instrument::Perpetual(instrument),
                &OffsetDateTime::now_utc(),
                &Duration::seconds(5),
            );
            info!("Window:");
            for trade in trades {
                info!("- {}", trade);
            }
        }
    }
}
