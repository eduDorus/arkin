use std::{fmt, sync::Arc};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use uuid::Uuid;

use crate::{
    types::{Price, Quantity},
    EventPayload, InstrumentQuery, PersistenceReader,
};

use super::Instrument;

#[derive(Debug, Clone)]
pub struct Book {
    pub received_time: UtcDateTime,
    pub event_time: UtcDateTime,
    pub instrument: Arc<Instrument>,
    pub bids: Vec<BookUpdateSide>,
    pub asks: Vec<BookUpdateSide>,
}

impl Book {
    pub fn new(
        event_time: UtcDateTime,
        instrument: Arc<Instrument>,
        bids: Vec<BookUpdateSide>,
        asks: Vec<BookUpdateSide>,
    ) -> Self {
        Self {
            received_time: UtcDateTime::now(),
            event_time,
            instrument,
            bids,
            asks,
        }
    }
}

#[async_trait]
impl EventPayload for Book {
    type Dto = BookDto;

    fn to_dto(&self) -> Self::Dto {
        self.clone().into()
    }

    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self> {
        let instrument = persistence
            .get_instrument(&InstrumentQuery::builder().id(dto.instrument_id).build())
            .await
            .context(format!("Failed to get instrument with id {}", dto.instrument_id))?;

        Ok(Book::new(dto.event_time, instrument, dto.bids, dto.asks))
    }
}

impl fmt::Display for Book {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} bid: {} ask: {}",
            self.instrument,
            self.event_time,
            self.bids.len(),
            self.asks.len()
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookUpdateSide {
    pub price: Price,
    pub quantity: Quantity,
}

impl BookUpdateSide {
    pub fn new(price: Price, quantity: Quantity) -> Self {
        Self { price, quantity }
    }
}

impl fmt::Display for BookUpdateSide {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.price, self.quantity)
    }
}

#[derive(Serialize, Deserialize)]
pub struct BookDto {
    pub received_time: UtcDateTime,
    pub event_time: UtcDateTime,
    pub instrument_id: Uuid,
    pub bids: Vec<BookUpdateSide>,
    pub asks: Vec<BookUpdateSide>,
}

impl From<Book> for BookDto {
    fn from(book: Book) -> Self {
        Self {
            received_time: book.received_time,
            event_time: book.event_time,
            instrument_id: book.instrument.id,
            bids: book.bids,
            asks: book.asks,
        }
    }
}
