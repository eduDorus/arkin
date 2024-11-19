use time::{Duration, OffsetDateTime};
use tokio::time::Duration as StdDuration;
use tokio::time::Interval;
use tokio::time::{interval_at, Instant, MissedTickBehavior};
use tracing::debug;

pub struct TickHelper {
    pub interval: Interval,
}

impl TickHelper {
    pub fn new(interval: Duration) -> Self {
        let interval = Self::create_interval(interval);
        TickHelper { interval }
    }

    fn create_interval(interval: Duration) -> Interval {
        let now = OffsetDateTime::now_utc();
        debug!("Now: {:?}", now);

        let epoch = OffsetDateTime::UNIX_EPOCH;

        // Calculate the difference between now and the next hour
        let difference = now - epoch;

        // Calculate the difference between now and the next tick
        let tick_difference = Duration::nanoseconds((now.unix_timestamp_nanos() % interval.whole_nanoseconds()) as i64);

        // Calculate the next tick
        let next_tick = difference - tick_difference + interval;
        debug!("Next Tick for new interval: {:?}", epoch + next_tick);

        // Calculate start time for interval
        let time_till_start = now - (epoch + next_tick);
        let start = Instant::now() + StdDuration::from_nanos(-time_till_start.whole_nanoseconds() as u64);
        let mut interval = interval_at(start, StdDuration::from_nanos(interval.whole_nanoseconds() as u64));
        interval.set_missed_tick_behavior(MissedTickBehavior::Burst);
        interval
    }

    pub async fn tick(&mut self) -> OffsetDateTime {
        self.interval.tick().await;
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("Failed to replace nanosecond")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;
    use tracing::info;

    #[test(tokio::test)]
    async fn test_create_interval() {
        let mut interval = TickHelper::new(Duration::seconds(60));

        // I get the instant of the tick
        loop {
            let tick = interval.tick().await;
            info!("Tick: {:?}", tick);
        }
    }
}
