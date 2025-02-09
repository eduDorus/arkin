#![allow(dead_code)]
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use rust_decimal::prelude::*;
use time::OffsetDateTime;

use arkin_core::prelude::*;
use tracing::debug;
use typed_builder::TypedBuilder;
use yata::core::Candle;

#[derive(Debug)]
struct BoundedBuffer {
    data: VecDeque<(OffsetDateTime, Decimal)>,
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

    /// Insert new (CompositeIndex, Decimal). Assume mostly ascending timestamps.
    /// If there's already an item at exactly the same CompositeIndex,
    /// we increment sub_index. Then push_back. If we exceed capacity, pop_front.
    fn push(&mut self, idx: OffsetDateTime, val: Decimal) {
        // Iterate from the back and find the first item whose time <= timestamp.

        let insert_idx = self.data.iter().rev().position(|(i, _)| i <= &idx);
        if let Some(insert_idx) = insert_idx {
            self.data.insert(self.data.len() - insert_idx, (idx, val));
        } else {
            self.data.push_back((idx, val));
        }
        if self.data.len() > self.capacity {
            self.data.pop_front();
        }
    }

    /// Remove any items older than `event_time` (exclusive).
    fn remove_before(&mut self, cutoff: OffsetDateTime) {
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
    fn last_inclusive(&self, timestamp: OffsetDateTime) -> Option<Decimal> {
        for &(idx, val) in self.data.iter().rev() {
            if idx <= timestamp {
                return Some(val);
            }
        }
        None
    }

    /// Return the last value <= timestamp.
    /// That is, from the back, find the first item whose time <= timestamp.
    fn last_exclusive(&self, timestamp: OffsetDateTime) -> Option<Decimal> {
        for &(idx, val) in self.data.iter().rev() {
            if idx < timestamp {
                return Some(val);
            }
        }
        None
    }

    /// Return a window of values in [start_time..end_time).
    /// We can do a quick linear scan. If capacity is only 100k, that might be okay.
    fn window(&self, start_time: OffsetDateTime, end_time: OffsetDateTime) -> Vec<Decimal> {
        // We'll just collect all entries with timestamp in [start..end).
        // If you want to be a bit more efficient, you could break early if you see timestamps >= end_time.
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

    /// Return the last `periods` values up to `timestamp`.
    fn periods(&self, timestamp: OffsetDateTime, periods: usize) -> Vec<Decimal> {
        let mut result = Vec::new();
        // Go from the back and pick up to `periods` items with time <= timestamp
        for &(idx, val) in self.data.iter().rev() {
            if idx <= timestamp {
                result.push(val);
                if result.len() == periods {
                    break;
                }
            }
        }
        // reverse them so they go from oldest to newest
        result.reverse();
        result
    }
}

#[derive(Debug, TypedBuilder)]
pub struct InsightsState {
    #[builder(default)]
    features: DashMap<(Option<Arc<Instrument>>, FeatureId), BoundedBuffer>,
    #[builder(default = 100_000)]
    capacity: usize,
}

impl InsightsState {
    pub fn insert(&self, event: Arc<Insight>) {
        let start = Instant::now();
        let key = (event.instrument.clone(), event.feature_id.clone());
        let mut entry = self.features.entry(key).or_insert_with(|| BoundedBuffer::new(self.capacity));

        entry.push(event.event_time, event.value);

        debug!("Insert took {:?}", start.elapsed());
    }

    pub fn insert_batch(&self, events: &[Arc<Insight>]) {
        let start = Instant::now();
        for e in events {
            self.insert(e.clone());
        }
        debug!("Insert batch took {:?}", start.elapsed());
    }

    pub fn last_candle(&self, instrument: Arc<Instrument>, timestamp: OffsetDateTime) -> Option<Candle> {
        let start = Instant::now();
        let open = self
            .last(Some(instrument.clone()), FeatureId::new("open".into()), timestamp)?
            .to_f64()?;
        let high = self
            .last(Some(instrument.clone()), FeatureId::new("high".into()), timestamp)?
            .to_f64()?;
        let low = self
            .last(Some(instrument.clone()), FeatureId::new("low".into()), timestamp)?
            .to_f64()?;
        let close = self
            .last(Some(instrument.clone()), FeatureId::new("close".into()), timestamp)?
            .to_f64()?;
        let volume = self
            .last(Some(instrument.clone()), FeatureId::new("volume".into()), timestamp)?
            .to_f64()?;

        debug!("Last candle took {:?}", start.elapsed());
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
        timestamp: OffsetDateTime,
    ) -> Option<Decimal> {
        let start = Instant::now();
        let key = (instrument, feature_id);
        let val = self.features.get(&key).and_then(|buf| buf.last_inclusive(timestamp));
        debug!("Last took {:?}", start.elapsed());
        val
    }

    /// Return the last value < timestamp.
    pub fn last_exclusive(
        &self,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        timestamp: OffsetDateTime,
    ) -> Option<Decimal> {
        let start = Instant::now();
        let key = (instrument, feature_id);
        let val = self.features.get(&key).and_then(|buf| buf.last_exclusive(timestamp));
        debug!("Last exclusive took {:?}", start.elapsed());
        val
    }

    /// Return a window of values in [start_time..end_time).
    pub fn window(
        &self,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        timestamp: OffsetDateTime,
        window: Duration,
    ) -> Vec<Decimal> {
        let start = Instant::now();
        let start_time = timestamp - window;
        let key = (instrument, feature_id);
        let vals = self
            .features
            .get(&key)
            .map(|buf| buf.window(start_time, timestamp))
            .unwrap_or_default();
        debug!("Window took {:?}", start.elapsed());
        vals
    }

    /// Return the last `periods` values up to `timestamp`.
    pub fn periods(
        &self,
        instrument: Option<Arc<Instrument>>,
        feature_id: FeatureId,
        timestamp: OffsetDateTime,
        periods: usize,
    ) -> Vec<Decimal> {
        let key = (instrument, feature_id);
        self.features
            .get(&key)
            .map(|buf| buf.periods(timestamp, periods))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use test_log::test;
    use time::OffsetDateTime;

    #[test(test)]
    fn test_insert_and_last() {
        let state = InsightsState::builder().build();
        let now = OffsetDateTime::now_utc();
        let instrument = test_inst_binance_btc_usdt_perp();
        let pipeline = test_pipeline();
        let feature_id = FeatureId::new("test_feature".to_string());

        let t1 = now - Duration::from_secs(10);
        let t2 = now - Duration::from_secs(5);
        let t3 = now; // boundary

        let insight1 = Insight::builder()
            .event_time(t1)
            .pipeline(pipeline.clone())
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .value(dec!(1.1))
            .build();
        let insight2 = Insight::builder()
            .event_time(t2)
            .pipeline(pipeline.clone())
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .value(dec!(1.0))
            .build();
        let insight3 = Insight::builder()
            .event_time(t3)
            .pipeline(pipeline.clone())
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .value(dec!(1.2))
            .build();
        state.insert(insight1.into());
        state.insert(insight3.into());
        state.insert(insight2.into());

        // "last" at time=now should find the inserted value
        let last = state.last(Some(instrument.clone()), feature_id.clone(), now);
        assert_eq!(last, Some(dec!(1.2)));

        let last_exclusive = state.last_exclusive(Some(instrument.clone()), feature_id.clone(), now);
        assert_eq!(last_exclusive, Some(dec!(1.0)));

        let last = state.last(Some(instrument.clone()), feature_id.clone(), now - Duration::from_secs(5));
        assert_eq!(last, Some(dec!(1.0)));

        let last_exclusive =
            state.last_exclusive(Some(instrument.clone()), feature_id.clone(), now - Duration::from_secs(5));
        assert_eq!(last_exclusive, Some(dec!(1.1)));

        let last = state.last(Some(instrument.clone()), feature_id.clone(), now - Duration::from_secs(10));
        assert_eq!(last, Some(dec!(1.1)));

        let last_exclusive =
            state.last_exclusive(Some(instrument.clone()), feature_id.clone(), now - Duration::from_secs(10));
        assert_eq!(last_exclusive, None);

        let last = state.last(Some(instrument), feature_id, now - Duration::from_secs(15));
        assert_eq!(last, None);
    }

    #[test]
    fn test_window_includes_start_excludes_end() {
        let state = InsightsState::builder().build();
        let now = OffsetDateTime::now_utc();
        let instrument = test_inst_binance_btc_usdt_perp();
        let pipeline = test_pipeline();
        let feature_id = FeatureId::new("test_feature".to_string());

        let t1 = now - Duration::from_secs(10);
        let t2 = now - Duration::from_secs(5);
        let t3 = now; // boundary
        let insight1 = Insight::builder()
            .event_time(t1)
            .pipeline(pipeline.clone())
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .value(dec!(1.0))
            .build();
        let insight2 = Insight::builder()
            .event_time(t2)
            .pipeline(pipeline.clone())
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .value(dec!(2.0))
            .build();
        let insight3 = Insight::builder()
            .event_time(t3)
            .pipeline(pipeline.clone())
            .instrument(Some(instrument.clone()))
            .feature_id(feature_id.clone())
            .value(dec!(3.0))
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
        assert_eq!(results, vec![dec!(1.0), dec!(2.0)]);

        // If we want to confirm that t1 is included:
        // results = [1.0, 2.0] => yes, t1 is included.
        // t3 is excluded => no 3.0
    }

    #[test]
    fn test_periods() {
        let state = InsightsState::builder().build();
        let now = OffsetDateTime::now_utc();
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
            let val = Decimal::from_f64(num).unwrap();
            let i = Insight::builder()
                .event_time(*t)
                .pipeline(pipeline.clone())
                .instrument(Some(instrument.clone()))
                .feature_id(feature_id.clone())
                .value(val)
                .build();
            state.insert(i.into());
        }

        // periods(timestamp=now, periods=3)
        // => it should give us the last 3 values before 'now':
        // times => -10s(10), -8s(11), -6s(12), -4s(13), -2s(14)
        // the last 3 => 12, 13, 14
        let p = state.periods(Some(instrument), feature_id, now, 3);
        assert_eq!(p, vec![dec!(3), dec!(4), dec!(5)]);
    }

    #[test]
    fn test_last_candle() {
        let state = InsightsState::builder().build();
        let now = OffsetDateTime::now_utc();
        let instrument = test_inst_binance_btc_usdt_perp();
        let pipeline = test_pipeline();

        // Suppose we have open=1, high=5, low=0, close=3, volume=100
        let open_insight = Insight::builder()
            .event_time(now)
            .pipeline(pipeline.clone())
            .instrument(Some(instrument.clone()))
            .feature_id(FeatureId::new("open".into()))
            .value(dec!(1.0))
            .build();
        let high_insight = Insight::builder()
            .event_time(now)
            .pipeline(pipeline.clone())
            .instrument(Some(instrument.clone()))
            .feature_id(FeatureId::new("high".into()))
            .value(dec!(5.0))
            .build();
        let low_insight = Insight::builder()
            .event_time(now)
            .pipeline(pipeline.clone())
            .instrument(Some(instrument.clone()))
            .feature_id(FeatureId::new("low".into()))
            .value(dec!(0.0))
            .build();
        let close_insight = Insight::builder()
            .event_time(now)
            .pipeline(pipeline.clone())
            .instrument(Some(instrument.clone()))
            .feature_id(FeatureId::new("close".into()))
            .value(dec!(3.0))
            .build();
        let volume_insight = Insight::builder()
            .event_time(now)
            .pipeline(pipeline.clone())
            .instrument(Some(instrument.clone()))
            .feature_id(FeatureId::new("volume".into()))
            .value(dec!(100.0))
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
