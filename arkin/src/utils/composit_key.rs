use std::fmt;

use time::OffsetDateTime;

use crate::constants::TIMESTAMP_FORMAT;

#[derive(Clone, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub struct CompositeKey {
    timestamp: OffsetDateTime,
    index: u64,
}

impl CompositeKey {
    pub fn new(timestamp: &OffsetDateTime) -> Self {
        CompositeKey {
            timestamp: timestamp.to_owned(),
            index: 0,
        }
    }

    pub fn new_max(timestamp: &OffsetDateTime) -> Self {
        CompositeKey {
            timestamp: timestamp.to_owned(),
            index: u64::MAX,
        }
    }

    pub fn increment(&mut self) {
        self.index += 1;
    }
}

impl fmt::Display for CompositeKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}-{}",
            self.timestamp.format(TIMESTAMP_FORMAT).expect("Failed to format timestamp"),
            self.index
        )
    }
}
