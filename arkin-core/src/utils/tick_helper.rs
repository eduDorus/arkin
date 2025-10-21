use std::time::Duration;
use time::UtcDateTime;
use tokio::time::Duration as StdDuration;
use tokio::time::Interval;
use tokio::time::{interval_at, Instant, MissedTickBehavior};
use tracing::debug;

pub struct TickHelper {
    frequency: Duration,
    interval: Interval,
}

impl TickHelper {
    pub fn new(frequency: Duration) -> Self {
        let interval = Self::create_interval(frequency);
        TickHelper {
            frequency,
            interval,
        }
    }

    fn create_interval(interval: Duration) -> Interval {
        let now = UtcDateTime::now();
        debug!("Now: {:?}", now);

        let epoch = UtcDateTime::UNIX_EPOCH;

        // Calculate the difference between now and the next hour
        let difference = now - epoch;

        // Calculate the difference between now and the next tick
        let tick_difference = Duration::from_nanos((now.unix_timestamp_nanos() % interval.as_nanos() as i128) as u64);
        // let tick_difference = Duration::nanoseconds((now.unix_timestamp_nanos() % interval.whole_nanoseconds()) as i64);

        // Calculate the next tick
        let next_tick = difference - tick_difference + interval;
        debug!("Next Tick for new interval: {:?}", epoch + next_tick);

        // Calculate start time for interval
        let time_till_start = now - (epoch + next_tick);
        let start = Instant::now() + StdDuration::from_nanos(-time_till_start.whole_nanoseconds() as u64);
        let mut interval = interval_at(start, StdDuration::from_nanos(interval.as_nanos() as u64));
        interval.set_missed_tick_behavior(MissedTickBehavior::Burst);
        interval
    }

    pub async fn tick(&mut self) -> (UtcDateTime, Duration) {
        self.interval.tick().await;
        let mut ts = UtcDateTime::now();
        // Round to nearest second (Tick can be off by a few nanoseconds in either direction)
        if ts.nanosecond() > 500_000_000 {
            ts += Duration::from_secs(1);
        }
        ts = ts.replace_nanosecond(0).expect("Failed to replace nanosecond");

        let frequency = self.frequency;
        (ts, frequency)
    }
}
