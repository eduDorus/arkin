use std::sync::Arc;

use arkin_core::{FeatureId, FeatureListQuery};

use crate::context::PersistenceContext;

pub async fn read_feature_id(ctx: &PersistenceContext, id: &str) -> FeatureId {
    // Get from cache or insert
    match ctx.cache.feature_id.get(id).await {
        Some(feature_id) => feature_id,
        None => {
            let feature_id = FeatureId::new(id.to_owned());
            ctx.cache.feature_id.insert(id.to_string(), feature_id.clone()).await;
            feature_id
        }
    }
}

pub async fn list_features(ctx: &PersistenceContext, _query: &FeatureListQuery) -> Vec<FeatureId> {
    // Get all feature IDs from cache
    // Note: Moka cache doesn't have a direct way to list all keys, so we return empty for now
    // In practice, you might need to maintain a separate list or use a different caching strategy
    vec![]
}
