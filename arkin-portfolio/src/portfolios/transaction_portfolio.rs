use std::sync::Arc;

use arkin_core::PubSub;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct TransactionPortfolio {
    pub pubsub: Arc<PubSub>,
}
