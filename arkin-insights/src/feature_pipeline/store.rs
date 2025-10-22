#![allow(dead_code)]
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use dashmap::DashMap;
use rayon::prelude::*;
use thiserror::Error;
use time::UtcDateTime;

use arkin_core::prelude::*;
use tokio::sync::Mutex;
use tracing::debug;
use typed_builder::TypedBuilder;

use crate::FillStrategy;

#[derive(Debug, Error)]
pub enum StateQueryError {
    #[error("No buffer found for instrument {instrument} and feature {feature}")]
    NoBuffer { instrument: String, feature: String },

    #[error("Buffer exists but contains no data")]
    NoData,

    #[error("Insufficient data: requested {requested}, available {available}")]
    InsufficientData { requested: usize, available: usize },

    #[error("Missing data at timestamp {timestamp} and Drop strategy specified")]
    MissingData { timestamp: UtcDateTime },
}

/// Apply fill strategy for a missing timestamp
fn apply_fill_strategy(
    buffer: &BoundedBuffer,
    missing_timestamp: UtcDateTime,
    strategy: FillStrategy,
) -> Result<f64, StateQueryError> {
    match strategy {
        FillStrategy::ForwardFill => {
            // Get last value before missing timestamp
            buffer
                .last(missing_timestamp - Duration::from_millis(1))
                .ok_or(StateQueryError::NoData)
        }
        FillStrategy::Zero => Ok(0.0),
        FillStrategy::Drop => Err(StateQueryError::MissingData {
            timestamp: missing_timestamp,
        }),
    }
}

#[derive(Debug)]
pub struct BoundedBuffer {
    data: VecDeque<(UtcDateTime, f64)>,
}

impl BoundedBuffer {
    fn new() -> Self {
        Self {
            data: VecDeque::new(),
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

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
    }

    fn extend_sorted(&mut self, values: &[(UtcDateTime, f64)]) {
        if values.is_empty() {
            return;
        }

        // Fast path: buffer is empty, just extend
        if self.data.is_empty() {
            self.data.extend(values.iter().copied());
            return;
        }

        // Fast path: all new values come after existing data
        if let Some(last_existing) = self.data.back()
            && values[0].0 >= last_existing.0
        {
            self.data.extend(values.iter().copied());
            return;
        }

        // Optimized path: find split point where values can be bulk-appended
        // Push individual values until we reach the point where remaining values
        // are all newer than the last existing value
        let mut split_idx = 0;

        for (idx, &(timestamp, value)) in values.iter().enumerate() {
            if let Some(last_existing) = self.data.back()
                && timestamp >= last_existing.0
            {
                // Found the split point - remaining values can be bulk-appended
                split_idx = idx;
                break;
            }
            // This value needs individual insertion
            self.push(timestamp, value);
            split_idx = idx + 1;
        }

        // Bulk-append remaining values (fast path for the tail)
        if split_idx < values.len() {
            self.data.extend(values[split_idx..].iter().copied());
        }
    }

    fn remove_before(&mut self, cutoff: UtcDateTime) {
        while let Some(front) = self.data.front() {
            if front.0 < cutoff {
                self.data.pop_front();
            } else {
                break;
            }
        }
    }

    fn last(&self, timestamp: UtcDateTime) -> Option<f64> {
        for &(idx, val) in self.data.iter().rev() {
            if idx <= timestamp {
                return Some(val);
            }
        }
        None
    }

    /// Get value at exact timestamp. Returns None if not found.
    fn value_at(&self, timestamp: UtcDateTime) -> Option<f64> {
        self.data
            .iter()
            .rev()
            .find_map(|&(idx, val)| if idx == timestamp { Some(val) } else { None })
    }

    fn lag(&self, timestamp: UtcDateTime, end_bound: UtcDateTime, lag: usize) -> Option<f64> {
        let mut count = 0;
        for &(idx, val) in self.data.iter().rev() {
            if idx <= timestamp && idx >= end_bound {
                if count == lag {
                    return Some(val);
                }
                count += 1;
            }
        }
        None
    }

    fn interval(&self, timestamp: UtcDateTime, intervals: usize) -> Vec<f64> {
        let mut result = Vec::new();
        // Go from the back and pick up to `periods` items with time <= timestamp
        for &(idx, val) in self.data.iter().rev() {
            if idx <= timestamp {
                debug!(target: "feature-store", "Interval found value {} at {}", val, idx);
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

    fn window(&self, start_time: UtcDateTime, end_time: UtcDateTime) -> Vec<f64> {
        let mut result = Vec::new();
        for &(idx, val) in self.data.iter().rev() {
            if idx > start_time && idx <= end_time {
                debug!(target: "feature-store", "Window found value {} at {}", val, idx);
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
}

#[derive(Debug, TypedBuilder)]
pub struct FeatureStore {
    /// In-memory feature store: (instrument, feature_id) -> BoundedBuffer of (timestamp, value)
    #[builder(default)]
    features: DashMap<(Arc<Instrument>, FeatureId), BoundedBuffer>,
    /// Time-to-live for insights in the buffer (how long to keep them)
    #[builder(default = Duration::from_secs(3600))] // 1 hour default
    ttl: Duration,
    /// Write-ahead log buffer for batched inserts (async-friendly)
    #[builder(default = Mutex::new(Vec::new()))]
    wal_buffer: Mutex<Vec<Arc<Insight>>>,
    /// Frequency of the pipeline in seconds (smallest time unit for aggregates)
    #[builder(default = 60)]
    min_interval: u64,
}

impl FeatureStore {
    pub fn new(ttl: u64) -> Self {
        Self {
            features: DashMap::new(),
            ttl: Duration::from_secs(ttl),
            wal_buffer: Mutex::new(Vec::new()),
            min_interval: 60, // Default 1 minute
        }
    }

    /// Insert an insight immediately into the feature store (bypasses WAL buffer)
    pub fn insert(&self, event: Arc<Insight>) {
        debug!(target: "feature-store", "Inserting insight {} into feature store", event.feature_id);
        let key = (event.instrument.clone(), event.feature_id.clone());
        let mut entry = self.features.entry(key).or_insert_with(BoundedBuffer::new);
        entry.push(event.event_time, event.value);
    }

    pub fn insert_batch(&self, events: &[Arc<Insight>]) {
        if events.is_empty() {
            return;
        }

        for event in events {
            debug!(target: "feature-store", "Inserting insight {} into feature store", event.feature_id);
        }

        // Last timestamp for TTL cleanup
        let last_time = events.iter().map(|e| e.event_time).max().unwrap_or(UtcDateTime::now());

        // Group by (instrument, feature_id)
        let grouped = events.iter().fold(HashMap::new(), |mut acc, event| {
            let key = (event.instrument.clone(), event.feature_id.clone());
            acc.entry(key).or_insert_with(Vec::new).push((event.event_time, event.value));
            acc
        });

        // Parallel sort and insert
        grouped.into_par_iter().for_each(|(key, mut values)| {
            // Sort this group's values by timestamp
            values.par_sort_unstable_by_key(|(time, _)| *time);

            // Insert the sorted values into the feature store
            let mut entry = self.features.entry(key).or_insert_with(BoundedBuffer::new);
            entry.extend_sorted(&values);

            // Clean up old values for this feature (while we have the lock)
            let cutoff_time = last_time - self.ttl;
            entry.remove_before(cutoff_time);
        });
    }

    /// Insert an insight into the WAL buffer (batched, will be committed later)
    pub async fn insert_buffered(&self, event: Arc<Insight>) {
        let mut buffer = self.wal_buffer.lock().await;
        buffer.push(event);
    }

    /// Batch insert into WAL buffer (async version)
    pub async fn insert_batch_buffered(&self, events: &[Arc<Insight>]) {
        let mut buffer = self.wal_buffer.lock().await;
        buffer.extend_from_slice(events);
    }

    /// Commit all buffered insights to the feature store
    /// This groups by (instrument, feature_id), sorts each group by timestamp,
    /// and bulk inserts with minimal lock contention
    pub async fn commit(&self, current_time: UtcDateTime) {
        // println!("Committing buffered insights");
        let mut buffer = self.wal_buffer.lock().await;

        if buffer.is_empty() {
            return;
        }

        // Stream: drain -> group -> sort in one go without extra allocations
        let grouped = buffer.drain(..).fold(HashMap::new(), |mut acc, event| {
            let key = (event.instrument.clone(), event.feature_id.clone());
            acc.entry(key).or_insert_with(Vec::new).push((event.event_time, event.value));
            acc
        });

        // Release the WAL buffer lock early since we've drained it
        drop(buffer);

        // Parallel sort and insert in one pass
        let cutoff_time = current_time - self.ttl;
        grouped.into_par_iter().for_each(|(key, mut values)| {
            // Sort this group's values by timestamp
            values.par_sort_unstable_by_key(|(time, _)| *time);

            // Insert the sorted values into the feature store
            let mut entry = self.features.entry(key).or_insert_with(BoundedBuffer::new);
            entry.extend_sorted(&values);

            // Clean up old values for this feature (while we have the lock)
            entry.remove_before(cutoff_time);
        });
    }

    /// Return the last value <= timestamp.
    pub fn last(&self, instrument: &Arc<Instrument>, feature_id: &FeatureId, timestamp: UtcDateTime) -> Option<f64> {
        let key = (instrument.clone(), feature_id.clone());

        self.features.get(&key).and_then(|buf| buf.last(timestamp))
    }

    /// Return a lagged value at timestamp - (lag * min_interval).
    /// Applies fill strategy if value doesn't exist at the expected timestamp.
    pub fn lag(
        &self,
        instrument: &Arc<Instrument>,
        feature_id: &FeatureId,
        timestamp: UtcDateTime,
        lag: usize,
        _fill_strategy: Option<FillStrategy>,
    ) -> Result<f64, StateQueryError> {
        let key = (instrument.clone(), feature_id.clone());
        let end_bound = timestamp - Duration::from_secs(self.min_interval * lag as u64);
        let value = self
            .features
            .get(&key)
            .map(|buf| buf.lag(timestamp, end_bound, lag))
            .ok_or_else(|| StateQueryError::NoBuffer {
                instrument: instrument.symbol.clone(),
                feature: format!("{:?}", feature_id),
            })?;
        match value {
            Some(v) => Ok(v),
            None => Err(StateQueryError::NoData),
        }

        // // Get buffer or error
        // let buffer = self.features.get(&key).ok_or_else(|| StateQueryError::NoBuffer {
        //     instrument: instrument.symbol.clone(),
        //     feature: format!("{:?}", feature_id),
        // })?;

        // // If no fill strategy provided, use default ForwardFill
        // let strategy = fill_strategy.unwrap_or(FillStrategy::ForwardFill);

        // // Calculate expected timestamp on the grid
        // let lag_offset = lag as i64 * self.min_interval as i64;
        // let target_timestamp = timestamp - Duration::from_secs(lag_offset as u64);

        // // Try to get value at exact timestamp
        // if let Some(value) = buffer.value_at(target_timestamp) {
        //     return Ok(value);
        // }

        // // Apply fill strategy
        // apply_fill_strategy(&buffer, target_timestamp, strategy)
    }

    /// Return the last `intervals` values up to `timestamp`, aligned to the frequency grid.
    /// Generates expected timestamps [T - (N-1)*freq, T - (N-2)*freq, ..., T - freq, T]
    /// and applies fill strategy for missing values.
    pub fn interval(
        &self,
        instrument: &Arc<Instrument>,
        feature_id: &FeatureId,
        timestamp: UtcDateTime,
        intervals: usize,
        _fill_strategy: Option<FillStrategy>,
    ) -> Result<Vec<f64>, StateQueryError> {
        let key = (instrument.clone(), feature_id.clone());
        let values = self
            .features
            .get(&key)
            .map(|buf| buf.interval(timestamp, intervals))
            .ok_or_else(|| StateQueryError::NoBuffer {
                instrument: instrument.symbol.clone(),
                feature: format!("{:?}", feature_id),
            })?;

        Ok(values)
    }

    /// Return a window of values in [start_time..end_time).
    pub fn window(
        &self,
        instrument: &Arc<Instrument>,
        feature_id: &FeatureId,
        timestamp: UtcDateTime,
        window: Duration,
    ) -> Vec<f64> {
        let start_time = timestamp - window;
        let key = (instrument.clone(), feature_id.clone());

        self.features
            .get(&key)
            .map(|buf| buf.window(start_time, timestamp))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkin_core::test_utils::{test_inst_binance_btc_usdt_perp, test_inst_binance_eth_usdt_perp, test_pipeline};
    use test_log::test;
    use time::UtcDateTime;
    use tracing::info;

    #[tokio::test]
    #[test_log::test]
    async fn test_large_batch_unsorted_commit() {
        use rand::rng;
        use rand::seq::SliceRandom;

        let pipeline = test_pipeline();
        // TTL needs to cover 1000 * 60 seconds = 60000 seconds = ~17 hours
        let state = FeatureStore::builder().ttl(Duration::from_secs(70000)).build();
        let now = UtcDateTime::now();
        let instrument = test_inst_binance_btc_usdt_perp();
        let feature_id = FeatureId::new("test_feature".to_string());

        // Generate 1000 insights with grid-aligned timestamps (60s intervals)
        let mut insights: Vec<_> = (0..1000)
            .map(|i| {
                // Create timestamps from oldest to newest: [now-999*60, now-998*60, ..., now-60, now]
                let timestamp = now - Duration::from_secs((999 - i) * 60);
                let insight = Insight::builder()
                    .event_time(timestamp)
                    .pipeline(Some(pipeline.clone()))
                    .instrument(instrument.clone())
                    .feature_id(feature_id.clone())
                    .value(i as f64)
                    .insight_type(InsightType::Raw)
                    .build();
                (Arc::new(insight), timestamp, i as f64)
            })
            .collect();

        // Shuffle them to create random insertion order
        let mut rng = rng();
        insights.shuffle(&mut rng);

        // Insert all shuffled insights
        for (insight, _, _) in &insights {
            state.insert_buffered(insight.clone()).await;
        }

        // Commit should sort them
        state.commit(now).await;

        // Verify they are sorted by checking interval returns sequential values
        let all_values = state.interval(&instrument, &feature_id, now, 1000, None).unwrap();
        assert_eq!(all_values.len(), 1000);

        // Values should be 0.0, 1.0, 2.0, ..., 999.0 in order
        for (i, &val) in all_values.iter().enumerate() {
            assert_eq!(val, i as f64, "Value at index {} should be {}, but got {}", i, i as f64, val);
        }

        // Verify first value (oldest) - grid aligned
        let first = state.last(&instrument, &feature_id, now - Duration::from_secs(999 * 60));
        assert_eq!(first, Some(0.0));

        // Verify last value (newest)
        let last = state.last(&instrument, &feature_id, now);
        assert_eq!(last, Some(999.0));

        // Verify middle values - grid aligned
        // At i=500: timestamp = now - 499*60, value=500
        // At i=499: timestamp = now - 500*60, value=499
        // So last(now - 500*60) should return 499
        let middle = state.last(&instrument, &feature_id, now - Duration::from_secs(500 * 60));
        assert_eq!(middle, Some(499.0));

        // Verify lag works correctly - uses grid alignment internally
        let value = state.lag(&instrument, &feature_id, now, 500, None).unwrap();
        assert_eq!(value, 499.0);
        assert_eq!(state.lag(&instrument, &feature_id, now, 0, None).unwrap(), 999.0);
        assert_eq!(state.lag(&instrument, &feature_id, now, 1, None).unwrap(), 998.0);
        assert_eq!(state.lag(&instrument, &feature_id, now, 10, None).unwrap(), 989.0);
        assert_eq!(state.lag(&instrument, &feature_id, now, 999, None).unwrap(), 0.0);

        // Verify window works correctly
        let window_end = now - Duration::from_secs(100 * 60);
        let window_values = state.window(&instrument, &feature_id, window_end, Duration::from_secs(10 * 60));

        // Window from (now-110*60) to (now-100*60)
        // i=890: timestamp = now-109*60 (included)
        // i=899: timestamp = now-100*60 (included)
        // So values 890 to 899
        info!("now: {:?}", now);
        assert_eq!(window_values.len(), 10); // Values 890-899
        for (i, &val) in window_values.iter().enumerate() {
            info!("Window value {}: {}", i, val);
            assert_eq!(val, (890 + i) as f64);
        }
    }

    #[tokio::test]
    async fn test_multiple_features_large_unsorted_batch() {
        use rand::rng;
        use rand::seq::SliceRandom;

        let pipeline = test_pipeline();
        // TTL needs to cover 250 * 60 seconds = 15000 seconds = ~4 hours
        let state = FeatureStore::builder().ttl(Duration::from_secs(20000)).build();
        let now = UtcDateTime::now();
        let instrument1 = test_inst_binance_btc_usdt_perp();
        let instrument2 = test_inst_binance_eth_usdt_perp();
        let feature1 = FeatureId::new("feature1".to_string());
        let feature2 = FeatureId::new("feature2".to_string());

        // Generate 250 insights for each combination (1000 total) - grid-aligned
        let mut all_insights = Vec::new();

        for (inst, feat, offset) in [
            (instrument1.clone(), feature1.clone(), 0.0),
            (instrument1.clone(), feature2.clone(), 1000.0),
            (instrument2.clone(), feature1.clone(), 2000.0),
            (instrument2.clone(), feature2.clone(), 3000.0),
        ] {
            for i in 0..250 {
                // Create timestamps from oldest to newest: [now-249*60, now-248*60, ..., now-60, now]
                let timestamp = now - Duration::from_secs((249 - i) * 60);
                let insight = Insight::builder()
                    .event_time(timestamp)
                    .pipeline(Some(pipeline.clone()))
                    .instrument(inst.clone())
                    .feature_id(feat.clone())
                    .value(offset + i as f64)
                    .insight_type(InsightType::Raw)
                    .build();
                all_insights.push(Arc::new(insight));
            }
        }

        // Shuffle all 1000 insights randomly
        let mut rng = rng();
        all_insights.shuffle(&mut rng);

        // Insert all shuffled insights
        for insight in &all_insights {
            state.insert_buffered(insight.clone()).await;
        }

        // Commit should group by (instrument, feature) and sort each group
        state.commit(now).await;

        // Verify each feature has 250 sorted values
        for (inst, feat, offset) in [
            (instrument1.clone(), feature1.clone(), 0.0),
            (instrument1.clone(), feature2.clone(), 1000.0),
            (instrument2.clone(), feature1.clone(), 2000.0),
            (instrument2.clone(), feature2.clone(), 3000.0),
        ] {
            let values = state.interval(&inst, &feat, now, 250, None).unwrap();
            assert_eq!(values.len(), 250);

            // Verify values are sequential
            for (i, &val) in values.iter().enumerate() {
                assert_eq!(
                    val,
                    offset + i as f64,
                    "Feature {:?} value at index {} should be {}, but got {}",
                    feat,
                    i,
                    offset + i as f64,
                    val
                );
            }
        }
    }

    #[test]
    fn test_push_out_of_order() {
        let mut buffer = BoundedBuffer::new();
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
        let mut buffer = BoundedBuffer::new();
        let now = UtcDateTime::now();
        buffer.push(now - Duration::from_secs(3601), 0.0);
        buffer.push(now - Duration::from_secs(2400), 1.0);
        buffer.push(now - Duration::from_secs(1200), 2.0);
        buffer.push(now, 3.0); // Exceeds capacity
        buffer.remove_before(now - Duration::from_secs(3600));

        let values: Vec<_> = buffer.data.iter().map(|(_, v)| *v).collect();
        assert_eq!(values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_last() {
        let mut buffer = BoundedBuffer::new();
        let now = UtcDateTime::now();
        buffer.push(now - Duration::from_secs(2), 0.0);
        buffer.push(now - Duration::from_secs(1), 1.0);
        buffer.push(now, 2.0);

        assert_eq!(buffer.last(now), Some(2.0));
        assert_eq!(buffer.last(now - Duration::from_secs(1)), Some(1.0));
        assert_eq!(buffer.last(now - Duration::from_secs(3)), None);
    }

    #[test]
    fn test_window() {
        let mut buffer = BoundedBuffer::new();
        let now = UtcDateTime::now();
        buffer.push(now - Duration::from_secs(4), 0.0);
        buffer.push(now - Duration::from_secs(3), 1.0);
        buffer.push(now - Duration::from_secs(2), 2.0);
        buffer.push(now - Duration::from_secs(1), 3.0);
        buffer.push(now, 4.0);

        let window = buffer.window(now - Duration::from_secs(3), now - Duration::from_secs(1));
        assert_eq!(window, vec![2.0, 3.0]);
    }

    #[test]
    fn test_intervals() {
        let mut buffer = BoundedBuffer::new();
        let now = UtcDateTime::now();
        buffer.push(now - Duration::from_secs(4), 0.0);
        buffer.push(now - Duration::from_secs(3), 1.0);
        buffer.push(now - Duration::from_secs(2), 2.0);
        buffer.push(now - Duration::from_secs(1), 3.0);
        buffer.push(now, 4.0);

        let intervals = buffer.interval(now, 3);
        assert_eq!(intervals, vec![2.0, 3.0, 4.0]);
        let past_intervals = buffer.interval(now - Duration::from_secs(2), 2);
        assert_eq!(past_intervals, vec![1.0, 2.0]);
    }

    #[test]
    fn test_lag() {
        let mut buffer = BoundedBuffer::new();
        let now = UtcDateTime::now();
        buffer.push(now - Duration::from_secs(4), 0.0);
        buffer.push(now - Duration::from_secs(3), 1.0);
        buffer.push(now - Duration::from_secs(2), 2.0);
        buffer.push(now - Duration::from_secs(1), 3.0);
        buffer.push(now, 4.0);

        let end_bound = now - Duration::from_secs(2);

        assert_eq!(buffer.lag(now, end_bound, 0), Some(4.0));
        assert_eq!(buffer.lag(now, end_bound, 1), Some(3.0));
        assert_eq!(buffer.lag(now, end_bound, 3), None);
        assert_eq!(buffer.lag(now, end_bound, 4), None);
        assert_eq!(buffer.lag(now, end_bound, 5), None);
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_insert_and_last() {
        let pipeline = test_pipeline();
        let state = FeatureStore::builder().build();
        let now = UtcDateTime::now();
        let instrument = test_inst_binance_btc_usdt_perp();
        let feature_id = FeatureId::new("test_feature".to_string());

        let t1 = now - Duration::from_secs(10);
        let t2 = now - Duration::from_secs(5);
        let t3 = now; // boundary

        let insight1 = Insight::builder()
            .event_time(t1)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(feature_id.clone())
            .value(1.1)
            .insight_type(InsightType::Raw)
            .build();
        let insight2 = Insight::builder()
            .event_time(t2)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(feature_id.clone())
            .insight_type(InsightType::Raw)
            .value(1.0)
            .build();
        let insight3 = Insight::builder()
            .event_time(t3)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(feature_id.clone())
            .insight_type(InsightType::Raw)
            .value(1.2)
            .build();
        state.insert_buffered(insight1.into()).await;
        state.insert_buffered(insight3.into()).await;
        state.insert_buffered(insight2.into()).await;
        state.commit(now).await;

        // "last" at time=now should find the inserted value
        let last = state.last(&instrument, &feature_id, now);
        assert_eq!(last, Some(1.2));

        let last = state.last(&instrument, &feature_id, now - Duration::from_secs(5));
        assert_eq!(last, Some(1.0));

        let last = state.last(&instrument, &feature_id, now - Duration::from_secs(10));
        assert_eq!(last, Some(1.1));

        let last = state.last(&instrument, &feature_id, now - Duration::from_secs(15));
        assert_eq!(last, None);
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_window_wal() {
        let pipeline = test_pipeline();
        let state = FeatureStore::builder().build();
        let now = UtcDateTime::now();
        let instrument = test_inst_binance_btc_usdt_perp();
        let feature_id = FeatureId::new("test_feature".to_string());

        let t1 = now - Duration::from_secs(10);
        let t2 = now - Duration::from_secs(5);
        let t3 = now; // boundary
        let insight1 = Insight::builder()
            .event_time(t1)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(feature_id.clone())
            .value(1.0)
            .insight_type(InsightType::Raw)
            .build();
        let insight2 = Insight::builder()
            .event_time(t2)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(feature_id.clone())
            .value(2.0)
            .insight_type(InsightType::Raw)
            .build();
        let insight3 = Insight::builder()
            .event_time(t3)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(feature_id.clone())
            .value(3.0)
            .insight_type(InsightType::Raw)
            .build();

        state.insert_buffered(insight1.into()).await;
        state.insert_buffered(insight2.into()).await;
        state.insert_buffered(insight3.into()).await;
        state.commit(now).await;

        let duration = Duration::from_secs(10);
        let results = state.window(&instrument, &feature_id, t3, duration);
        assert_eq!(results.len(), 2);
        assert_eq!(results, vec![2.0, 3.0]);
    }

    #[test]
    #[test_log::test]
    fn test_periods() {
        let pipeline = test_pipeline();
        let state = FeatureStore::builder().build();
        let now = UtcDateTime::now();
        let instrument = test_inst_binance_btc_usdt_perp();
        let feature_id = FeatureId::new("test_feature".to_string());

        // Insert multiple points at grid-aligned timestamps (60s intervals)
        let times = [
            now - Duration::from_secs(240), // T-240
            now - Duration::from_secs(180), // T-180
            now - Duration::from_secs(120), // T-120
            now - Duration::from_secs(60),  // T-60
            now,                            // T
        ];
        for (idx, t) in times.iter().enumerate() {
            let num = idx as f64;
            let i = Insight::builder()
                .event_time(*t)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(num)
                .insight_type(InsightType::Raw)
                .build();
            state.insert(i.into());
        }

        // interval(timestamp=now, intervals=3)
        // Grid: [T-120, T-60, T]
        // Values: [2.0, 3.0, 4.0]
        let p = state.interval(&instrument, &feature_id, now, 3, None).unwrap();
        assert_eq!(p, vec![2.0, 3.0, 4.0]);
    }

    #[tokio::test]
    async fn test_wal_buffer_and_commit() {
        let pipeline = test_pipeline();
        // TTL needs to cover 240 seconds
        let state = FeatureStore::builder().ttl(Duration::from_secs(300)).build();
        let now = UtcDateTime::now();
        let instrument = test_inst_binance_btc_usdt_perp();
        let feature_id = FeatureId::new("test_feature".to_string());

        // Insert some values into the WAL buffer at grid-aligned timestamps (60s intervals)
        let times = [
            now - Duration::from_secs(240), // T-240
            now - Duration::from_secs(180), // T-180
            now - Duration::from_secs(120), // T-120
            now - Duration::from_secs(60),  // T-60
            now,                            // T
        ];

        for (idx, t) in times.iter().enumerate() {
            let num = idx as f64;
            let i = Insight::builder()
                .event_time(*t)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(feature_id.clone())
                .value(num)
                .insight_type(InsightType::Raw)
                .build();
            state.insert_buffered(i.into()).await;
        }

        // Before commit, should not find values
        let last = state.last(&instrument, &feature_id, now);
        assert_eq!(last, None);

        // Commit the buffer
        state.commit(now).await;

        // After commit, should find the values
        let last = state.last(&instrument, &feature_id, now);
        assert_eq!(last, Some(4.0));

        // Test that values within TTL are available
        // Query last 3 intervals: [T-120, T-60, T]
        let intervals = state.interval(&instrument, &feature_id, now, 3, None).unwrap();
        assert_eq!(intervals, vec![2.0, 3.0, 4.0]);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let pipeline = test_pipeline();
        let state = FeatureStore::builder().ttl(Duration::from_secs(5)).build();
        let now = UtcDateTime::now();
        let instrument = test_inst_binance_btc_usdt_perp();
        let feature_id = FeatureId::new("test_feature".to_string());

        // Insert old value (beyond TTL=5s) - use grid-aligned timestamp
        let old_insight = Insight::builder()
            .event_time(now - Duration::from_secs(360)) // T-360 (6 minutes old, beyond TTL)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(feature_id.clone())
            .value(1.0)
            .insight_type(InsightType::Raw)
            .build();
        state.insert_buffered(old_insight.into()).await;

        // Insert recent value (within TTL=5s) - use grid-aligned timestamp
        let recent_insight = Insight::builder()
            .event_time(now) // T (within TTL)
            .pipeline(Some(pipeline.clone()))
            .instrument(instrument.clone())
            .feature_id(feature_id.clone())
            .value(2.0)
            .insight_type(InsightType::Raw)
            .build();
        state.insert_buffered(recent_insight.into()).await;

        // Commit with current time - old value should be filtered out
        state.commit(now).await;

        // Should only have the recent value
        let last = state.last(&instrument, &feature_id, now);
        assert_eq!(last, Some(2.0));

        // Should only return 1 interval (the recent one)
        // Query asks for 10 intervals but only 1 exists, forward fill will get NoData error
        // So we expect only the one value we have
        let intervals = state.interval(&instrument, &feature_id, now, 1, None).unwrap();
        assert_eq!(intervals, vec![2.0]);
    }

    #[tokio::test]
    async fn test_commit_with_multiple_instruments_and_features() {
        let pipeline = test_pipeline();
        // TTL needs to cover 360 seconds
        let state = FeatureStore::builder().ttl(Duration::from_secs(500)).build();
        let now = UtcDateTime::now();
        let instrument1 = test_inst_binance_btc_usdt_perp();
        let instrument2 = test_inst_binance_eth_usdt_perp();
        let feature_id1 = FeatureId::new("feature1".to_string());
        let feature_id2 = FeatureId::new("feature2".to_string());

        // Insert mixed data for different instruments and features at grid-aligned timestamps
        // This tests the grouping and sorting optimization
        let insights = vec![
            (instrument1.clone(), feature_id1.clone(), now - Duration::from_secs(240), 1.0), // T-240
            (instrument2.clone(), feature_id1.clone(), now - Duration::from_secs(180), 2.0), // T-180
            (instrument1.clone(), feature_id2.clone(), now - Duration::from_secs(120), 3.0), // T-120
            (instrument1.clone(), feature_id1.clone(), now - Duration::from_secs(360), 4.0), // T-360 (out of order)
            (instrument2.clone(), feature_id2.clone(), now - Duration::from_secs(60), 5.0),  // T-60
            (instrument1.clone(), feature_id1.clone(), now, 6.0),                            // T
            (instrument2.clone(), feature_id1.clone(), now - Duration::from_secs(300), 7.0), // T-300 (out of order)
        ];

        for (inst, feat, time, val) in insights {
            let insight = Insight::builder()
                .event_time(time)
                .pipeline(Some(pipeline.clone()))
                .instrument(inst)
                .feature_id(feat)
                .value(val)
                .insight_type(InsightType::Raw)
                .build();
            state.insert_buffered(insight.into()).await;
        }

        // Commit should group by (instrument, feature) and sort each group
        state.commit(now).await;

        // Verify instrument1, feature1 has values [4.0, 1.0, 6.0] sorted by time at [T-360, T-240, T]
        // Query last 5 intervals [T-240, T-180, T-120, T-60, T]
        // We have: T-360(4.0), T-240(1.0), T(6.0)
        // Expected fill: T-240(1.0), T-180(forward fill 1.0), T-120(forward fill 1.0), T-60(forward fill 1.0), T(6.0)
        let last = state.last(&instrument1, &feature_id1, now);
        assert_eq!(last, Some(6.0));
        let intervals = state.interval(&instrument1, &feature_id1, now, 5, None).unwrap();
        assert_eq!(intervals, vec![4.0, 1.0, 6.0]); // [T-240, T-180, T-120, T-60, T]

        // Verify instrument2, feature1 has values [7.0, 2.0] sorted by time
        // We have: T-300(7.0), T-180(2.0)
        // Query from T-180 for 2 intervals: [T-300, T-180]
        let intervals = state
            .interval(&instrument2, &feature_id1, now - Duration::from_secs(180), 2, None)
            .unwrap();
        assert_eq!(intervals, vec![7.0, 2.0]); // [T-300, T-180]

        // Verify instrument1, feature2 has value [3.0]
        let last = state.last(&instrument1, &feature_id2, now);
        assert_eq!(last, Some(3.0));

        // Verify instrument2, feature2 has value [5.0]
        let last = state.last(&instrument2, &feature_id2, now);
        assert_eq!(last, Some(5.0));
    }
}
