use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

use arkin_core::prelude::*;
use arkin_persistance::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, info};

use crate::ComputationGraph;
use crate::{config::InsightsServiceConfig, state::InsightsState};

pub trait FeatureModule: Debug + Send + Sync {
    fn inputs(&self) -> &[NodeId];
    fn outputs(&self) -> &[NodeId];
    // fn data(&self) -> &[DataRequest];
    fn calculate(
        &self,
        instruments: &[Instrument],
        timestamp: &OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>>;
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

    pub fn insert(&self, insight: Insight) {
        self.state.insert(insight);
    }

    pub fn insert_batch(&self, insights: Vec<Insight>) {
        self.state.insert_batch(insights);
    }

    pub async fn process(&self, instruments: &[Instrument], from: &OffsetDateTime, to: &OffsetDateTime) -> Result<()> {
        info!("Running insights pipeline from {} to {}", from, to);
        // Fetch data from persistance service
        // let instrument_ids = instruments.iter().map(|i| i.id).collect::<Vec<Uuid>>();
        // let trades = self.persistance_service.read_trades_range(&instrument_ids, from, to).await?;
        // let ticks = self.persistance_service.read_ticks_range(&instrument_ids, from, to).await?;

        // // Transform data to insights and add to state
        // trades
        //     .into_iter()
        //     .flat_map(|trade| trade.to_insights())
        //     .for_each(|event| self.state.insert(event));
        // ticks
        //     .into_iter()
        //     .flat_map(|tick| tick.to_insights())
        //     .for_each(|event| self.state.insert(event));

        // Generate insights
        let insights = self.pipeline.calculate(self.state.clone(), instruments, to);

        for insight in &insights {
            debug!("Generated insight: {}", insight);
        }

        self.persistance_service.insert_insight_batch(insights).await?;
        Ok(())
    }
}
