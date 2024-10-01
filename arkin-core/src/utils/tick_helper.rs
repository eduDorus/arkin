use time::{Duration, OffsetDateTime};
use tokio::time::Duration as StdDuration;
use tokio::time::Interval;
use tokio::time::{interval_at, Instant, MissedTickBehavior};

pub fn create_interval(interval: Duration) -> Interval {
    let now = OffsetDateTime::now_utc();
    println!("Now: {:?}", now);

    let epoch = OffsetDateTime::UNIX_EPOCH;

    // Calculate the difference between now and the next hour
    let difference = now - epoch;

    // Calculate the difference between now and the next tick
    let tick_difference = Duration::nanoseconds((now.unix_timestamp_nanos() % interval.whole_nanoseconds()) as i64);

    // Calculate the next tick
    let next_tick = difference - tick_difference + interval;
    println!("Next Tick for new interval: {:?}", epoch + next_tick);

    // Calculate start time for interval
    let time_till_start = now - (epoch + next_tick);
    println!("Time till start: {:?}", time_till_start);

    let start = Instant::now() + StdDuration::from_nanos(-time_till_start.whole_nanoseconds() as u64);
    println!("Start: {:?}", start);
    let mut interval = interval_at(start, StdDuration::from_nanos(interval.whole_nanoseconds() as u64));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    println!("Interval created with duration: {:?}", interval);
    interval
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging;

    #[tokio::test]
    #[ignore]
    async fn test_create_interval() {
        logging::init_test_tracing();
        let mut interval = create_interval(Duration::seconds(5));

        interval.tick().await;
        println!("Timestamp: {:?}", OffsetDateTime::now_utc());
    }
}
