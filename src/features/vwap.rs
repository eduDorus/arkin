use crate::{
    config::VWAPConfig,
    constants::TIMESTAMP_FORMAT,
    models::{Event, EventType, Instrument, Notional, Price, Quantity},
    state::StateManager,
};
use anyhow::Result;
use rust_decimal::Decimal;
use std::sync::Arc;
use std::{fmt, time::Duration};
use time::OffsetDateTime;
use tracing::info;

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct VWAP {
    pub id: String,
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub price: Price,
}

impl VWAP {
    pub fn from_trades(
        id: &str,
        instrument: &Instrument,
        event_time: &OffsetDateTime,
        trades: &[Event],
    ) -> Result<Self> {
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
            id: id.to_owned(),
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
    id: String,
    frequency: Duration,
    window: Duration,
}

impl VWAPFeature {
    pub fn new(state: Arc<StateManager>, config: &VWAPConfig) -> VWAPFeature {
        let frequency = Duration::from_secs(config.frequency);
        let window = Duration::from_secs(config.window);
        VWAPFeature {
            state,
            id: config.id.to_owned(),
            frequency,
            window,
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&self) {
        info!("Starting VWAP feature...");
        let mut rx = self.state.subscribe_frequency(self.frequency);

        while let Ok(tick) = rx.recv().await {
            for instrument in self.state.data.list_instruments().await {
                let res = self
                    .state
                    .data
                    .list_events(&instrument, EventType::TradeUpdate, tick, self.window)
                    .await;

                if let Ok(vwap) = VWAP::from_trades(self.id(), &instrument, &tick, &res) {
                    info!(
                        "Calculated VWAP with frequency {:?} and window {:?} for {} at {} is {}",
                        self.frequency,
                        self.window,
                        instrument,
                        tick.format(TIMESTAMP_FORMAT).expect("Unable to format timestamp"),
                        vwap.price
                    );
                    self.state.event_update(Event::VWAP(vwap)).await;
                }
            }
        }
    }
}
