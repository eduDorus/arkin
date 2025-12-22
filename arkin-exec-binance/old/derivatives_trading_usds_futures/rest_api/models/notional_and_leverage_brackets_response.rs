#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NotionalAndLeverageBracketsResponse {
    NotionalAndLeverageBracketsResponse1(Vec<models::NotionalAndLeverageBracketsResponse1Inner>),
    NotionalAndLeverageBracketsResponse2(Box<models::NotionalAndLeverageBracketsResponse2>),
    Other(serde_json::Value),
}

impl Default for NotionalAndLeverageBracketsResponse {
    fn default() -> Self {
        Self::NotionalAndLeverageBracketsResponse1(Default::default())
    }
}
