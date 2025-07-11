use std::fmt;

use time::UtcDateTime;

use crate::constants::TIMESTAMP_FORMAT;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub struct CompositeIndex {
    pub timestamp: i128,
    pub index: u64,
}

impl CompositeIndex {
    pub fn new(timestamp: UtcDateTime) -> Self {
        CompositeIndex {
            timestamp: timestamp.unix_timestamp_nanos(),
            index: 0,
        }
    }

    pub fn new_max(timestamp: UtcDateTime) -> Self {
        CompositeIndex {
            timestamp: timestamp.unix_timestamp_nanos(),
            index: u64::MAX,
        }
    }

    pub fn increment(&mut self) {
        self.index += 1;
    }

    pub fn as_timestamp(&self) -> UtcDateTime {
        UtcDateTime::from_unix_timestamp_nanos(self.timestamp).expect("Failed to create timestamp")
    }
}

impl fmt::Display for CompositeIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let timestamp = UtcDateTime::from_unix_timestamp_nanos(self.timestamp).expect("Failed to create timestamp");
        write!(
            f,
            "{}-{}",
            timestamp.format(TIMESTAMP_FORMAT).expect("Failed to format timestamp"),
            self.index
        )
    }
}
