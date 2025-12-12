use futures::future::join_all;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{info, instrument};

use crate::service::Service;
use crate::{CoreCtx, EventFilter, InstanceType, PersistenceReader, PubSubTrait, Runnable, Subscriber, SystemTime};

#[derive(Clone)]
struct ServiceEntry {
    service: Arc<Service>,
    start_priority: u64,
    stop_priority: u64,
}

pub struct Engine {
    core_ctx: Arc<CoreCtx>,
    services: Vec<ServiceEntry>,
    instance_type: InstanceType,
}

impl Engine {
    pub fn new(
        time: Arc<dyn SystemTime>,
        pubsub: Arc<dyn PubSubTrait>,
        persistence: Arc<dyn PersistenceReader>,
        instance_type: InstanceType,
    ) -> Self {
        let engine = Self {
            core_ctx: Arc::new(CoreCtx::new(time, pubsub, persistence)),
            services: Vec::new(),
            instance_type,
        };

        engine
    }

    pub fn register(&mut self, identifier: &str, service: Arc<dyn Runnable>, start_priority: u64, stop_priority: u64) {
        let subscriber: Option<Arc<dyn Subscriber>> =
            if matches!(service.event_filter(self.instance_type), EventFilter::None) {
                None
            } else {
                Some(self.core_ctx.pubsub.subscribe(service.event_filter(self.instance_type)))
            };

        let svc = Service::new(identifier, service, self.core_ctx.clone(), subscriber);
        info!(target: "engine", "register services {}", identifier);

        self.services.push(ServiceEntry {
            service: svc,
            start_priority,
            stop_priority,
        });
    }

    #[instrument(parent = None, skip_all)]
    pub async fn start(&self) {
        info!(target: "engine", "starting service");
        let mut sorted = self.services.clone();
        sorted.sort_by_key(|e| e.start_priority);

        // Group by priority to handle concurrent starts at same priority
        let mut priority_groups: BTreeMap<u64, Vec<&ServiceEntry>> = BTreeMap::new();
        for entry in &sorted {
            priority_groups.entry(entry.start_priority).or_insert_with(Vec::new).push(entry);
        }

        for (_priority, entries) in priority_groups.iter() {
            let mut handles = vec![];
            for entry in entries {
                handles.push(entry.service.start());
            }
            if !handles.is_empty() {
                join_all(handles).await;
            }
        }
        info!(target: "engine", "started services");
    }

    pub async fn wait_for_shutdown(&self) {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint = signal(SignalKind::interrupt()).unwrap();

        // Wait for first shutdown signal
        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received terminate signal, starting graceful shutdown...");
            },
            _ = sigint.recv() => {
                info!("Received interrupt signal, starting graceful shutdown...");
            },
        }

        // Now perform shutdown, but allow interrupt for force
        tokio::select! {
            _ = self.stop() => {
                info!("Graceful shutdown completed successfully!");
            },
            _ = sigint.recv() => {
                info!("Received second interrupt signal, forcing exit...");
                std::process::exit(130);
            },
        }
    }

    #[instrument(parent = None, skip_all)]
    pub async fn stop(&self) {
        info!(target: "engine", "stopping services");
        let mut sorted = self.services.clone();
        sorted.sort_by_key(|e| std::cmp::Reverse(e.stop_priority));

        // Group by priority to handle concurrent stops at same priority
        let mut priority_groups: BTreeMap<std::cmp::Reverse<u64>, Vec<&ServiceEntry>> = BTreeMap::new();
        for entry in &sorted {
            priority_groups
                .entry(std::cmp::Reverse(entry.stop_priority))
                .or_insert_with(Vec::new)
                .push(entry);
        }

        for (_priority, entries) in priority_groups.iter() {
            let mut handles = vec![];
            for entry in entries {
                handles.push(entry.service.stop());
            }
            if !handles.is_empty() {
                join_all(handles).await;
            }
        }
        info!(target: "engine", "stopped services");
    }
}
