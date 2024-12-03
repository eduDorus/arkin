use std::sync::Arc;

use derive_builder::Builder;
use moka2::future::Cache;
use tracing::debug;
use uuid::Uuid;

use arkin_core::Instrument;

use crate::{repos::InstrumentRepo, PersistenceError};

use super::{asset::AssetStore, venue::VenueStore};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct InstrumentStore {
    venue_store: VenueStore,
    asset_store: AssetStore,
    instrument_repo: InstrumentRepo,
    #[builder(default = Cache::new(1000))]
    instrument_id_cache: Cache<Uuid, Arc<Instrument>>,
    #[builder(default = Cache::new(1000))]
    instrument_venue_symbol_cache: Cache<String, Arc<Instrument>>,
}

impl InstrumentStore {
    async fn update_instrument_cache(&self, instrument: Arc<Instrument>) {
        self.instrument_id_cache.insert(instrument.id, instrument.clone()).await;
        self.instrument_venue_symbol_cache
            .insert(instrument.venue_symbol.clone(), instrument.clone())
            .await;
    }

    async fn read_cache_by_id(&self, id: &Uuid) -> Option<Arc<Instrument>> {
        self.instrument_id_cache.get(id).await
    }

    async fn read_cache_by_venue_symbol(&self, venue_symbol: &str) -> Option<Arc<Instrument>> {
        self.instrument_venue_symbol_cache.get(venue_symbol).await
    }

    pub async fn insert(&self, instrument: Arc<Instrument>) -> Result<(), PersistenceError> {
        self.update_instrument_cache(instrument.clone()).await;
        self.instrument_repo.insert(instrument.into()).await
    }

    pub async fn read_by_id(&self, id: &Uuid) -> Result<Arc<Instrument>, PersistenceError> {
        // Check cache
        match self.read_cache_by_id(id).await {
            Some(instrument) => Ok(instrument),
            None => {
                debug!("Instrument not found in cache");
                let instrument_dto = self.instrument_repo.read_by_id(id).await?;
                let venue = self.venue_store.read_by_id(&instrument_dto.venue_id).await?;

                let base_asset = self.asset_store.read_by_id(&instrument_dto.base_asset_id).await?;
                let quote_asset = self.asset_store.read_by_id(&instrument_dto.quote_asset_id).await?;

                let instrument = Instrument {
                    id: instrument_dto.id,
                    symbol: instrument_dto.symbol,
                    venue_symbol: instrument_dto.venue_symbol,
                    venue,
                    instrument_type: instrument_dto.instrument_type.into(),
                    base_asset,
                    quote_asset,
                    maturity: instrument_dto.maturity,
                    strike: instrument_dto.strike,
                    option_type: instrument_dto.option_type.map(|v| v.into()),
                    contract_size: instrument_dto.contract_size,
                    price_precision: instrument_dto.price_precision as u32,
                    quantity_precision: instrument_dto.quantity_precision as u32,
                    base_precision: instrument_dto.base_precision as u32,
                    quote_precision: instrument_dto.quote_precision as u32,
                    tick_size: instrument_dto.tick_size,
                    lot_size: instrument_dto.lot_size,
                    status: instrument_dto.status.into(),
                };

                // Update cache
                let instrument = Arc::new(instrument);
                self.update_instrument_cache(instrument.clone()).await;
                Ok(instrument)
            }
        }
    }

    pub async fn read_by_venue_symbol(&self, venue_symbol: &str) -> Result<Arc<Instrument>, PersistenceError> {
        // Check cache
        match self.read_cache_by_venue_symbol(venue_symbol).await {
            Some(instrument) => Ok(instrument),
            None => {
                debug!("Instrument not found in cache");
                let instrument_dto = self.instrument_repo.read_by_venue_symbol(&venue_symbol).await?;
                let venue = self.venue_store.read_by_id(&instrument_dto.venue_id).await?;

                let base_asset = self.asset_store.read_by_id(&instrument_dto.base_asset_id).await?;
                let quote_asset = self.asset_store.read_by_id(&instrument_dto.quote_asset_id).await?;

                let instrument = Instrument {
                    id: instrument_dto.id,
                    symbol: instrument_dto.symbol,
                    venue_symbol: instrument_dto.venue_symbol,
                    venue,
                    instrument_type: instrument_dto.instrument_type.into(),
                    base_asset,
                    quote_asset,
                    maturity: instrument_dto.maturity,
                    strike: instrument_dto.strike,
                    option_type: instrument_dto.option_type.map(|v| v.into()),
                    contract_size: instrument_dto.contract_size,
                    price_precision: instrument_dto.price_precision as u32,
                    quantity_precision: instrument_dto.quantity_precision as u32,
                    base_precision: instrument_dto.base_precision as u32,
                    quote_precision: instrument_dto.quote_precision as u32,
                    tick_size: instrument_dto.tick_size,
                    lot_size: instrument_dto.lot_size,
                    status: instrument_dto.status.into(),
                };

                // Update cache
                let instrument = Arc::new(instrument);
                self.update_instrument_cache(instrument.clone()).await;
                Ok(instrument)
            }
        }
    }
}
