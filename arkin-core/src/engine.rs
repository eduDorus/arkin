use futures::future::join_all;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};

use crate::service::Service;

#[derive(Default)]
pub struct Engine {
    services: RwLock<HashMap<String, Arc<Service>>>,
    start_order: RwLock<BTreeMap<u64, Vec<String>>>,
    stop_order: RwLock<BTreeMap<u64, Vec<String>>>,
}

impl Engine {
    pub fn new() -> Arc<Self> {
        Self {
            services: RwLock::new(HashMap::new()),
            start_order: RwLock::new(BTreeMap::new()),
            stop_order: RwLock::new(BTreeMap::new()),
        }
        .into()
    }

    pub async fn register(&self, service: Arc<Service>, start_priority: u64, stop_priority: u64) {
        info!(target: "engine", "register services {}", service.identifier());
        let name = service.identifier().to_owned();
        self.services.write().await.insert(name.clone(), service);

        self.start_order
            .write()
            .await
            .entry(start_priority)
            .or_insert_with(Vec::new)
            .push(name.clone());

        self.stop_order
            .write()
            .await
            .entry(stop_priority)
            .or_insert_with(Vec::new)
            .push(name);
    }

    #[instrument(parent = None, skip_all)]
    pub async fn start(&self) {
        info!(target: "engine", "starting service");
        let services = self.services.read().await;
        let start_order = self.start_order.read().await;
        for (_priority, service_names) in start_order.iter() {
            let mut handles = vec![];
            for name in service_names {
                if let Some(service) = services.get(name) {
                    handles.push(service.start_service());
                }
            }
            if !handles.is_empty() {
                let jh = join_all(handles);
                jh.await;
            }
        }
        info!(target: "engine", "started services");
    }

    #[instrument(parent = None, skip_all)]
    pub async fn stop(&self) {
        info!(target: "engine", "stopping services");
        let services = self.services.read().await;
        let stop_order = self.stop_order.read().await;
        for (_priority, service_names) in stop_order.iter() {
            let mut handles = vec![];
            for name in service_names {
                if let Some(service) = services.get(name) {
                    handles.push(service.stop_service());
                }
            }
            if !handles.is_empty() {
                let jh = join_all(handles);
                jh.await;
            }
        }
        info!(target: "engine", "stopped services");
    }
}
