use std::sync::Arc;

use anyhow::{Error, Result};
use dashmap::DashMap;
use tracing::{debug, error};
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::repos::InstrumentRepo;

use super::venues::VenueService;

#[derive(Debug)]
pub struct InstrumentCache {
    by_id: DashMap<Uuid, Arc<Instrument>>,
    by_venue_symbol: DashMap<String, Arc<Instrument>>,
}

impl InstrumentCache {
    pub fn new() -> Self {
        Self {
            by_id: DashMap::new(),
            by_venue_symbol: DashMap::new(),
        }
    }

    pub fn insert(&self, instrument: Instrument) -> Arc<Instrument> {
        let instrument = Arc::new(instrument);
        self.by_id.insert(instrument.id, instrument.clone());
        self.by_venue_symbol.insert(instrument.venue_symbol.clone(), instrument.clone());
        instrument
    }

    pub fn get_by_id(&self, id: Uuid) -> Option<Arc<Instrument>> {
        self.by_id.get(&id).map(|entry| entry.value().clone())
    }

    pub fn get_by_venue_symbol(&self, venue_symbol: &str) -> Option<Arc<Instrument>> {
        self.by_venue_symbol.get(venue_symbol).map(|entry| entry.value().clone())
    }
}

#[derive(Debug)]
pub struct AssetCache {
    assets: DashMap<String, AssetId>,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            assets: DashMap::new(),
        }
    }

    pub fn insert(&self, id: String) -> AssetId {
        let key = id.clone().to_uppercase();
        let asset = Arc::new(id.to_uppercase());
        self.assets.insert(key, asset.clone());
        asset
    }

    pub fn get_by_name(&self, id: String) -> Option<AssetId> {
        let key = id.to_uppercase();
        self.assets.get(&key).map(|entry| entry.value().clone())
    }
}

#[derive(Debug)]
pub struct InstrumentService {
    instrument_repo: InstrumentRepo,
    instrument_cache: InstrumentCache,
    asset_cache: AssetCache,
    venue_service: VenueService,
}

impl InstrumentService {
    pub fn new(instrument_repo: InstrumentRepo, venue_service: VenueService) -> Self {
        Self {
            instrument_repo,
            instrument_cache: InstrumentCache::new(),
            asset_cache: AssetCache::new(),
            venue_service,
        }
    }

    pub async fn insert(&self, instrument: Instrument) -> Result<()> {
        let instrument_repo = &self.instrument_repo;
        instrument_repo.create(instrument).await
    }

    pub async fn read_by_id(&self, id: Uuid) -> Result<Arc<Instrument>> {
        let instrument_cache = &self.instrument_cache;
        let instrument_repo = &self.instrument_repo;
        let venue_service = &self.venue_service;

        // Check cache
        let instrument = match instrument_cache.get_by_id(id) {
            Some(instrument) => instrument,
            None => {
                debug!("Instrument not found in cache");
                if let Some(db_instrument) = instrument_repo.read_by_id(id).await? {
                    let venue = venue_service
                        .read_by_id(db_instrument.venue_id)
                        .await?
                        .ok_or_else(|| Error::msg("Venue not found"))?;

                    let base_asset = self
                        .asset_cache
                        .get_by_name(db_instrument.base_asset.clone())
                        .unwrap_or_else(|| self.asset_cache.insert(db_instrument.base_asset.clone()));

                    let quote_asset = self
                        .asset_cache
                        .get_by_name(db_instrument.quote_asset.clone())
                        .unwrap_or_else(|| self.asset_cache.insert(db_instrument.quote_asset.clone()));

                    let instrument = Instrument {
                        id: db_instrument.id,
                        symbol: db_instrument.symbol,
                        venue_symbol: db_instrument.venue_symbol,
                        venue,
                        instrument_type: db_instrument.instrument_type.into(),
                        base_asset,
                        quote_asset,
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
                    let instrument = instrument_cache.insert(instrument);
                    instrument
                } else {
                    let msg = format!("Instrument with id {:?} not found in the database", id);
                    error!("{}", msg);
                    Err(Error::msg(msg))?
                }
            }
        };
        Ok(instrument)
    }

    pub async fn read_by_venue_symbol(&self, venue_symbol: String) -> Result<Arc<Instrument>> {
        let instrument_cache = &self.instrument_cache;
        let instrument_repo = &self.instrument_repo;
        let venue_service = &self.venue_service;

        // Check cache
        let instrument = match instrument_cache.get_by_venue_symbol(&venue_symbol) {
            Some(instrument) => instrument,
            None => {
                debug!("Instrument not found in cache");
                if let Some(db_instrument) = instrument_repo.read_by_venue_symbol(&venue_symbol).await? {
                    let venue = venue_service
                        .read_by_id(db_instrument.venue_id)
                        .await?
                        .ok_or_else(|| Error::msg("Venue not found"))?;

                    let base_asset = self
                        .asset_cache
                        .get_by_name(db_instrument.base_asset.clone())
                        .unwrap_or_else(|| self.asset_cache.insert(db_instrument.base_asset.clone()));

                    let quote_asset = self
                        .asset_cache
                        .get_by_name(db_instrument.quote_asset.clone())
                        .unwrap_or_else(|| self.asset_cache.insert(db_instrument.quote_asset.clone()));

                    let instrument = Instrument {
                        id: db_instrument.id,
                        symbol: db_instrument.symbol,
                        venue_symbol: db_instrument.venue_symbol,
                        venue,
                        instrument_type: db_instrument.instrument_type.into(),
                        base_asset,
                        quote_asset,
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
                    let instrument = instrument_cache.insert(instrument);
                    instrument
                } else {
                    let msg = format!("Instrument with venue symbol {:?} not found in the database", venue_symbol);
                    error!("{}", msg);
                    Err(Error::msg(msg))?
                }
            }
        };
        Ok(instrument)
    }
}
