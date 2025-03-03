use std::{collections::HashMap, fmt, sync::Arc};

use serde::Deserialize;
use time::OffsetDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{state::InsightsState, Feature};

#[derive(Deserialize, Debug, Clone)]
pub struct ScalerData {
    pub feature_id: FeatureId,
    pub median: f64,
    pub iqr: f64,
    pub percentile_01: f64,
    pub percentile_99: f64,
    pub is_skewed: u8,
    pub skew: f64,
    pub skew_offset: f64,
    pub kurtosis: f64,
}

#[derive(TypedBuilder)]
pub struct RobustScaler {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    #[builder(default)]
    scalers: HashMap<FeatureId, ScalerData>,
    input: Vec<FeatureId>,
    output: FeatureId,
    persist: bool,
}

impl fmt::Debug for RobustScaler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RobustScaler ").field("pipeline", &self.pipeline).finish()
    }
}

impl Feature for RobustScaler {
    fn inputs(&self) -> Vec<FeatureId> {
        self.input.clone()
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Robust scaling...");

        //  Get data
        let insights = self
            .input
            .iter()
            .filter_map(|id| {
                // Get the value
                let value = self.insight_state.last(Some(instrument.clone()), id.clone(), event_time)?;

                // Calculate scaled values
                if let Some(scaler) = self.scalers.get(id) {
                    // Quick check to avoid division by zero
                    if scaler.iqr == 0.0 {
                        warn!("Scaler IQR is zero for {}, please check!", id);
                        return None;
                    }

                    // Clip the value to the 1st and 99th percentiles
                    let value = value.max(scaler.percentile_01).min(scaler.percentile_99);

                    // let scaled_value = if scaler.is_skewed == 1 {
                    //     let transformed_value = (value + scaler.skew_offset).ln();
                    //     (transformed_value - scaler.median) / scaler.iqr
                    // } else {
                    //     (value - scaler.median) / scaler.iqr
                    // };
                    let scaled_value = (value - scaler.median) / scaler.iqr;

                    // Create Insight
                    Some(
                        Insight::builder()
                            .event_time(event_time)
                            .pipeline(Some(self.pipeline.clone()))
                            .instrument(Some(instrument.clone()))
                            .feature_id(id.clone())
                            .value(scaled_value)
                            .insight_type(InsightType::Scaled)
                            .persist(self.persist)
                            .build()
                            .into(),
                    )
                } else {
                    warn!("No scaler data found for {}", id);
                    None
                }
            })
            .collect::<Vec<_>>();

        Some(insights)
    }
}
