use std::sync::Arc;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::BollingerBandsConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct BollingerBandsFeature {
    input_price: NodeId,
    input_sma: NodeId,
    input_stddev: NodeId,
    output_upper: NodeId,
    output_lower: NodeId,
    output_oscillator: NodeId,
    output_width: NodeId,
    sigma: Decimal,
}

impl BollingerBandsFeature {
    pub fn from_config(config: &BollingerBandsConfig) -> Self {
        BollingerBandsFeature {
            input_price: config.input_price.to_owned(),
            input_sma: config.input_sma.to_owned(),
            input_stddev: config.input_stddev.to_owned(),
            output_upper: config.output_upper.to_owned(),
            output_lower: config.output_lower.to_owned(),
            output_oscillator: config.output_oscillator.to_owned(),
            output_width: config.output_width.to_owned(),
            sigma: config.sigma,
        }
    }
}

impl Computation for BollingerBandsFeature {
    fn inputs(&self) -> Vec<NodeId> {
        vec![self.input_price.clone(), self.input_sma.clone(), self.input_stddev.clone()]
    }

    fn outputs(&self) -> Vec<NodeId> {
        vec![
            self.output_upper.clone(),
            self.output_lower.clone(),
            self.output_oscillator.clone(),
            self.output_width.clone(),
        ]
    }

    fn calculate(
        &self,
        instruments: &[Instrument],
        timestamp: &OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating BollingerBands");

        // Calculate the mean (BollingerBands)
        let insights = instruments
            .iter()
            .filter_map(|instrument| {
                // Get data
                let price = state.get_last(Some(instrument), &self.input_price, timestamp);
                let sma = state.get_last(Some(instrument), &self.input_sma, timestamp);
                let std_dev = state.get_last(Some(instrument), &self.input_stddev, timestamp);

                // Check if we have enough data
                match (price, sma, std_dev) {
                    (Some(price), Some(ma), Some(std_dev)) => {
                        // Calculate BollingerBands
                        let upper = ma + std_dev * self.sigma;
                        let lower = ma - std_dev * self.sigma;
                        let oscillator = (price - lower) / (upper - lower);
                        let width = (upper - lower) / ma;

                        Some(vec![
                            Insight::new(timestamp.clone(), Some(instrument.clone()), self.output_upper.clone(), upper),
                            Insight::new(timestamp.clone(), Some(instrument.clone()), self.output_lower.clone(), lower),
                            Insight::new(
                                timestamp.clone(),
                                Some(instrument.clone()),
                                self.output_oscillator.clone(),
                                oscillator,
                            ),
                            Insight::new(timestamp.clone(), Some(instrument.clone()), self.output_width.clone(), width),
                        ])
                    }
                    _ => {
                        warn!("Not enough data to calculate BollingerBands");
                        None
                    }
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
