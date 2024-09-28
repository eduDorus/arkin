use std::sync::Arc;

use anyhow::{Error, Result};
use arkin_core::prelude::*;
use tracing::debug;
use uuid::Uuid;

use crate::repos::InstrumentRepo;

use super::venues::VenueService;

pub struct InstrumentService {
    instrument_repo: Arc<InstrumentRepo>,
    venue_service: Arc<VenueService>,
}

impl InstrumentService {
    pub fn new(instrument_repo: Arc<InstrumentRepo>, venue_service: Arc<VenueService>) -> Self {
        Self {
            instrument_repo,
            venue_service,
        }
    }

    pub async fn insert(&self, instrument: Instrument) -> Result<()> {
        self.instrument_repo.create(instrument).await
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Option<Instrument>> {
        if let Some(db_instrument) = self.instrument_repo.read_by_id(id).await? {
            let venue = self
                .venue_service
                .read_by_id(&db_instrument.venue_id)
                .await?
                .ok_or_else(|| Error::msg("Venue not found"))?;

            let instrument = Instrument {
                id: db_instrument.id,
                symbol: db_instrument.symbol,
                venue_symbol: db_instrument.venue_symbol,
                venue,
                instrument_type: db_instrument.instrument_type.into(),
                base_asset: db_instrument.base_asset,
                quote_asset: db_instrument.quote_asset,
                maturity: db_instrument.maturity,
                strike: db_instrument.strike,
                option_type: db_instrument.option_type.map(|v| v.into()),
                contract_size: db_instrument.contract_size,
                price_precision: db_instrument.price_precision as u32,
                quantity_precision: db_instrument.quantity_precision as u32,
                base_precision: db_instrument.base_precision as u32,
                quote_precision: db_instrument.quote_precision as u32,
                tick_size: db_instrument.tick_size,
                lot_size: db_instrument.lot_size,
                status: db_instrument.status.into(),
            };
            Ok(Some(instrument))
        } else {
            Ok(None)
        }
    }

    pub async fn read_by_venue_symbol(&self, venue_symbol: &str) -> Result<Option<Instrument>> {
        debug!("Reading instrument by venue symbol: {}", venue_symbol);
        if let Some(db_instrument) = self.instrument_repo.read_by_venue_symbol(venue_symbol).await? {
            let venue = self
                .venue_service
                .read_by_id(&db_instrument.venue_id)
                .await?
                .ok_or_else(|| Error::msg("Venue not found"))?;

            let instrument = Instrument {
                id: db_instrument.id,
                symbol: db_instrument.symbol,
                venue_symbol: db_instrument.venue_symbol,
                venue,
                instrument_type: db_instrument.instrument_type.into(),
                base_asset: db_instrument.base_asset,
                quote_asset: db_instrument.quote_asset,
                maturity: db_instrument.maturity,
                strike: db_instrument.strike,
                option_type: db_instrument.option_type.map(|v| v.into()),
                contract_size: db_instrument.contract_size,
                price_precision: db_instrument.price_precision as u32,
                quantity_precision: db_instrument.quantity_precision as u32,
                base_precision: db_instrument.base_precision as u32,
                quote_precision: db_instrument.quote_precision as u32,
                tick_size: db_instrument.tick_size,
                lot_size: db_instrument.lot_size,
                status: db_instrument.status.into(),
            };
            Ok(Some(instrument))
        } else {
            Ok(None)
        }
    }
}
