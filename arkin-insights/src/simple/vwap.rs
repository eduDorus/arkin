use std::{sync::Arc, time::Duration};

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::{debug, warn};

use crate::{config::VWAPConfig, state::InsightsState, Computation};

#[derive(Debug)]
pub struct VWAPFeature {
    input_price: FeatureId,
    input_quantity: FeatureId,
    output: FeatureId,
    window: Duration,
}

impl VWAPFeature {
    pub fn from_config(config: &VWAPConfig) -> Self {
        VWAPFeature {
            input_price: config.input_price.to_owned(),
            input_quantity: config.input_quantity.to_owned(),
            output: config.output.to_owned(),
            window: Duration::from_secs(config.window),
        }
    }
}

impl Computation for VWAPFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_price.clone(), self.input_quantity.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(
        &self,
        instruments: &[Arc<Instrument>],
        timestamp: OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating VWAP feature");

        let insights = instruments
            .iter()
            .cloned()
            .filter_map(|instrument| {
                let prices = state.window(Some(instrument.clone()), self.input_price.clone(), timestamp, self.window);
                let quantities =
                    state.window(Some(instrument.clone()), self.input_quantity.clone(), timestamp, self.window);

                if prices.is_empty() {
                    warn!("No price data found for instrument {}", instrument);
                    return None;
                }

                if quantities.is_empty() {
                    warn!("No quantity data found for instrument {}", instrument);
                    return None;
                }

                if prices.len() != quantities.len() {
                    warn!("Price and volume data have different lengths");
                    return None;
                }

                let (total_price_volume, total_volume) = prices.iter().zip(quantities.iter()).fold(
                    (Decimal::ZERO, Decimal::ZERO),
                    |(acc_price_volume, acc_quantity), (price, quantity)| {
                        (acc_price_volume + (price * quantity.abs()), acc_quantity + quantity.abs())
                    },
                );

                let vwap = if total_volume.is_zero() {
                    Decimal::ZERO
                } else {
                    total_price_volume / total_volume
                };

                Some(Insight::new(timestamp, Some(instrument), self.output.clone(), vwap))
            })
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
