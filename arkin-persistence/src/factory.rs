use std::sync::Arc;

use arkin_core::{load, PubSub};

use crate::{PersistenceConfig, PersistenceService};

pub struct PersistenceFactory {}

impl PersistenceFactory {
    pub async fn init_from_config(pubsub: Arc<PubSub>, dry_run: bool) -> Arc<PersistenceService> {
        let config = load::<PersistenceConfig>();
        Arc::new(PersistenceService::from_config(&config, pubsub, dry_run).await)
    }
}
