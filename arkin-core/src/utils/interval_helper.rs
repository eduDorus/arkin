use std::time::Duration;

use time::OffsetDateTime;

pub fn calculate_intervals(
    start: OffsetDateTime,
    end: OffsetDateTime,
    frequency_secs: Duration,
) -> (OffsetDateTime, i64) {
    let timestamp = start + frequency_secs;
    let intervals = ((end - start).whole_seconds() / frequency_secs.as_secs() as i64) - 1;
    (timestamp, intervals)
}
