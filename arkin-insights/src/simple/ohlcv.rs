use std::{sync::Arc, time::Duration};

use anyhow::Result;
use rayon::prelude::*;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{state::InsightsState, Computation};

#[derive(Debug, Clone, TypedBuilder)]
pub struct OHLCVFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
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

    fn calculate(&self, instruments: &[Arc<Instrument>], event_time: OffsetDateTime) -> Result<Vec<Arc<Insight>>> {
        debug!("Calculating OHLCV");

        // Calculate the mean (OHLC)
        let insights = instruments
            .par_iter()
            .filter_map(|instrument| {
                // Get data
                let prices = self.insight_state.window(
                    Some(instrument.clone()),
                    self.input_price.clone(),
                    event_time,
                    self.window,
                );
                let quantities = self.insight_state.window(
                    Some(instrument.clone()),
                    self.input_quantity.clone(),
                    event_time,
                    self.window,
                );

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

                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_open.clone())
                        .value(open)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_high.clone())
                        .value(high)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_low.clone())
                        .value(low)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_close.clone())
                        .value(close)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_typical_price.clone())
                        .value(typical_price)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_vwap.clone())
                        .value(vwap)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_volume.clone())
                        .value(volume)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_buy_volume.clone())
                        .value(buy_volume)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_sell_volume.clone())
                        .value(sell_volume)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_notional_volume.clone())
                        .value(notional_volume)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_buy_notional_volume.clone())
                        .value(buy_notional_volume)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_sell_notional_volume.clone())
                        .value(sell_notional_volume)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_trade_count.clone())
                        .value(trade_count)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_buy_trade_count.clone())
                        .value(buy_trade_count)
                        .build()
                        .into(),
                );
                insights.push(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output_sell_trade_count.clone())
                        .value(sell_trade_count)
                        .build()
                        .into(),
                );
                Some(insights)
            })
            .flatten()
            .collect::<Vec<_>>();

        self.insight_state.insert_batch(&insights);
        Ok(insights)
    }
}
