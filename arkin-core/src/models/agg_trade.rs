use std::{cmp::Ordering, fmt, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{EventPayload, InstrumentQuery, PersistenceReader, Price, Quantity};

use super::{Instrument, MarketSide};

#[derive(Debug, Clone, TypedBuilder)]
pub struct AggTrade {
    pub event_time: UtcDateTime,
    pub instrument: Arc<Instrument>,
    pub trade_id: u64,
    pub side: MarketSide,
    pub price: Price,
    pub quantity: Quantity,
}

impl AggTrade {
    pub fn new(
        event_time: UtcDateTime,
        instrument: Arc<Instrument>,
        trade_id: u64,
        side: MarketSide,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Self {
            event_time,
            instrument,
            trade_id,
            side,
            price,
            quantity,
        }
    }
}

#[async_trait]
impl EventPayload for AggTrade {
    type Dto = AggTradeDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let instrument = persistence
            .get_instrument(&InstrumentQuery::builder().id(dto.instrument_id).build())
            .await
            .context(format!("Failed to get instrument with id {}", dto.instrument_id))?;

        Ok(AggTrade::builder()
            .event_time(dto.event_time)
            .instrument(instrument)
            .trade_id(dto.trade_id)
            .side(dto.side)
            .price(dto.price)
            .quantity(dto.quantity)
            .build())
    }
}

impl PartialEq for AggTrade {
    fn eq(&self, other: &Self) -> bool {
        self.event_time == other.event_time
            && self.trade_id == other.trade_id
            && Arc::ptr_eq(&self.instrument, &other.instrument)
    }
}

impl Eq for AggTrade {}

impl PartialOrd for AggTrade {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AggTrade {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.event_time, self.trade_id).cmp(&(other.event_time, other.trade_id))
    }
}

impl fmt::Display for AggTrade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} @ {}", self.side, self.quantity, self.instrument, self.price)
    }
}

#[derive(Serialize, Deserialize)]
pub struct AggTradeDto {
    pub event_time: UtcDateTime,
    pub instrument_id: Uuid,
    pub trade_id: u64,
    pub side: MarketSide,
    pub price: Decimal,
    pub quantity: Decimal,
}

impl From<AggTrade> for AggTradeDto {
    fn from(trade: AggTrade) -> Self {
        Self {
            event_time: trade.event_time,
            instrument_id: trade.instrument.id,
            trade_id: trade.trade_id,
            side: trade.side,
            price: trade.price,
            quantity: trade.quantity,
        }
    }
}
