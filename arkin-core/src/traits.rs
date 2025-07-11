use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use time::UtcDateTime;

use crate::{Event, ServiceCtx};

#[async_trait]
pub trait SystemTime: Send + Sync {
    async fn now(&self) -> UtcDateTime;
    async fn advance_time_to(&self, time: UtcDateTime);
    async fn advance_time_by(&self, duration: Duration);
    async fn is_final_hour(&self) -> bool;
    async fn is_finished(&self) -> bool;
    async fn is_live(&self) -> bool;
}

/// A trait for defining the lifecycle of a service in the system.
///
/// `Runnable` provides a structured way to manage the initialization, operation, and shutdown of services.
/// Implementors of this trait can define custom behavior for each phase of the service lifecycle:
/// - **Setup**: Initialization logic before starting the main tasks.
/// - **Start Tasks**: Launching the main operational tasks or event loops.
/// - **Stop Tasks**: Gracefully stopping the main tasks.
/// - **Teardown**: Cleaning up resources or connections after tasks have stopped.
///
/// Services implementing `Runnable` are expected to handle their own task management and lifecycle
/// transitions, ensuring they integrate smoothly with the system's service management framework.
#[async_trait]
pub trait Runnable: Sync + Send {
    /// Returns a unique identifier for the service.
    ///
    /// This identifier is used for logging, monitoring, and distinguishing between different services.
    fn identifier(&self) -> &str;

    // async fn event_loop(self: Arc<Self>, _ctx: Arc<ServiceCtx>) {}
    async fn handle_event(&self, _event: Event) {}

    /// Performs initialization logic before starting the main tasks.
    ///
    /// This method is called once when the service is being set up. It can be used to initialize
    /// connections, load configurations, or prepare resources needed by the service.
    ///
    /// The default implementation does nothing, making this method optional for implementors.
    ///
    /// # Parameters
    /// - `ctx`: The service context, providing access to the service's state and task tracker.
    /// - `pubsub`: The publish-subscribe system for event communication.
    async fn setup(&self, _ctx: Arc<ServiceCtx>) {}

    /// Starts the main operational tasks of the service.
    ///
    /// This method is responsible for launching any background tasks, event loops, or other
    /// operational logic that the service requires to function. It is called after `setup` and
    /// should ensure that all necessary tasks are spawned and managed via the `ServiceCtx`.
    ///
    /// The default implementation does nothing, making this method optional for implementors.
    ///
    /// # Parameters
    /// - `self`: An `Arc<Self>` to allow the service to be shared across tasks.
    /// - `ctx`: The service context for managing state and tasks.
    /// - `pubsub`: The publish-subscribe system for event communication.
    async fn start_tasks(self: Arc<Self>, _ctx: Arc<ServiceCtx>) {}

    /// Stops the main operational tasks of the service.
    ///
    /// This method should gracefully shut down any running tasks, ensuring that the service
    /// stops cleanly without leaving resources in an inconsistent state. It is called before
    /// `teardown` and should coordinate with the `ServiceCtx` to wait for tasks to complete.
    ///
    /// The default implementation does nothing, making this method optional for implementors.
    ///
    /// # Parameters
    /// - `self`: An `Arc<Self>` to allow the service to be shared across tasks.
    /// - `ctx`: The service context for managing state and tasks.
    /// - `pubsub`: The publish-subscribe system for event communication.
    async fn stop_tasks(self: Arc<Self>, _ctx: Arc<ServiceCtx>) {}

    /// Performs cleanup logic after stopping the main tasks.
    ///
    /// This method is called once after `stop_tasks` to handle any final cleanup, such as closing
    /// connections, releasing resources, or flushing data. The default implementation does nothing,
    /// making this method optional for implementors.
    ///
    /// # Parameters
    /// - `ctx`: The service context, providing access to the service's state and task tracker.
    /// - `pubsub`: The publish-subscribe system for event communication.
    async fn teardown(&self, _ctx: Arc<ServiceCtx>) {}
}

#[async_trait]
pub trait Publisher: Send + Sync {
    async fn publish(&self, event: Event);
}

#[async_trait]
pub trait Subscriber: Send + Sync {
    async fn recv(&self) -> Option<Event>;
    fn needs_ack(&self) -> bool;
    async fn send_ack(&self);
}
