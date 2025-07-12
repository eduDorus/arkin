use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use time::UtcDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{state::InsightsState, Feature};

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
    persist: bool,
}

#[async_trait]
impl Feature for OHLCVFeature {
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

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: UtcDateTime) -> Option<Vec<Insight>> {
        debug!("Calculating OHLCV");

        // Get data
        let prices =
            self.insight_state
                .window(Some(instrument.clone()), self.input_price.clone(), event_time, self.window);
        let quantities =
            self.insight_state
                .window(Some(instrument.clone()), self.input_quantity.clone(), event_time, self.window);

        // Check if we have enough data
        if prices.is_empty() || quantities.is_empty() || prices.len() != quantities.len() {
            warn!("Not enough data for OHLC calculation");
            return None;
        }

        // Calculate OHLC
        let open = prices.first().expect("Should have at least one value").to_owned();
        let high = prices.iter().max_by(|a, b| a.total_cmp(b))?.to_owned();
        let low = prices.iter().min_by(|a, b| a.total_cmp(b))?.to_owned();
        let close = prices.last().expect("Should have at least one value").to_owned();
        let typical_price = (high + low + close) / 3.;

        // Calculate volume
        let (volume, buy_volume, sell_volume) =
            quantities
                .iter()
                .fold((0., 0., 0.), |(volume, buy_volume, sell_volume), quantity| {
                    if quantity >= &0. {
                        (volume + quantity, buy_volume + quantity, sell_volume)
                    } else {
                        (volume + quantity.abs(), buy_volume, sell_volume + quantity.abs())
                    }
                });
        debug!("Volume: {}, Buy Volume: {}, Sell Volume: {}", volume, buy_volume, sell_volume);

        // Calculate notional volume
        let (notional_volume, buy_notional_volume, sell_notional_volume) = prices.iter().zip(quantities.iter()).fold(
            (0., 0., 0.),
            |(notional_volume, notional_buy_volume, notional_sell_volume), (price, quantity)| {
                if quantity >= &0. {
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
        debug!(
            "Notional Volume: {}, Buy Notional Volume: {}, Sell Notional Volume: {}",
            notional_volume, buy_notional_volume, sell_notional_volume
        );

        // Calculate VWAP
        let vwap = notional_volume / volume;
        debug!("VWAP: {}", vwap);

        // Calculate trade count
        let (trade_count, buy_trade_count, sell_trade_count) =
            quantities
                .iter()
                .fold((0, 0, 0), |(trade_count, buy_trade_count, sell_trade_count), quantity| {
                    if quantity >= &0. {
                        (trade_count + 1, buy_trade_count + 1, sell_trade_count)
                    } else {
                        (trade_count + 1, buy_trade_count, sell_trade_count + 1)
                    }
                });
        debug!(
            "Trade Count: {}, Buy Trade Count: {}, Sell Trade Count: {}",
            trade_count, buy_trade_count, sell_trade_count
        );

        // Create insights
        let mut insights = Vec::with_capacity(self.outputs().len());

        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_open.clone())
                .value(open)
                .insight_type(InsightType::Ohlcv)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_high.clone())
                .value(high)
                .insight_type(InsightType::Ohlcv)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_low.clone())
                .value(low)
                .insight_type(InsightType::Ohlcv)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_close.clone())
                .value(close)
                .insight_type(InsightType::Ohlcv)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_typical_price.clone())
                .value(typical_price)
                .insight_type(InsightType::Price)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_vwap.clone())
                .value(vwap)
                .insight_type(InsightType::Price)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_volume.clone())
                .value(volume)
                .insight_type(InsightType::Continuous)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_buy_volume.clone())
                .value(buy_volume)
                .insight_type(InsightType::Continuous)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_sell_volume.clone())
                .value(sell_volume)
                .insight_type(InsightType::Continuous)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_notional_volume.clone())
                .value(notional_volume)
                .insight_type(InsightType::Continuous)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_buy_notional_volume.clone())
                .value(buy_notional_volume)
                .insight_type(InsightType::Continuous)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_sell_notional_volume.clone())
                .value(sell_notional_volume)
                .insight_type(InsightType::Continuous)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_trade_count.clone())
                .value(trade_count as f64)
                .insight_type(InsightType::Continuous)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_buy_trade_count.clone())
                .value(buy_trade_count as f64)
                .insight_type(InsightType::Continuous)
                .persist(self.persist)
                .build(),
        );
        insights.push(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output_sell_trade_count.clone())
                .value(sell_trade_count as f64)
                .insight_type(InsightType::Continuous)
                .persist(self.persist)
                .build(),
        );

        Some(insights)
    }

    async fn async_calculate(&self, instrument: &Arc<Instrument>, timestamp: UtcDateTime) -> Option<Vec<Insight>> {
        self.calculate(instrument, timestamp)
    }
}
