use std::time::Duration;

use time::OffsetDateTime;
use tracing::info;

pub struct Clock {
    start: OffsetDateTime,
    end: OffsetDateTime,
    frequency_secs: Duration,
    current_timestamp: OffsetDateTime,
}

impl Clock {
    pub fn new(start: OffsetDateTime, end: OffsetDateTime, frequency_secs: Duration) -> Self {
        info!(
            "Creating new clock with start: {}, end: {}, frequency_secs: {}",
            start,
            end,
            frequency_secs.as_secs()
        );
        Self {
            start,
            end,
            frequency_secs,
            current_timestamp: start,
        }
    }

    pub fn next(&mut self) -> Option<OffsetDateTime> {
        if self.current_timestamp >= self.end {
            return None;
        }

        let next_timestamp = self.current_timestamp;
        self.current_timestamp += self.frequency_secs;
        Some(next_timestamp)
    }

    pub fn reset(&mut self) {
        self.current_timestamp = self.start;
    }

    pub fn start(&self) -> OffsetDateTime {
        self.start
    }

    pub fn end(&self) -> OffsetDateTime {
        self.end
    }
}
