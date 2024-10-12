use std::{sync::Arc, time::Duration};

use anyhow::Result;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::OHLCVConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct OHLCVFeature {
    trade_price_input: NodeId,
    trade_quantity_input: NodeId,
    open_output: NodeId,
    high_output: NodeId,
    low_output: NodeId,
    close_output: NodeId,
    volume_output: NodeId,
    notional_volume_output: NodeId,
    window: Duration,
}

impl OHLCVFeature {
    pub fn from_config(config: &OHLCVConfig) -> Self {
        OHLCVFeature {
            trade_price_input: config.input_price.to_owned(),
            trade_quantity_input: config.input_quantity.to_owned(),
            open_output: config.output_open.to_owned(),
            high_output: config.output_high.to_owned(),
            low_output: config.output_low.to_owned(),
            close_output: config.output_close.to_owned(),
            volume_output: config.output_volume.to_owned(),
            notional_volume_output: config.output_notional_volume.to_owned(),
            window: Duration::from_secs(config.window),
        }
    }
}

impl Computation for OHLCVFeature {
    fn inputs(&self) -> Vec<NodeId> {
        vec![self.trade_price_input.clone(), self.trade_quantity_input.clone()]
    }

    fn outputs(&self) -> Vec<NodeId> {
        vec![
            self.open_output.clone(),
            self.high_output.clone(),
            self.low_output.clone(),
            self.close_output.clone(),
        ]
    }

    fn calculate(
        &self,
        instruments: &[Instrument],
        timestamp: &OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating OHLCV");

        // Get data from state
        let price_data = state.get_window_by_instruments(instruments, &self.trade_price_input, timestamp, &self.window);
        let quantity_data =
            state.get_window_by_instruments(instruments, &self.trade_quantity_input, timestamp, &self.window);

        // Calculate the mean (OHLC)
        let insights = instruments
            .iter()
            .filter_map(|instrument| {
                let prices = price_data.get(instrument).cloned().unwrap_or(vec![]);
                let quantities = quantity_data.get(instrument).cloned().unwrap_or(vec![]);

                // Check if we have enough data
                if prices.is_empty() || quantities.is_empty() || prices.len() != quantities.len() {
                    warn!("Not enough data for OHLC calculation");
                    return None;
                }

                // Calculate OHLC
                let open = prices.first().expect("Should have at least one value");
                let high = prices.iter().max().expect("Should have at least one value");
                let low = prices.iter().min().expect("Should have at least one value");
                let close = prices.last().expect("Should have at least one value");
                let volume = quantities.iter().sum::<Decimal>();
                let notional_volume = prices
                    .iter()
                    .zip(quantities.iter())
                    .map(|(price, quantity)| price * quantity)
                    .sum::<Decimal>();

                // Create insights
                let open_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.open_output.clone(),
                    open.clone(),
                );
                let high_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.high_output.clone(),
                    high.clone(),
                );
                let low_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.low_output.clone(),
                    low.clone(),
                );
                let close_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.close_output.clone(),
                    close.clone(),
                );
                let volume_insight =
                    Insight::new(timestamp.clone(), Some(instrument.clone()), self.volume_output.clone(), volume);

                let notional_volume_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.notional_volume_output.clone(),
                    notional_volume,
                );

                Some(vec![
                    open_insight,
                    high_insight,
                    low_insight,
                    close_insight,
                    volume_insight,
                    notional_volume_insight,
                ])
            })
            .flatten()
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
