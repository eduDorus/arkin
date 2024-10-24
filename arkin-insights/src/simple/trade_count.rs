use std::{sync::Arc, time::Duration};

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::{debug, warn};

use crate::{config::TradeCountConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct TradeCountFeature {
    input_side: FeatureId,
    output_buy: FeatureId,
    output_sell: FeatureId,
    output_total: FeatureId,
    output_ratio: FeatureId,
    window: Duration,
}

impl TradeCountFeature {
    pub fn from_config(config: &TradeCountConfig) -> Self {
        Self {
            input_side: config.input_side.to_owned(),
            output_buy: config.output_buy.to_owned(),
            output_sell: config.output_sell.to_owned(),
            output_total: config.output_total.to_owned(),
            output_ratio: config.output_ratio.to_owned(),
            window: Duration::from_secs(config.window),
        }
    }
}

impl Computation for TradeCountFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_side.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![
            self.output_buy.clone(),
            self.output_sell.clone(),
            self.output_total.clone(),
            self.output_ratio.clone(),
        ]
    }

    fn calculate(
        &self,
        instruments: &[Arc<Instrument>],
        timestamp: OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating trade count feature");

        // Get data from state

        // Calculate the trade count
        let insights = instruments
            .iter()
            .filter_map(|instrument| {
                let data = state.window(Some(instrument.clone()), self.input_side.clone(), timestamp, self.window);

                if data.is_empty() {
                    warn!("Trade side data is empty, cannot calculate trade count");
                    return None;
                }

                // Calculate counts
                let buy_count = data.iter().filter(|x| *x > &Decimal::ZERO).count().into();
                let sell_count = data.iter().filter(|x| *x < &Decimal::ZERO).count().into();
                let total_count = buy_count + sell_count;

                // Create insights
                let buy_count_insight =
                    Insight::new(timestamp, Some(instrument.clone()), self.output_buy.clone(), buy_count);
                let sell_count_insight =
                    Insight::new(timestamp, Some(instrument.clone()), self.output_sell.clone(), sell_count);
                let total_count_insight =
                    Insight::new(timestamp, Some(instrument.clone()), self.output_total.clone(), total_count);
                // Buy Sell Ratio Occilator (buy - sell) / (total)
                let buy_sell_ratio = Insight::new(
                    timestamp,
                    Some(instrument.clone()),
                    self.output_ratio.clone(),
                    (buy_count - sell_count) / total_count,
                );

                Some(vec![buy_count_insight, sell_count_insight, total_count_insight, buy_sell_ratio])
            })
            .flatten()
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
