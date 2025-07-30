use std::sync::Arc;

use time::UtcDateTime;

use arkin_core::prelude::*;

use crate::{context::PersistenceContext, repos::ch::scaler_repo, PersistenceError};

pub async fn get_iqr(
    ctx: &PersistenceContext,
    pipeline: &Arc<Pipeline>,
    instrument: &Arc<Instrument>,
    from: UtcDateTime,
    till: UtcDateTime,
    levels: &[f64],
) -> Result<Vec<QuantileData>, PersistenceError> {
    scaler_repo::get_iqr(ctx, pipeline.id, instrument.id, from, till, levels).await
}
