use std::sync::Arc;

use arkin_core::Signal;
use derive_builder::Builder;

use crate::{repos::SignalRepo, PersistenceError};

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct SignalStore {
    signal_repo: SignalRepo,
}

impl SignalStore {
    pub async fn insert(&self, signal: Arc<Signal>) -> Result<(), PersistenceError> {
        self.signal_repo.insert(signal.into()).await
    }

    pub async fn insert_batch(&self, signals: Vec<Arc<Signal>>) -> Result<(), PersistenceError> {
        let signals_dto: Vec<_> = signals.into_iter().map(|signal| signal.into()).collect();
        self.signal_repo.insert_batch(signals_dto).await
    }
}
