use std::fmt;

use time::OffsetDateTime;

use crate::constants::TIMESTAMP_FORMAT;

#[derive(Clone, Copy, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub struct CompositeIndex {
    timestamp: OffsetDateTime,
    index: u64,
}

impl CompositeIndex {
    pub fn new(timestamp: &OffsetDateTime) -> Self {
        CompositeIndex {
            timestamp: timestamp.to_owned(),
            index: 0,
        }
    }

    pub fn new_max(timestamp: &OffsetDateTime) -> Self {
        CompositeIndex {
            timestamp: timestamp.to_owned(),
            index: u64::MAX,
        }
    }

    pub fn increment(&mut self) {
        self.index += 1;
    }
}

impl fmt::Display for CompositeIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}-{}",
            self.timestamp.format(TIMESTAMP_FORMAT).expect("Failed to format timestamp"),
            self.index
        )
    }
}