use std::sync::Arc;

use arkin_core::prelude::*;
use clickhouse::Client;
use futures::lock::Mutex;
use moka2::future::Cache;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PersistenceContext {
    pub pg_pool: PgPool,
    pub ch_client: Client, // Or Arc if not Clone
    pub instance: Arc<Instance>,
    pub cache: Caches,
    pub buffer: Buffers,
}

impl PersistenceContext {
    pub fn new(pg_pool: PgPool, ch_client: Client, instance: Arc<Instance>) -> Self {
        Self {
            pg_pool,
            ch_client,
            instance,
            cache: Caches::new(1000),
            buffer: Buffers::default(),
        }
    }
}

#[derive(Clone)]
pub struct Caches {
    pub asset_id: Cache<Uuid, Arc<Asset>>,
    pub asset_symbol: Cache<String, Arc<Asset>>,
    pub instrument_id: Cache<Uuid, Arc<Instrument>>,
    pub instrument_venue_symbol: Cache<String, Arc<Instrument>>,
    pub venue_id: Cache<Uuid, Arc<Venue>>,
    pub venue_name: Cache<String, Arc<Venue>>,
}

impl Caches {
    pub fn new(capacity: u64) -> Self {
        Self {
            asset_id: Cache::new(capacity),
            asset_symbol: Cache::new(capacity),
            instrument_id: Cache::new(capacity),
            instrument_venue_symbol: Cache::new(capacity),
            venue_id: Cache::new(capacity),
            venue_name: Cache::new(capacity),
        }
    }
}

#[derive(Clone, Default)]
pub struct Buffers {
    pub insights: Arc<Mutex<Vec<Arc<Insight>>>>,
    pub ticks: Arc<Mutex<Vec<Arc<Tick>>>>,
    pub trades: Arc<Mutex<Vec<Arc<AggTrade>>>>,
}
