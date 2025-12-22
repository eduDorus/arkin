use std::{sync::Arc, time::Duration};

use strum::Display;
use time::UtcDateTime;
use tokio::{select, sync::RwLock};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, instrument};

use crate::{
    barrier::SyncBarrier,
    traits::{Runnable, Subscriber},
    Event, PersistenceReader, PubSubTrait, SystemTime,
};

#[derive(PartialEq, Debug, Copy, Clone, Default, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ServiceState {
    Starting,
    Running,
    Stopping,
    #[default]
    Stopped,
}

pub struct ServiceCtx {
    state: RwLock<ServiceState>,
    tracker: TaskTracker,
    shutdown: CancellationToken,
}

impl Default for ServiceCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceCtx {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(ServiceState::Stopped),
            tracker: TaskTracker::new(),
            shutdown: CancellationToken::new(),
        }
    }

    pub async fn get_state(&self) -> ServiceState {
        *self.state.read().await
    }

    pub async fn is_running(&self) -> bool {
        matches!(self.get_state().await, ServiceState::Starting | ServiceState::Running)
    }

    pub fn get_shutdown_token(&self) -> CancellationToken {
        self.shutdown.clone()
    }

    pub async fn starting(&self) {
        *self.state.write().await = ServiceState::Starting
    }

    pub async fn started(&self) {
        self.tracker.reopen();
        *self.state.write().await = ServiceState::Running
    }

    pub async fn stopping(&self) {
        *self.state.write().await = ServiceState::Stopping
    }

    pub async fn stopped(&self) {
        *self.state.write().await = ServiceState::Stopped;
    }

    pub fn signal_shutdown(&self) {
        self.shutdown.cancel();
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tracker.spawn(future);
    }

    pub async fn wait(&self) {
        self.tracker.close();
        while !self.tracker.is_empty() {
            info!("waiting for {} tasks to stop", self.tracker.len());
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        self.tracker.wait().await;
    }
}

pub struct CoreCtx {
    pub time: Arc<dyn SystemTime>,
    pub pubsub: Arc<dyn PubSubTrait>,
    pub persistence: Arc<dyn PersistenceReader>,
    pub simulation_barrier: RwLock<Option<Arc<SyncBarrier>>>,
}

impl CoreCtx {
    pub fn new(
        time: Arc<dyn SystemTime>,
        pubsub: Arc<dyn PubSubTrait>,
        persistence: Arc<dyn PersistenceReader>,
    ) -> Self {
        Self {
            time,
            pubsub,
            persistence,
            simulation_barrier: RwLock::new(None),
        }
    }

    pub async fn now(&self) -> UtcDateTime {
        self.time.now().await
    }

    pub async fn publish(&self, event: Event) {
        self.pubsub.publish(event).await
    }

    pub async fn publish_batch(&self, events: Vec<Event>) {
        // self.pubsub.publish_batch(events).await
        for event in events {
            self.publish(event).await;
        }
    }
}

pub struct Service {
    service_ctx: Arc<ServiceCtx>,
    core_ctx: Arc<CoreCtx>,
    identifier: String,
    subscriber: Option<Arc<dyn Subscriber>>,
    service: Arc<dyn Runnable>,
}

impl Service {
    pub fn new(
        identifier: &str,
        service: Arc<dyn Runnable>,
        core_ctx: Arc<CoreCtx>,
        subscriber: Option<Arc<dyn Subscriber>>,
    ) -> Arc<Self> {
        Self {
            service_ctx: Arc::new(ServiceCtx::new()),
            core_ctx,
            identifier: identifier.to_owned(),
            subscriber,
            service,
        }
        .into()
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn event_loop(&self) {
        info!(target: "service", "starting event loop");
        // let service_ctx = self.service_ctx.clone();
        let core_ctx = self.core_ctx.clone();
        let service = self.service.clone();
        let token = self.service_ctx.get_shutdown_token();
        let subscriber = self.subscriber.as_ref().expect("Need subscriber for event loop").clone();
        self.service_ctx.spawn(async move {
            loop {
                select! {
                  Some(event) = subscriber.recv() => {
                        // Handle the event
                        service.handle_event(core_ctx.clone(), event.clone()).await;

                        // Send ack if needed
                        if subscriber.needs_ack() {
                            subscriber.send_ack().await;
                        }

                        // MULTI MODE LOGIC - IGNORE FOR NOW
                        // if subscriber.needs_ack() {
                        //     // Simulation mode: Process sequentially
                        //     service.handle_event(core_ctx.clone(), event).await;
                        //     subscriber.send_ack().await;
                        // } else {
                        //     // Live mode: Spawn a task for concurrent processing
                        //     let service_clone = service.clone();
                        //     let ctx_clone = core_ctx.clone();
                        //     service_ctx.spawn(async move {
                        //         service_clone.handle_event(ctx_clone, event).await;
                        //     });
                        // }
                  },
                  _ = token.cancelled() => {
                    break
                  }
                }
            }
        });
        info!(target: "service", "started event loop");
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn start(&self) {
        info!(target: "service", "starting");

        // Starting the service
        self.service_ctx.starting().await;

        // Start the service event_loop
        if self.subscriber.is_some() {
            self.event_loop().await;
        }

        // Run service setup functionality
        self.service.setup(self.service_ctx.to_owned(), self.core_ctx.to_owned()).await;

        // Start the service tasks
        let tasks = self
            .service
            .clone()
            .get_tasks(self.service_ctx.clone(), self.core_ctx.clone())
            .await;
        for task in tasks {
            self.service_ctx.spawn(task);
        }

        // Set service to be running
        self.service_ctx.started().await;
        info!(target: "service", "started");
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn stop(&self) {
        info!(target: "service", "stopping");
        // Stopping the srevice
        self.service_ctx.stopping().await;

        // Run down the tear down process
        self.service
            .teardown(self.service_ctx.to_owned(), self.core_ctx.to_owned())
            .await;

        // Stop the tasks
        self.service_ctx.signal_shutdown();
        self.service_ctx.wait().await;

        // Set to stopped
        self.service_ctx.stopped().await;
        info!(target: "service", "stopped");
    }
}
