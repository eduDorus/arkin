use std::{sync::Arc, time::Duration};

use anyhow::Result;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::OHLCVConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct OHLCVFeature {
    input_price: FeatureId,
    input_quantity: FeatureId,
    output_open: FeatureId,
    output_high: FeatureId,
    output_low: FeatureId,
    output_close: FeatureId,
    output_typical_price: FeatureId,
    output_vwap: FeatureId,
    output_volume: FeatureId,
    output_buy_volume: FeatureId,
    output_sell_volume: FeatureId,
    output_notional_volume: FeatureId,
    output_buy_notional_volume: FeatureId,
    output_sell_notional_volume: FeatureId,
    output_trade_count: FeatureId,
    output_buy_trade_count: FeatureId,
    output_sell_trade_count: FeatureId,
    window: Duration,
}

impl OHLCVFeature {
    pub fn from_config(config: &OHLCVConfig) -> Self {
        OHLCVFeature {
            input_price: config.input_price.to_owned(),
            input_quantity: config.input_quantity.to_owned(),
            output_open: config.output_open.to_owned(),
            output_high: config.output_high.to_owned(),
            output_low: config.output_low.to_owned(),
            output_close: config.output_close.to_owned(),
            output_typical_price: config.output_typical_price.to_owned(),
            output_vwap: config.output_vwap.to_owned(),
            output_volume: config.output_volume.to_owned(),
            output_buy_volume: config.output_buy_volume.to_owned(),
            output_sell_volume: config.output_sell_volume.to_owned(),
            output_notional_volume: config.output_notional_volume.to_owned(),
            output_buy_notional_volume: config.output_buy_notional_volume.to_owned(),
            output_sell_notional_volume: config.output_sell_notional_volume.to_owned(),
            output_trade_count: config.output_trade_count.to_owned(),
            output_buy_trade_count: config.output_buy_trade_count.to_owned(),
            output_sell_trade_count: config.output_sell_trade_count.to_owned(),
            window: Duration::from_secs(config.window),
        }
    }
}

impl Computation for OHLCVFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_price.clone(), self.input_quantity.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![
            self.output_open.clone(),
            self.output_high.clone(),
            self.output_low.clone(),
            self.output_close.clone(),
            self.output_typical_price.clone(),
            self.output_vwap.clone(),
            self.output_volume.clone(),
            self.output_buy_volume.clone(),
            self.output_sell_volume.clone(),
            self.output_notional_volume.clone(),
            self.output_buy_notional_volume.clone(),
            self.output_sell_notional_volume.clone(),
            self.output_trade_count.clone(),
            self.output_buy_trade_count.clone(),
            self.output_sell_trade_count.clone(),
        ]
    }

    fn calculate(
        &self,
        instruments: &[Arc<Instrument>],
        event_time: OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating OHLCV");

        // Get data from state

        // Calculate the mean (OHLC)
        let insights = instruments
            .iter()
            .filter_map(|instrument| {
                // Get data
                let prices = state.window(Some(instrument.clone()), self.input_price.clone(), event_time, self.window);
                let quantities =
                    state.window(Some(instrument.clone()), self.input_quantity.clone(), event_time, self.window);

                // Check if we have enough data
                if prices.is_empty() || quantities.is_empty() || prices.len() != quantities.len() {
                    warn!("Not enough data for OHLC calculation");
                    return None;
                }

                // Calculate OHLC
                let open = prices.first().expect("Should have at least one value").to_owned();
                let high = prices.iter().max().expect("Should have at least one value").to_owned();
                let low = prices.iter().min().expect("Should have at least one value").to_owned();
                let close = prices.last().expect("Should have at least one value").to_owned();
                let typical_price = (high + low + close) / Decimal::from(3);

                // Calculate volume
                let (volume, buy_volume, sell_volume) = quantities.iter().fold(
                    (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
                    |(volume, buy_volume, sell_volume), quantity| {
                        if quantity > &Decimal::ZERO {
                            (volume + quantity, buy_volume + quantity, sell_volume)
                        } else {
                            (volume + quantity.abs(), buy_volume, sell_volume + quantity.abs())
                        }
                    },
                );

                // Calculate notional volume
                let (notional_volume, buy_notional_volume, sell_notional_volume) =
                    prices.iter().zip(quantities.iter()).fold(
                        (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
                        |(notional_volume, notional_buy_volume, notional_sell_volume), (price, quantity)| {
                            if quantity > &Decimal::ZERO {
                                (
                                    notional_volume + price * quantity,
                                    notional_buy_volume + price * quantity,
                                    notional_sell_volume,
                                )
                            } else {
                                (
                                    notional_volume + price * quantity.abs(),
                                    notional_buy_volume,
                                    notional_sell_volume + price * quantity.abs(),
                                )
                            }
                        },
                    );

                // Calculate VWAP
                let vwap = notional_volume / volume;

                // Calculate trade count
                let (trade_count, buy_trade_count, sell_trade_count) = quantities.iter().fold(
                    (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
                    |(trade_count, buy_trade_count, sell_trade_count), quantity| {
                        if quantity > &Decimal::ZERO {
                            (trade_count + Decimal::ONE, buy_trade_count + Decimal::ONE, sell_trade_count)
                        } else {
                            (trade_count + Decimal::ONE, buy_trade_count, sell_trade_count + Decimal::ONE)
                        }
                    },
                );

                // Create insights
                let mut insights = Vec::with_capacity(self.outputs().len());

                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_open.clone(),
                    open,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_high.clone(),
                    high,
                ));
                insights.push(Insight::new(event_time, Some(instrument.clone()), self.output_low.clone(), low));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_close.clone(),
                    close,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_typical_price.clone(),
                    typical_price,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_vwap.clone(),
                    vwap,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_volume.clone(),
                    volume,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_buy_volume.clone(),
                    buy_volume,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_sell_volume.clone(),
                    sell_volume,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_notional_volume.clone(),
                    notional_volume,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_buy_notional_volume.clone(),
                    buy_notional_volume,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_sell_notional_volume.clone(),
                    sell_notional_volume,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_trade_count.clone(),
                    trade_count,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_buy_trade_count.clone(),
                    buy_trade_count,
                ));
                insights.push(Insight::new(
                    event_time,
                    Some(instrument.clone()),
                    self.output_sell_trade_count.clone(),
                    sell_trade_count,
                ));
                Some(insights)
            })
            .flatten()
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
