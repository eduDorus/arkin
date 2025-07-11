use std::time::Duration;

use futures::{stream, Stream};
use time::UtcDateTime;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Clock {
    start: UtcDateTime,
    end: UtcDateTime,
    frequency_secs: Duration,
    current_timestamp: UtcDateTime,
}

impl Clock {
    pub fn new(start: UtcDateTime, end: UtcDateTime, frequency_secs: Duration) -> Self {
        info!(
            "Creating new clock with start: {}, end: {}, frequency_secs: {}",
            start,
            end,
            frequency_secs.as_secs()
        );
        Self {
            start: start.clone(),
            end: end.clone(),
            frequency_secs: frequency_secs.clone(),
            current_timestamp: start.clone(),
        }
    }

    pub fn next(&mut self) -> Option<(UtcDateTime, UtcDateTime)> {
        if self.current_timestamp >= self.end {
            return None;
        }

        let next_timestamp = self.current_timestamp;
        self.current_timestamp += self.frequency_secs;
        Some((next_timestamp, next_timestamp + self.frequency_secs))
    }

    pub fn to_stream(&mut self) -> impl Stream<Item = (UtcDateTime, UtcDateTime)> + Send + '_ {
        let mut intervals = Vec::new();
        while let Some((start, end)) = self.next() {
            intervals.push((start, end));
        }
        stream::iter(intervals)
    }

    pub fn reset(&mut self) {
        self.current_timestamp = self.start;
    }

    pub fn start(&self) -> UtcDateTime {
        self.start
    }

    pub fn end(&self) -> UtcDateTime {
        self.end
    }
}

impl Iterator for Clock {
    type Item = (UtcDateTime, UtcDateTime);

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
