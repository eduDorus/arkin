use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use time::UtcDateTime;
use tokio::sync::RwLock;
use tracing::{debug, error};

use crate::SystemTime;

pub struct LiveSystemTime;

impl LiveSystemTime {
    pub fn new() -> Arc<Self> {
        Self.into()
    }
}

#[async_trait]
impl SystemTime for LiveSystemTime {
    async fn now(&self) -> UtcDateTime {
        UtcDateTime::now()
    }

    async fn advance_time_to(&self, _time: UtcDateTime) {
        // No-op in production mode
        error!("advance time to is a no-op in production mode");
    }

    async fn advance_time_by(&self, _duration: Duration) {
        error!("advance time by is a no-op in production mode");
    }

    async fn is_final_hour(&self) -> bool {
        false
    }

    async fn is_finished(&self) -> bool {
        false
    }

    async fn is_live(&self) -> bool {
        true
    }

    async fn check_interval(&self) -> Vec<UtcDateTime> {
        vec![]
    }
}

#[derive(Clone, Copy, Debug)]
struct SimTimeState {
    current: UtcDateTime,
    next_tick: UtcDateTime,
    initialized: bool,
}

pub struct SimulationSystemTime {
    state: RwLock<SimTimeState>,
    end_time: UtcDateTime,
    tick_frequency: Duration,
}

impl SimulationSystemTime {
    pub fn new(start_time: UtcDateTime, end_time: UtcDateTime, tick_frequency: Duration) -> Arc<Self> {
        Self {
            state: RwLock::new(SimTimeState {
                current: start_time,
                next_tick: start_time + tick_frequency,
                initialized: false,
            }),
            end_time,
            tick_frequency,
        }
        .into()
    }
}

#[async_trait]
impl SystemTime for SimulationSystemTime {
    async fn now(&self) -> UtcDateTime {
        self.state.read().await.current
    }

    async fn advance_time_to(&self, time: UtcDateTime) {
        // We can only move forward in time
        let current_time = self.state.read().await.current;
        match (current_time, time) {
            (current, new) if current < new => {
                self.state.write().await.current = new;
                debug!(target: "time", "advanced time to {}", new);
            }
            (current, new) if current == new => {
                // No-op
            }
            (_current, _new) => {
                // warn!(target: "time", "attempted to move time backwards from {} to {}", current, new);
            }
        }
    }

    async fn advance_time_by(&self, duration: Duration) {
        self.state.write().await.current += duration;
    }

    async fn is_final_hour(&self) -> bool {
        let current_time = self.state.read().await.current;
        let end_time_minus_one_hour = self.end_time - Duration::from_secs(3600);
        current_time >= end_time_minus_one_hour
    }

    async fn is_finished(&self) -> bool {
        let current_time = self.state.read().await.current;
        current_time >= self.end_time
    }

    async fn is_live(&self) -> bool {
        false
    }

    async fn check_interval(&self) -> Vec<UtcDateTime> {
        let mut guard = self.state.write().await;
        let mut ticks = Vec::new();

        // On first call, emit the first tick
        if !guard.initialized {
            guard.initialized = true;
            ticks.push(guard.next_tick);
            guard.next_tick += self.tick_frequency;
            return ticks;
        }

        // Subsequent calls: emit ticks if current has advanced past them
        while guard.current >= guard.next_tick {
            ticks.push(guard.next_tick);
            guard.next_tick += self.tick_frequency;
        }

        ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[tokio::test]
    async fn test_simulation_clock() {
        let start_time = datetime!(2023-10-01 12:00:00 UTC).to_utc();
        let end_time = datetime!(2023-10-01 14:00:00 UTC).to_utc();
        let clock = SimulationSystemTime::new(start_time, end_time, Duration::from_secs(60));

        let intervals = clock.check_interval().await;
        assert_eq!(intervals.len(), 1);
        assert_eq!(intervals[0], datetime!(2023-10-01 12:01:00 UTC).to_utc());

        assert_eq!(clock.now().await, start_time);
        assert!(!clock.is_finished().await);

        let new_time = datetime!(2023-10-01 13:00:00 UTC).to_utc();
        clock.advance_time_to(new_time).await;

        assert_eq!(clock.now().await, new_time);
        assert!(!clock.is_finished().await);

        clock.advance_time_to(end_time).await;
        assert_eq!(clock.now().await, end_time);
        assert!(clock.is_finished().await);
    }
}
