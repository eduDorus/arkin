use crate::models::{Asset, Event, Instrument, PerpetualContract, Venue};
use anyhow::Result;
use tracing::error;

use super::swaps::BinanceSwapsEvent;

pub struct BinanceParser {}

impl BinanceParser {
    pub fn parse(data: &str) -> Result<Event> {
        let event = match serde_json::from_str::<BinanceSwapsEvent>(data) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to parse Binance event: {}", e);
                error!("Data: {}", data);
                return Err(e.into());
            }
        };
        match event {
            BinanceSwapsEvent::Trade(t) => Ok(t.into()),
            BinanceSwapsEvent::AggTrade(t) => Ok(t.into()),
            BinanceSwapsEvent::Tick(t) => Ok(t.into()),
            BinanceSwapsEvent::Book(t) => Ok(t.into()),
        }
    }

    pub fn parse_instrument(instrument: &str) -> Instrument {
        let (base, quote) = instrument.split_at(instrument.len() - 4);
        Instrument::Perpetual(PerpetualContract::new(&Venue::Binance, &Asset::new(base), &Asset::new(quote)))
    }
}
