use std::sync::Arc;

use arkin_core::prelude::*;

use arkin_core::PersistenceError;

use crate::{context::PersistenceContext, repos::ch::insight_repo};

pub async fn insert(ctx: &PersistenceContext, insight: Arc<Insight>) -> Result<(), PersistenceError> {
    insight_repo::insert(ctx, insight.into()).await?;
    Ok(())
}

pub async fn insert_vec(ctx: &PersistenceContext, insights: &[Arc<Insight>]) -> Result<(), PersistenceError> {
    let insights = insights
        .iter()
        .filter(|i| i.persist)
        .map(|i| i.clone().into())
        .collect::<Vec<_>>();
    insight_repo::insert_batch(ctx, &insights).await
}
