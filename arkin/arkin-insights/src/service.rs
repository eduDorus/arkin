use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use rayon::prelude::*;
use rust_decimal::Decimal;

use arkin_core::prelude::*;
use arkin_persistance::prelude::*;
use time::OffsetDateTime;
use tracing::info;
use uuid::Uuid;

use crate::ComputationGraph;
use crate::{
    config::InsightsServiceConfig,
    state::{DataRequest, DataResponse, InsightsState},
};

pub trait FeatureModule: Debug + Send + Sync {
    fn id(&self) -> &NodeId;
    fn sources(&self) -> &[NodeId];
    fn data(&self) -> &[DataRequest];
    fn calculate(&self, data: DataResponse) -> Result<HashMap<FeatureId, Decimal>>;
}

pub struct InsightsService {
    state: Arc<InsightsState>,
    persistance_service: Arc<PersistanceService>,
    pipeline: ComputationGraph,
}

impl InsightsService {
    pub fn from_config(config: &InsightsServiceConfig, persistance_service: Arc<PersistanceService>) -> Self {
        Self {
            state: Arc::new(InsightsState::default()),
            persistance_service,
            pipeline: ComputationGraph::from_config(&config.pipeline),
        }
    }

    pub async fn process(&self, instruments: &[Instrument], from: &OffsetDateTime, to: &OffsetDateTime) -> Result<()> {
        // Fetch data from persistance service
        let instrument_ids = instruments.iter().map(|i| i.id).collect::<Vec<Uuid>>();
        let trades = self.persistance_service.read_trades_range(&instrument_ids, from, to).await?;
        let ticks = self.persistance_service.read_ticks_range(&instrument_ids, from, to).await?;

        // Transform data to insights and add to state
        trades
            .into_iter()
            .flat_map(|trade| trade.to_insights())
            .for_each(|event| self.state.insert(event));
        ticks
            .into_iter()
            .flat_map(|tick| tick.to_insights())
            .for_each(|event| self.state.insert(event));

        // Generate insights
        let insights = instruments
            .par_iter()
            .map(|instrument| self.pipeline.calculate(self.state.clone(), &to, instrument))
            .flat_map(|f| f)
            .collect::<Vec<_>>();

        for insight in &insights {
            info!("Generated insight: {}", insight);
        }

        self.persistance_service.insert_insight_batch(insights).await?;
        Ok(())
    }
}
