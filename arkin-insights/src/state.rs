#![allow(dead_code)]
use std::{collections::VecDeque, sync::Arc, time::Duration};

use dashmap::DashMap;
use time::UtcDateTime;

use arkin_core::prelude::*;
use typed_builder::TypedBuilder;
use yata::core::Candle;

#[derive(Debug)]
struct BoundedBuffer {
    data: VecDeque<(UtcDateTime, f64)>,
    capacity: usize,
}

impl BoundedBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity.min(1024)),
            capacity,
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    /// Insert new (CompositeIndex, f64). Assume mostly ascending timestamps.
    /// If there's already an item at exactly the same CompositeIndex,
    /// we increment sub_index. Then push_back. If we exceed capacity, pop_front.
    fn push(&mut self, idx: UtcDateTime, val: f64) {
        if let Some(last) = self.data.back() {
            if idx >= last.0 {
                self.data.push_back((idx, val));
            } else {
                let insert_idx = self.data.iter().rev().position(|(i, _)| i <= &idx);
                if let Some(insert_idx) = insert_idx {
                    self.data.insert(self.data.len() - insert_idx, (idx, val));
                } else {
                    self.data.push_front((idx, val));
                }
            }
        } else {
            self.data.push_back((idx, val));
        }
        if self.data.len() > self.capacity {
            self.data.pop_front();
        }
    }

    /// Remove any items older than `event_time` (exclusive).
    fn remove_before(&mut self, cutoff: UtcDateTime) {
        while let Some(front) = self.data.front() {
            if front.0 < cutoff {
                self.data.pop_front();
            } else {
                break;
            }
        }
    }

    /// Return the last value <= timestamp.
    /// That is, from the back, find the first item whose time <= timestamp.
    fn last_inclusive(&self, timestamp: UtcDateTime) -> Option<f64> {
        for &(idx, val) in self.data.iter().rev() {
            if idx <= timestamp {
                return Some(val);
            }
        }
        None
    }

    /// Return the last value <= timestamp.
    /// That is, from the back, find the first item whose time <= timestamp.
    fn last_exclusive(&self, timestamp: UtcDateTime) -> Option<f64> {
        for &(idx, val) in self.data.iter().rev() {
            if idx < timestamp {
                return Some(val);
            }
        }
        None
    }

    /// Return a window of values in [start_time=..end_time).
    fn window(&self, start_time: UtcDateTime, end_time: UtcDateTime) -> Vec<f64> {
        let mut result = Vec::new();
        for &(idx, val) in self.data.iter().rev() {
            if idx >= start_time && idx < end_time {
                result.push(val);
            }
            if idx < start_time {
                break;
            }
        }
        // reverse them so they go from oldest to newest
        result.reverse();
        result
    }

    /// Return the last `intervals` values up to `timestamp`.
    fn intervals(&self, timestamp: UtcDateTime, intervals: usize) -> Vec<f64> {
        let mut result = Vec::new();
        // Go from the back and pick up to `periods` items with time <= timestamp
        for &(idx, val) in self.data.iter().rev() {
            if idx <= timestamp {
                result.push(val);
                if result.len() == intervals {
                    break;
                }
            }
        }
        // reverse them so they go from oldest to newest
        result.reverse();
        result
    }

    /// Returns the last value + lag periods.
    fn lag(&self, timestamp: UtcDateTime, lag: usize) -> Option<f64> {
        let mut count = 0;
        for &(idx, val) in self.data.iter().rev() {
            if idx <= timestamp {
                if count == lag {
                    return Some(val);
                }
                count += 1;
            }
        }
        None
    }
}

#[derive(Debug, TypedBuilder)]
pub struct InsightsState {
    #[builder(default)]
    features: DashMap<(Option<Arc<Instrument>>, FeatureId), BoundedBuffer>,
    #[builder(default = 1_000_000)]
    capacity: usize,
}

impl InsightsState {
    pub fn insert(&self, event: Arc<Insight>) {
        let key = (event.instrument.clone(), event.feature_id.clone());
        let mut entry = self.features.entry(key).or_insert_with(|| BoundedBuffer::new(self.capacity));
        entry.push(event.event_time, event.value);
    }

    pub fn insert_batch(&self, events: &[Arc<Insight>]) {
        // Filter out InsightTypes Transformed and Scaled
        // TODO: Probably need a better solution than this.
        let events: Vec<_> = events
            .iter()
            .filter(|event| event.insight_type != InsightType::Normalized)
            .cloned()
            .collect();
        for event in events {
            let key = (event.instrument.clone(), event.feature_id.clone());
            let mut entry = self.features.entry(key).or_insert_with(|| BoundedBuffer::new(self.capacity));
            entry.push(event.event_time, event.value);
        }
    }

    pub fn last_candle(&self, instrument: Arc<Instrument>, timestamp: UtcDateTime) -> Option<Candle> {
        let open = self.last(Some(instrument.clone()), FeatureId::new("open".into()), timestamp)?;
        let high = self.last(Some(instrument.clone()), FeatureId::new("high".into()), timestamp)?;
        let low = self.last(Some(instrument.clone()), FeatureId::new("low".into()), timestamp)?;
        let close = self.last(Some(instrument.clone()), FeatureId::new("close".into()), timestamp)?;
        let volume = self.last(Some(instrument.clone()), FeatureId::new("volume".into()), timestamp)?;

        Some(Candle {
            open,
            high,
            low,
            close,
            volume,
        })
    }

    /// Return the last value <= timestamp.
    pub fn last(
        &self,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        timestamp: UtcDateTime,
    ) -> Option<f64> {
        let key = (instrument, feature_id);
        let val = self.features.get(&key).and_then(|buf| buf.last_inclusive(timestamp));
        val
    }

    /// Return a last value <= timestamp.
    pub fn lag(
        &self,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        timestamp: UtcDateTime,
        lag: usize,
    ) -> Option<f64> {
        let key = (instrument, feature_id);
        let val = self.features.get(&key).and_then(|buf| buf.lag(timestamp, lag));
        val
    }

    /// Return the last value < timestamp.
    pub fn last_exclusive(
        &self,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        timestamp: UtcDateTime,
    ) -> Option<f64> {
        let key = (instrument, feature_id);
        let val = self.features.get(&key).and_then(|buf| buf.last_exclusive(timestamp));
        val
    }

    /// Return a window of values in [start_time..end_time).
    pub fn window(
        &self,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        timestamp: UtcDateTime,
        window: Duration,
    ) -> Vec<f64> {
        let start_time = timestamp - window;
        let key = (instrument, feature_id);
        let vals = self
            .features
            .get(&key)
            .map(|buf| buf.window(start_time, timestamp))
            .unwrap_or_default();
        vals
    }

    /// Return the last `intervals` values up to `timestamp`.
    pub fn intervals(
        &self,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        timestamp: UtcDateTime,
        intervals: usize,
    ) -> Vec<f64> {
        let key = (instrument, feature_id);
        self.features
            .get(&key)
            .map(|buf| buf.intervals(timestamp, intervals))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;
    use time::UtcDateTime;

    #[test]
    fn test_push_out_of_order() {
        let mut buffer = BoundedBuffer::new(5);
        let now = UtcDateTime::now();
        buffer.push(now, 1.0);
        buffer.push(now - Duration::from_secs(1), 0.5); // older
        buffer.push(now + Duration::from_secs(1), 1.5); // newer
        buffer.push(now - Duration::from_secs(2), 0.0); // oldest
        buffer.push(now + Duration::from_secs(2), 2.0); // newest

        let times: Vec<_> = buffer.data.iter().map(|(t, _)| *t).collect();
        assert_eq!(
            times,
            vec![
                now - Duration::from_secs(2),
                now - Duration::from_secs(1),
                now,
                now + Duration::from_secs(1),
                now + Duration::from_secs(2),
            ]
        );
        let values: Vec<_> = buffer.data.iter().map(|(_, v)| *v).collect();
        assert_eq!(values, vec![0.0, 0.5, 1.0, 1.5, 2.0]);
    }

    #[test]
    fn test_capacity_limit() {
        let mut buffer = BoundedBuffer::new(3);
        let now = UtcDateTime::now();
        buffer.push(now - Duration::from_secs(3), 0.0);
        buffer.push(now - Duration::from_secs(2), 1.0);
        buffer.push(now - Duration::from_secs(1), 2.0);
        buffer.push(now, 3.0); // Exceeds capacity

        let values: Vec<_> = buffer.data.iter().map(|(_, v)| *v).collect();
        assert_eq!(values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_last_inclusive() {
        let mut buffer = BoundedBuffer::new(5);
        let now = UtcDateTime::now();
        buffer.push(now - Duration::from_secs(2), 0.0);
        buffer.push(now - Duration::from_secs(1), 1.0);
        buffer.push(now, 2.0);

        assert_eq!(buffer.last_inclusive(now), Some(2.0));
        assert_eq!(buffer.last_inclusive(now - Duration::from_secs(1)), Some(1.0));
        assert_eq!(buffer.last_inclusive(now - Duration::from_secs(3)), None);
    }

    #[test]
    fn test_window() {
        let mut buffer = BoundedBuffer::new(5);
        let now = UtcDateTime::now();
        buffer.push(now - Duration::from_secs(4), 0.0);
        buffer.push(now - Duration::from_secs(3), 1.0);
        buffer.push(now - Duration::from_secs(2), 2.0);
        buffer.push(now - Duration::from_secs(1), 3.0);
        buffer.push(now, 4.0);

        let window = buffer.window(now - Duration::from_secs(3), now - Duration::from_secs(1));
        assert_eq!(window, vec![1.0, 2.0]);
    }

    #[test]
    fn test_intervals() {
        let mut buffer = BoundedBuffer::new(5);
        let now = UtcDateTime::now();
        buffer.push(now - Duration::from_secs(4), 0.0);
        buffer.push(now - Duration::from_secs(3), 1.0);
        buffer.push(now - Duration::from_secs(2), 2.0);
        buffer.push(now - Duration::from_secs(1), 3.0);
        buffer.push(now, 4.0);

        let intervals = buffer.intervals(now, 3);
        assert_eq!(intervals, vec![2.0, 3.0, 4.0]);
        let past_intervals = buffer.intervals(now - Duration::from_secs(2), 2);
        assert_eq!(past_intervals, vec![1.0, 2.0]);
    }

    #[test]
    fn test_lag() {
        let mut buffer = BoundedBuffer::new(5);
        let now = UtcDateTime::now();
        buffer.push(now - Duration::from_secs(4), 0.0);
        buffer.push(now - Duration::from_secs(3), 1.0);
        buffer.push(now - Duration::from_secs(2), 2.0);
        buffer.push(now - Duration::from_secs(1), 3.0);
        buffer.push(now, 4.0);

        assert_eq!(buffer.lag(now, 0), Some(4.0));
        assert_eq!(buffer.lag(now, 1), Some(3.0));
        assert_eq!(buffer.lag(now - Duration::from_secs(2), 0), Some(2.0));
        assert_eq!(buffer.lag(now - Duration::from_secs(2), 1), Some(1.0));
        assert_eq!(buffer.lag(now, 5), None);
    }

    #[test(test)]
    fn test_insert_and_last() {
        let state = InsightsState::builder().build();
        let now = UtcDateTime::now();
        let instrument = test_inst_binance_btc_usdt_perp();
        let pipeline = test_pipeline();
        let feature_id = FeatureId::new("test_feature".to_string());

        let t1 = now - Duration::from_secs(10);
        let t2 = now - Duration::from_secs(5);
        let t3 = now; // boundary

        let insight1 = Insight::builder()
            .event_time(t1)
            .pipeline(Some(pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .value(1.1)
            .insight_type(InsightType::Raw)
            .build();
        let insight2 = Insight::builder()
            .event_time(t2)
            .pipeline(Some(pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .insight_type(InsightType::Raw)
            .value(1.0)
            .build();
        let insight3 = Insight::builder()
            .event_time(t3)
            .pipeline(Some(pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .insight_type(InsightType::Raw)
            .value(1.2)
            .build();
        state.insert(insight1.into());
        state.insert(insight3.into());
        state.insert(insight2.into());

        // "last" at time=now should find the inserted value
        let last = state.last(Some(instrument.clone()), feature_id.clone(), now);
        assert_eq!(last, Some(1.2));

        let last_exclusive = state.last_exclusive(Some(instrument.clone()), feature_id.clone(), now);
        assert_eq!(last_exclusive, Some(1.0));

        let last = state.last(Some(instrument.clone()), feature_id.clone(), now - Duration::from_secs(5));
        assert_eq!(last, Some(1.0));

        let last_exclusive =
            state.last_exclusive(Some(instrument.clone()), feature_id.clone(), now - Duration::from_secs(5));
        assert_eq!(last_exclusive, Some(1.1));

        let last = state.last(Some(instrument.clone()), feature_id.clone(), now - Duration::from_secs(10));
        assert_eq!(last, Some(1.1));

        let last_exclusive =
            state.last_exclusive(Some(instrument.clone()), feature_id.clone(), now - Duration::from_secs(10));
        assert_eq!(last_exclusive, None);

        let last = state.last(Some(instrument), feature_id, now - Duration::from_secs(15));
        assert_eq!(last, None);
    }

    #[test]
    fn test_window_includes_start_excludes_end() {
        let state = InsightsState::builder().build();
        let now = UtcDateTime::now();
        let instrument = test_inst_binance_btc_usdt_perp();
        let pipeline = test_pipeline();
        let feature_id = FeatureId::new("test_feature".to_string());

        let t1 = now - Duration::from_secs(10);
        let t2 = now - Duration::from_secs(5);
        let t3 = now; // boundary
        let insight1 = Insight::builder()
            .event_time(t1)
            .pipeline(Some(pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .value(1.0)
            .insight_type(InsightType::Raw)
            .build();
        let insight2 = Insight::builder()
            .event_time(t2)
            .pipeline(Some(pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .value(2.0)
            .insight_type(InsightType::Raw)
            .build();
        let insight3 = Insight::builder()
            .event_time(t3)
            .pipeline(Some(pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .value(3.0)
            .insight_type(InsightType::Raw)
            .build();

        state.insert(insight1.into());
        state.insert(insight2.into());
        state.insert(insight3.into());

        // Query window from t1 to t3:
        //   - Should include events at t1 and t2
        //   - Should EXCLUDE event exactly at t3 if the code uses range(t1..t3)
        let duration = Duration::from_secs(10);
        let results = state.window(Some(instrument), feature_id, t3, duration);
        assert_eq!(results.len(), 2);
        assert_eq!(results, vec![1.0, 2.0]);

        // If we want to confirm that t1 is included:
        // results = [1.0, 2.0] => yes, t1 is included.
        // t3 is excluded => no 3.0
    }

    #[test]
    fn test_periods() {
        let state = InsightsState::builder().build();
        let now = UtcDateTime::now();
        let instrument = test_inst_binance_btc_usdt_perp();
        let pipeline = test_pipeline();
        let feature_id = FeatureId::new("test_feature".to_string());

        // Insert multiple points in ascending order
        let times = [
            now - Duration::from_secs(10),
            now - Duration::from_secs(8),
            now - Duration::from_secs(6),
            now - Duration::from_secs(4),
            now - Duration::from_secs(2),
            now,
        ];
        for (idx, t) in times.iter().enumerate() {
            let num = idx as f64;
            let i = Insight::builder()
                .event_time(*t)
                .pipeline(Some(pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(feature_id.clone())
                .value(num)
                .insight_type(InsightType::Raw)
                .build();
            state.insert(i.into());
        }

        // periods(timestamp=now, periods=3)
        // => it should give us the last 3 values before 'now':
        // times => -10s(10), -8s(11), -6s(12), -4s(13), -2s(14)
        // the last 3 => 12, 13, 14
        let p = state.intervals(Some(instrument), feature_id, now, 3);
        assert_eq!(p, vec![3., 4., 5.]);
    }

    #[test]
    fn test_last_candle() {
        let state = InsightsState::builder().build();
        let now = UtcDateTime::now();
        let instrument = test_inst_binance_btc_usdt_perp();
        let pipeline = test_pipeline();

        // Suppose we have open=1, high=5, low=0, close=3, volume=100
        let open_insight = Insight::builder()
            .event_time(now)
            .pipeline(Some(pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(FeatureId::new("open".into()))
            .value(1.0)
            .insight_type(InsightType::Ohlcv)
            .build();
        let high_insight = Insight::builder()
            .event_time(now)
            .pipeline(Some(pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(FeatureId::new("high".into()))
            .value(5.0)
            .insight_type(InsightType::Ohlcv)
            .build();
        let low_insight = Insight::builder()
            .event_time(now)
            .pipeline(Some(pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(FeatureId::new("low".into()))
            .value(0.0)
            .insight_type(InsightType::Ohlcv)
            .build();
        let close_insight = Insight::builder()
            .event_time(now)
            .pipeline(Some(pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(FeatureId::new("close".into()))
            .value(3.0)
            .insight_type(InsightType::Ohlcv)
            .build();
        let volume_insight = Insight::builder()
            .event_time(now)
            .pipeline(Some(pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(FeatureId::new("volume".into()))
            .value(100.)
            .insight_type(InsightType::Ohlcv)
            .build();

        state.insert(open_insight.into());
        state.insert(high_insight.into());
        state.insert(low_insight.into());
        state.insert(close_insight.into());
        state.insert(volume_insight.into());

        let candle = state.last_candle(instrument.clone(), now).expect("Expected a candle");
        assert_eq!(candle.open, 1.0);
        assert_eq!(candle.high, 5.0);
        assert_eq!(candle.low, 0.0);
        assert_eq!(candle.close, 3.0);
        assert_eq!(candle.volume, 100.0);
    }
}
