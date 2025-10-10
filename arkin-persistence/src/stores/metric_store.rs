use std::sync::Arc;

use arkin_core::prelude::*;

use arkin_core::PersistenceError;

use crate::repos::ch::metric_repo;
use crate::{context::PersistenceContext, repos::ch::insight_repo};

pub async fn create_table(ctx: &PersistenceContext) -> Result<(), PersistenceError> {
    metric_repo::create_table(ctx).await
}

pub async fn insert_metric(ctx: &PersistenceContext, metric: Arc<Metric>) -> Result<(), PersistenceError> {
    metric_repo::insert(ctx, metric.into()).await?;
    Ok(())
}

pub async fn batch_insert_metric(ctx: &PersistenceContext, metrics: &[Arc<Metric>]) -> Result<(), PersistenceError> {
    let metrics = metrics.iter().map(|i| i.clone().into()).collect::<Vec<_>>();
    metric_repo::insert_batch(ctx, &metrics).await
}
