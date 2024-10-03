use std::{sync::Arc, time::Duration};

use anyhow::Result;
use arkin_core::prelude::*;
use rayon::prelude::*;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::debug;

use crate::{config::CountFeatureConfig, service::FeatureModule, state::InsightsState};

#[derive(Debug)]
pub struct CountFeature {
    inputs: Vec<NodeId>,
    outputs: Vec<NodeId>,
    window: Duration,
}

impl CountFeature {
    pub fn from_config(config: &CountFeatureConfig) -> Self {
        Self {
            inputs: config.inputs.to_owned(),
            outputs: config.outputs.to_owned(),
            window: Duration::from_secs(config.window),
        }
    }
}

impl FeatureModule for CountFeature {
    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn outputs(&self) -> &[NodeId] {
        &self.outputs
    }

    fn calculate(
        &self,
        instruments: &[Instrument],
        timestamp: &OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating count with id: {}", self.inputs[0]);

        // Query the data from the internal state
        let data: Vec<Insight> = self
            .inputs()
            .par_iter()
            .flat_map(|feature| {
                instruments.par_iter().map(|instrument| {
                    let entries = state.list_entries_window(instrument, feature, timestamp, &self.window);
                    let count = Decimal::from(entries.len());
                    Insight::new(timestamp.clone(), instrument.clone(), feature.clone(), count)
                })
            })
            .collect();

        state.insert_batch(data.clone());
        Ok(data)
    }
}
