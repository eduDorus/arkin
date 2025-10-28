use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;

use arkin_core::prelude::*;

use crate::{FeatureStore, InstrumentScope};

/// Strategy for handling missing data in time-series queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FillStrategy {
    /// Forward fill - use last valid value (default, best for prices)
    #[default]
    ForwardFill,
    /// Forward fill with mean of lookback window
    // ForwardFillWithMean(u64),
    /// Fill with zero (good for volume, counts, deltas)
    Zero,
    /// Skip missing values (current behavior, may cause shorter windows)
    Drop,
}

#[async_trait]
pub trait Feature: std::fmt::Debug + Send + Sync {
    fn inputs(&self) -> Vec<FeatureId>;
    fn outputs(&self) -> Vec<FeatureId>;
    fn calculate(
        &self,
        state: &FeatureStore,
        pipeline: &Arc<Pipeline>,
        event_time: UtcDateTime,
    ) -> Option<Vec<Arc<Insight>>>;

    /// Get the fill strategy for this feature's input data
    /// Default is ForwardFill, which is suitable for most price-based features
    fn fill_strategy(&self) -> FillStrategy {
        FillStrategy::ForwardFill
    }

    /// Get the instrument scopes that this feature operates on
    /// Returns a reference to the scopes if the feature has them
    fn scopes(&self) -> &[InstrumentScope] {
        &[] // Default implementation returns empty slice
    }
}
