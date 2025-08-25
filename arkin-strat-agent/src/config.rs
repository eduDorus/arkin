use arkin_core::FeatureId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentStrategyConfig {
    pub strategy_agent: ComponentConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentConfig {
    pub strategy: StrategySettings,
    pub model: ModelSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrategySettings {
    pub capital_per_inst: Decimal,
    pub input_feature_ids: Vec<FeatureId>,
    pub input_state_ids: Vec<FeatureId>,
    pub inference_interval: u64,
    pub inference_host: String,
    pub inference_port: u16,
    pub inference_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelSettings {
    pub model_name_prefix: String,
    pub model_name_postfix: String,
    pub batch_size: usize,
    pub sequence_length: usize,
    pub num_features_obs: usize,
    pub num_state_obs: usize,
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
    pub output_weight_name: String,
    pub possible_weights: Vec<f32>,
}
