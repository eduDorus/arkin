use super::Feature;
use crate::{
    config::VWAPConfig,
    constants::TIMESTAMP_FORMAT,
    models::{Event, EventID, Instrument, Notional, Price, Quantity},
    state::StateManager,
    utils::create_interval,
};
use anyhow::Result;
use rust_decimal::Decimal;
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
    pub fn from_trades(instrument: &Instrument, event_time: &OffsetDateTime, trades: &[Event]) -> Result<Self> {
        if trades.is_empty() {
            return Err(anyhow::anyhow!("No trades to calculate VWAP"));
        }

        let mut total_quantity = Quantity::new(Decimal::ZERO);
        let mut total_notional = Notional::new(Decimal::ZERO);

        for trade in trades {
            if let Event::TradeUpdate(trade) = trade {
                total_quantity += trade.quantity;
                total_notional += trade.price * trade.quantity.abs();
            }
        }

        Ok(VWAP {
            instrument: instrument.to_owned(),
            event_time: event_time.to_owned(),
            price: (total_notional / total_quantity),
        })
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
    frequency: Duration,
    window: Duration,
}

impl VWAPFeature {
    pub fn new(state: Arc<StateManager>, config: &VWAPConfig) -> VWAPFeature {
        let frequency = Duration::seconds(config.frequency as i64);
        let window = Duration::seconds(config.window as i64);
        VWAPFeature {
            state,
            frequency,
            window,
        }
    }
}

impl Feature for VWAPFeature {
    async fn start(&self) {
        info!("Starting VWAP feature...");
        let mut interval = create_interval(self.frequency);

        loop {
            interval.tick().await;
            let now = OffsetDateTime::now_utc();

            for instrument in self.state.data.list_instruments() {
                let res = self.state.data.list_events(now, self.window, |event| {
                    if matches!(event.event_type(), EventID::TradeUpdate) && event.instrument() == &instrument {
                        return Some(event);
                    }
                    None
                });

                if let Ok(vwap) = VWAP::from_trades(&instrument, &now, &res) {
                    info!(
                        "Calculated VWAP with frequency {} and window {} for {} at {} is {}",
                        self.frequency,
                        self.window,
                        instrument,
                        now.format(TIMESTAMP_FORMAT).expect("Unable to format timestamp"),
                        vwap.price
                    );
                }
            }
        }
    }
}
