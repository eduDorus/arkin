// Allow dead code
#![allow(dead_code)]
use parking_lot::RwLock;
use std::{
    collections::HashMap,
    time::{Duration, Instant, UNIX_EPOCH},
};
use time::OffsetDateTime;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, error, info};

use crate::{config::ClockConfig, constants::TIMESTAMP_FORMAT};

pub struct Clock {
    pub subscribers: RwLock<HashMap<Duration, Sender<OffsetDateTime>>>,
    pub tick_frequency: Duration,
}

impl Clock {
    pub fn from_config(config: &ClockConfig) -> Self {
        let tick_frequency = Duration::from_secs(config.tick_frequency);
        info!("Creating time component with tick frequency: {:?}", tick_frequency);
        Clock {
            subscribers: RwLock::new(HashMap::new()),
            tick_frequency,
        }
    }

    pub async fn start(&self) {
        info!("Starting time component...");
        loop {
            let (start, tick_time) = self.calculate_next_tick(self.tick_frequency);
            tokio::time::sleep_until(start.into()).await;

            debug!("Time component tick: {:?}", tick_time);
            for (frequency, sender) in self.subscribers.read().iter() {
                let diff = tick_time.unix_timestamp_nanos() as u128 % frequency.as_nanos();
                if diff != 0 {
                    debug!("Skipping time event for frequency: {:?}", frequency);
                    continue;
                }
                if let Err(e) = sender.send(tick_time) {
                    error!("Failed to send time event: {:?}", e);
                }
            }
        }
    }

    pub fn calculate_next_tick(&self, interval: Duration) -> (Instant, OffsetDateTime) {
        let now = OffsetDateTime::now_utc();

        // Calculate the difference between now and the epoch
        let difference = now - UNIX_EPOCH;

        // Calculate the difference between now and the next tick
        let tick_difference = Duration::from_nanos((now.unix_timestamp_nanos() as u128 % interval.as_nanos()) as u64);
        debug!("Tick difference: {:?}", tick_difference);

        // Calculate the next tick
        let next_tick = difference - tick_difference + interval;
        let next_tick_time = OffsetDateTime::from_unix_timestamp_nanos(next_tick.whole_nanoseconds()).unwrap();
        debug!("Next Tick in: {:?}", next_tick);
        debug!("Next Tick for new interval: {:?}", next_tick_time.format(TIMESTAMP_FORMAT));

        // Calculate start time for interval
        let time_till_start = now - (UNIX_EPOCH + next_tick);
        debug!("Time till start: {:?}", time_till_start);

        let start = Instant::now() + Duration::from_nanos(-time_till_start.whole_nanoseconds() as u64);
        debug!("Start: {:?}", start);
        (start, next_tick_time)
    }

    pub fn subscribe(&self, frequency: Duration) -> Receiver<OffsetDateTime> {
        info!("Subscribing to time component with frequency: {:?}", frequency);
        if let Some(sender) = self.subscribers.read().get(&frequency) {
            info!("Found existing subscriber for frequency: {:?}", frequency);
            return sender.subscribe();
        }

        info!("Creating new subscriber for frequency: {:?}", frequency);
        let (sender, receiver) = broadcast::channel(1);
        self.subscribers.write().insert(frequency, sender);
        receiver
    }
}

#[cfg(test)]
mod tests {
    use crate::logging;

    use super::*;

    #[tokio::test]
    async fn test_time_component() {
        logging::init_test_tracing();
        info!("Starting time component test...");
        let config = ClockConfig { tick_frequency: 1 };
        let time_component = Clock::from_config(&config);
        let mut rx_5_1 = time_component.subscribe(Duration::from_secs(5));
        let mut rx_5_2 = time_component.subscribe(Duration::from_secs(5));
        let mut rx_10_1 = time_component.subscribe(Duration::from_secs(10));

        info!("Spawning time component...");
        tokio::spawn(async move {
            time_component.start().await;
        });

        let ts = rx_5_1.recv().await.unwrap();
        info!("Test received time event: {:?}", ts);
        let ts = rx_5_2.recv().await.unwrap();
        info!("Test received time event: {:?}", ts);
        let ts = rx_10_1.recv().await.unwrap();
        info!("Test received time event: {:?}", ts);
    }
}
