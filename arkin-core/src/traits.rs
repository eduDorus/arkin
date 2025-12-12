use std::{pin::Pin, sync::Arc, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use serde::{de::DeserializeOwned, Serialize};
use time::UtcDateTime;

use crate::models::*;
use crate::prelude::*;

#[async_trait]
pub trait EventPayload: Sized + Send + Sync {
    type Dto: Serialize + DeserializeOwned + Send + Sync;

    fn to_dto(&self) -> Self::Dto;
    async fn from_dto(dto: Self::Dto, persistence: Arc<dyn PersistenceReader>) -> Result<Self>;
}

#[async_trait]
pub trait SystemTime: Send + Sync {
    async fn now(&self) -> UtcDateTime;
    async fn advance_time_to(&self, time: UtcDateTime);
    async fn advance_time_by(&self, duration: Duration);
    async fn is_final_hour(&self) -> bool;
    async fn is_finished(&self) -> bool;
    async fn is_live(&self) -> bool;
    async fn check_interval(&self) -> Vec<UtcDateTime>;
}

#[async_trait]
pub trait PersistenceReader: Send + Sync {
    async fn refresh(&self) -> Result<(), PersistenceError>;

    // === Single Item Getters ===
    async fn get_instrument(&self, query: &InstrumentQuery) -> Result<Arc<Instrument>, PersistenceError>;
    async fn get_instance(&self, query: &InstanceQuery) -> Result<Arc<Instance>, PersistenceError>;
    async fn get_pipeline(&self, query: &PipelineQuery) -> Result<Arc<Pipeline>, PersistenceError>;
    async fn get_venue(&self, query: &VenueQuery) -> Result<Arc<Venue>, PersistenceError>;
    async fn get_asset(&self, query: &AssetQuery) -> Result<Arc<Asset>, PersistenceError>;
    async fn get_account(&self, query: &AccountQuery) -> Result<Arc<Account>, PersistenceError>;
    async fn get_strategy(&self, query: &StrategyQuery) -> Result<Arc<Strategy>, PersistenceError>;
    async fn get_feature(&self, query: &FeatureQuery) -> FeatureId;

    // === Query Methods ===
    async fn list_instruments(&self, query: &InstrumentListQuery) -> Result<Vec<Arc<Instrument>>, PersistenceError>;
    async fn list_instances(&self, query: &InstanceListQuery) -> Result<Vec<Arc<Instance>>, PersistenceError>;
    async fn list_pipelines(&self, query: &PipelineListQuery) -> Result<Vec<Arc<Pipeline>>, PersistenceError>;
    async fn list_venues(&self, query: &VenueListQuery) -> Result<Vec<Arc<Venue>>, PersistenceError>;
    async fn list_assets(&self, query: &AssetListQuery) -> Result<Vec<Arc<Asset>>, PersistenceError>;
    async fn list_accounts(&self, query: &AccountListQuery) -> Result<Vec<Arc<Account>>, PersistenceError>;
    async fn list_strategies(&self, query: &StrategyListQuery) -> Result<Vec<Arc<Strategy>>, PersistenceError>;
    async fn list_features(&self, query: &FeatureListQuery) -> Result<Vec<FeatureId>, PersistenceError>;

    // === Trade Data ===
    async fn list_trades(
        &self,
        instruments: &[Arc<Instrument>],
        start: UtcDateTime,
        end: UtcDateTime,
    ) -> Result<Vec<Arc<AggTrade>>, PersistenceError>;

    async fn get_last_tick(&self, instrument: &Arc<Instrument>) -> Result<Option<Arc<Tick>>, PersistenceError>;

    // === Streams ===
    async fn agg_trade_stream_range_buffered(
        &self,
        instruments: &[Arc<Instrument>],
        start: UtcDateTime,
        end: UtcDateTime,
        buffer_size: usize,
        frequency: Frequency,
    ) -> Result<Box<dyn Stream<Item = Event> + Send + Unpin>, PersistenceError>;

    async fn tick_stream_range_buffered(
        &self,
        instruments: &[Arc<Instrument>],
        start: UtcDateTime,
        end: UtcDateTime,
        buffer_size: usize,
        frequency: Frequency,
    ) -> Result<Box<dyn Stream<Item = Event> + Send + Unpin>, PersistenceError>;

    async fn metric_stream_range_buffered(
        &self,
        instruments: &[Arc<Instrument>],
        metric_type: MetricType,
        start: UtcDateTime,
        end: UtcDateTime,
        buffer_size: usize,
        frequency: Frequency,
    ) -> Result<Box<dyn Stream<Item = Event> + Send + Unpin>, PersistenceError>;
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
    fn event_filter(&self, _instance_type: InstanceType) -> EventFilter {
        EventFilter::None
    }

    // async fn event_loop(self: Arc<Self>, _ctx: Arc<ServiceCtx>) {}
    async fn handle_event(&self, _core_ctx: Arc<CoreCtx>, _event: Event) {}

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
    async fn setup(&self, _service_ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {}

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
    async fn get_tasks(
        self: Arc<Self>,
        _service_ctx: Arc<ServiceCtx>,
        _core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        vec![]
    }

    /// Performs cleanup logic after stopping the main tasks.
    ///
    /// This method is called once after `stop_tasks` to handle any final cleanup, such as closing
    /// connections, releasing resources, or flushing data. The default implementation does nothing,
    /// making this method optional for implementors.
    ///
    /// # Parameters
    /// - `ctx`: The service context, providing access to the service's state and task tracker.
    /// - `pubsub`: The publish-subscribe system for event communication.
    async fn teardown(&self, _service_ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {}
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

#[async_trait]
pub trait PubSubTrait: Send + Sync {
    fn subscribe(&self, filter: EventFilter) -> Arc<dyn Subscriber>;
    fn publisher(&self) -> Arc<dyn Publisher>;
}
