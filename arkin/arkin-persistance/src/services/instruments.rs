use std::{collections::HashMap, sync::Arc};

use anyhow::{Error, Result};
use arkin_core::prelude::*;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::repos::InstrumentRepo;

use super::venues::VenueService;

pub struct InstrumentCache {
    by_id: RwLock<HashMap<Uuid, Instrument>>,
    by_venue_symbol: RwLock<HashMap<String, Instrument>>,
}

impl InstrumentCache {
    pub fn new() -> Self {
        Self {
            by_id: RwLock::new(HashMap::new()),
            by_venue_symbol: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, instrument: Instrument) {
        self.by_id.write().insert(instrument.id, instrument.clone());
        self.by_venue_symbol.write().insert(instrument.venue_symbol.clone(), instrument);
    }

    pub fn get_by_id(&self, id: &Uuid) -> Option<Instrument> {
        self.by_id.read().get(id).cloned()
    }

    pub fn get_by_venue_symbol(&self, venue_symbol: &str) -> Option<Instrument> {
        self.by_venue_symbol.read().get(venue_symbol).cloned()
    }
}

pub struct InstrumentService {
    instrument_repo: Arc<InstrumentRepo>,
    instrument_cache: InstrumentCache,
    venue_service: Arc<VenueService>,
}

impl InstrumentService {
    pub fn new(instrument_repo: Arc<InstrumentRepo>, venue_service: Arc<VenueService>) -> Self {
        Self {
            instrument_repo,
            instrument_cache: InstrumentCache::new(),
            venue_service,
        }
    }

    pub async fn insert(&self, instrument: Instrument) -> Result<()> {
        self.instrument_repo.create(instrument).await
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Option<Instrument>> {
        // Check cache
        if let Some(instrument) = self.instrument_cache.get_by_id(id) {
            return Ok(Some(instrument));
        }

        // Read from db
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

            // Update cache
            self.instrument_cache.insert(instrument.clone());

            Ok(Some(instrument))
        } else {
            Ok(None)
        }
    }

    pub async fn read_by_venue_symbol(&self, venue_symbol: &str) -> Result<Option<Instrument>> {
        // Read from cache
        if let Some(instrument) = self.instrument_cache.get_by_venue_symbol(venue_symbol) {
            return Ok(Some(instrument));
        }

        // Read from db
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

            // Update cache
            self.instrument_cache.insert(instrument.clone());

            Ok(Some(instrument))
        } else {
            Ok(None)
        }
    }
}
