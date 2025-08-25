use async_trait::async_trait;
use std::collections::HashMap;

use rust_decimal::Decimal;

#[async_trait]
pub trait InferenceService: Send + Sync {
    async fn request(
        &self,
        model_name: &str,
        input_names: &[&str],
        input_values: &[&[f32]],
        shapes: &[&[i64]],
        output_names: &[&str],
    ) -> Option<HashMap<String, Vec<Decimal>>>;
}
