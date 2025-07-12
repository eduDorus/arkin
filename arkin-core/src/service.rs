use std::{sync::Arc, time::Duration};

use strum::Display;
use tokio::{select, sync::RwLock};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, instrument};

use crate::traits::{Runnable, Subscriber};

#[derive(PartialEq, Debug, Copy, Clone, Default, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ServiceState {
    Starting,
    Running,
    Stopping,
    #[default]
    Stopped,
}

#[derive(Debug, Default)]
pub struct ServiceCtx {
    state: RwLock<ServiceState>,
    tracker: TaskTracker,
    shutdown: CancellationToken,
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

pub struct Service {
    ctx: Arc<ServiceCtx>,
    service: Arc<dyn Runnable>,
    subscriber: Option<Arc<dyn Subscriber>>,
}

impl Service {
    pub fn new(service: Arc<dyn Runnable>, subscriber: Option<Arc<dyn Subscriber>>) -> Arc<Self> {
        Self {
            ctx: Arc::new(ServiceCtx::new()),
            subscriber,
            service,
        }
        .into()
    }

    pub fn identifier(&self) -> &str {
        self.service.identifier()
    }

    pub async fn event_loop(&self) {
        info!(target: "service", "starting event loop");
        let ctx = self.ctx.clone();
        let service = self.service.clone();
        let token = self.ctx.get_shutdown_token();
        let subscriber = self.subscriber.as_ref().expect("Need subscriber for event loop").clone();
        self.ctx.spawn(async move {
            loop {
                select! {
                  Some(event) = subscriber.recv() => {
                        if subscriber.needs_ack() {
                            // Simulation mode: Process sequentially
                            service.handle_event(event).await;
                            subscriber.send_ack().await;
                        } else {
                            // Live mode: Spawn a task for concurrent processing
                            let service_clone = service.clone();
                            ctx.spawn(async move {
                                service_clone.handle_event(event).await;
                            });
                        }
                  },
                  _ = token.cancelled() => {
                    break
                  }
                }
            }
        });
        info!(target: "service", "started event loop");
    }

    #[instrument(parent = None, skip_all, fields(service = %self.service.identifier()))]
    pub async fn start_service(&self) {
        info!(target: "service", "starting");
        self.ctx.starting().await;
        self.service.setup(self.ctx.to_owned()).await;
        if self.subscriber.is_some() {
            self.event_loop().await;
        }
        self.service.clone().start_tasks(self.ctx.clone()).await;
        self.ctx.started().await;
        info!(target: "service", "started");
    }

    #[instrument(parent = None, skip_all, fields(service = %self.service.identifier()))]
    pub async fn stop_service(&self) {
        info!(target: "service", "stopping");
        self.ctx.stopping().await;
        self.service.clone().stop_tasks(self.ctx.clone()).await;
        self.ctx.signal_shutdown();
        self.ctx.wait().await;
        self.service.teardown(self.ctx.to_owned()).await;
        self.ctx.stopped().await;
        info!(target: "service", "stopped");
    }
}
