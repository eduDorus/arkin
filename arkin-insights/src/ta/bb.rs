use std::sync::Arc;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::BollingerBandsConfig, state::InsightsState, Computation};

#[derive(Debug)]
pub struct BollingerBandsFeature {
    input_price: FeatureId,
    input_sma: FeatureId,
    input_stddev: FeatureId,
    output_upper: FeatureId,
    output_lower: FeatureId,
    output_oscillator: FeatureId,
    output_width: FeatureId,
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
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_price.clone(), self.input_sma.clone(), self.input_stddev.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![
            self.output_upper.clone(),
            self.output_lower.clone(),
            self.output_oscillator.clone(),
            self.output_width.clone(),
        ]
    }

    fn calculate(
        &self,
        instruments: &[Arc<Instrument>],
        timestamp: OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating BollingerBands");

        // Calculate the mean (BollingerBands)
        let insights = instruments
            .iter()
            .cloned()
            .filter_map(|instrument| {
                // Get data
                let price = state.last(Some(instrument.clone()), self.input_price.clone(), timestamp);
                let sma = state.last(Some(instrument.clone()), self.input_sma.clone(), timestamp);
                let std_dev = state.last(Some(instrument.clone()), self.input_stddev.clone(), timestamp);

                // Check if we have enough data
                match (price, sma, std_dev) {
                    (Some(price), Some(ma), Some(std_dev)) => {
                        // Calculate BollingerBands
                        let upper = ma + std_dev * self.sigma;
                        let lower = ma - std_dev * self.sigma;
                        let oscillator = (price - lower) / (upper - lower);
                        let width = (upper - lower) / ma;

                        Some(vec![
                            Insight::new(timestamp, Some(instrument.clone()), self.output_upper.clone(), upper),
                            Insight::new(timestamp, Some(instrument.clone()), self.output_lower.clone(), lower),
                            Insight::new(
                                timestamp,
                                Some(instrument.clone()),
                                self.output_oscillator.clone(),
                                oscillator,
                            ),
                            Insight::new(timestamp, Some(instrument), self.output_width.clone(), width),
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
