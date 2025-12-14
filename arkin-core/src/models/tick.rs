use std::{cmp::Ordering, fmt, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{EventPayload, InstrumentQuery, PersistenceReader, Price, Quantity};

use super::Instrument;

#[derive(Debug, Clone, TypedBuilder)]
pub struct Tick {
    pub event_time: UtcDateTime,
    pub instrument: Arc<Instrument>,
    pub tick_id: u64,
    pub bid_price: Price,
    pub bid_quantity: Quantity,
    pub ask_price: Price,
    pub ask_quantity: Quantity,
}

impl Tick {
    pub fn new(
        event_time: UtcDateTime,
        instrument: Arc<Instrument>,
        tick_id: u64,
        bid_price: Price,
        bid_quantity: Quantity,
        ask_price: Price,
        ask_quantity: Quantity,
    ) -> Self {
        Self {
            event_time,
            instrument,
            tick_id,
            bid_price,
            bid_quantity,
            ask_price,
            ask_quantity,
        }
    }

    pub fn spread(&self) -> Decimal {
        self.round_to_tick(self.ask_price - self.bid_price)
    }

    pub fn mid_price(&self) -> Price {
        self.round_to_tick((self.bid_price + self.ask_price) / dec!(2))
    }

    pub fn bid_price(&self) -> Price {
        self.round_to_tick(self.bid_price)
    }

    pub fn ask_price(&self) -> Price {
        self.round_to_tick(self.ask_price)
    }

    fn round_to_tick(&self, value: Decimal) -> Decimal {
        if value.is_zero() {
            return Decimal::ZERO;
        }
        let tick_size = self.instrument.tick_size;
        let scaling_factor = Decimal::ONE / tick_size;
        let scaled = value * scaling_factor;
        let rounded_scaled = scaled.round();
        let rounded_value = rounded_scaled * tick_size;
        rounded_value.round_dp(self.instrument.price_precision)
    }
}

impl PartialEq for Tick {
    fn eq(&self, other: &Self) -> bool {
        self.event_time == other.event_time
            && self.tick_id == other.tick_id
            && Arc::ptr_eq(&self.instrument, &other.instrument)
    }
}

impl Eq for Tick {}

impl PartialOrd for Tick {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tick {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.event_time, self.tick_id).cmp(&(other.event_time, other.tick_id))
    }
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} / {}", self.instrument, self.bid_price, self.ask_price)
    }
}

#[async_trait]
impl EventPayload for Tick {
    type Dto = TickDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let instrument = persistence
            .get_instrument(&InstrumentQuery::builder().id(dto.instrument_id).build())
            .await
            .context(format!("Failed to get instrument with id {}", dto.instrument_id))?;

        Ok(Tick::builder()
            .event_time(dto.event_time)
            .instrument(instrument)
            .tick_id(dto.tick_id)
            .bid_price(dto.bid_price)
            .bid_quantity(dto.bid_quantity)
            .ask_price(dto.ask_price)
            .ask_quantity(dto.ask_quantity)
            .build())
    }
}

#[derive(Serialize, Deserialize)]
pub struct TickDto {
    pub event_time: UtcDateTime,
    pub instrument_id: Uuid,
    pub tick_id: u64,
    pub bid_price: Decimal,
    pub bid_quantity: Decimal,
    pub ask_price: Decimal,
    pub ask_quantity: Decimal,
}

impl From<Tick> for TickDto {
    fn from(tick: Tick) -> Self {
        Self {
            event_time: tick.event_time,
            instrument_id: tick.instrument.id,
            tick_id: tick.tick_id,
            bid_price: tick.bid_price,
            bid_quantity: tick.bid_quantity,
            ask_price: tick.ask_price,
            ask_quantity: tick.ask_quantity,
        }
    }
}
