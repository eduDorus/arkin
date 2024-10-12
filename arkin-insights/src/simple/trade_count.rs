use std::{sync::Arc, time::Duration};

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::{debug, warn};

use crate::{config::TradeCountConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct TradeCountFeature {
    trade_side_input: NodeId,
    buy_output: NodeId,
    sell_output: NodeId,
    total_output: NodeId,
    ratio_output: NodeId,
    window: Duration,
}

impl TradeCountFeature {
    pub fn from_config(config: &TradeCountConfig) -> Self {
        Self {
            trade_side_input: config.trade_side_input.to_owned(),
            buy_output: config.buy_output.to_owned(),
            sell_output: config.sell_output.to_owned(),
            total_output: config.total_output.to_owned(),
            ratio_output: config.ratio_output.to_owned(),
            window: Duration::from_secs(config.window),
        }
    }
}

impl Computation for TradeCountFeature {
    fn inputs(&self) -> Vec<NodeId> {
        vec![self.trade_side_input.clone()]
    }

    fn outputs(&self) -> Vec<NodeId> {
        vec![
            self.buy_output.clone(),
            self.sell_output.clone(),
            self.total_output.clone(),
            self.ratio_output.clone(),
        ]
    }

    fn calculate(
        &self,
        instruments: &[Instrument],
        timestamp: &OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating trade count feature");

        // Get data from state

        // Calculate the trade count
        let insights = instruments
            .iter()
            .filter_map(|instrument| {
                let data =
                    state.get_window_by_instrument(Some(instrument), &self.trade_side_input, timestamp, &self.window);

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
                    Insight::new(timestamp.clone(), Some(instrument.clone()), self.buy_output.clone(), buy_count);
                let sell_count_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.sell_output.clone(),
                    sell_count,
                );
                let total_count_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.total_output.clone(),
                    total_count,
                );
                // Buy Sell Ratio Occilator (buy - sell) / (total)
                let buy_sell_ratio = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.ratio_output.clone(),
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
