use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tracing::warn;

#[async_trait]
pub trait SimulationClock: Send + Sync {
    async fn get_current_time(&self) -> OffsetDateTime;
    async fn advance_time(&self, time: OffsetDateTime);
    async fn is_final_hour(&self) -> bool;
    async fn is_finished(&self) -> bool;
    async fn is_live(&self) -> bool {
        false
    }
}

pub struct LiveModeClock;

#[async_trait]
impl SimulationClock for LiveModeClock {
    async fn get_current_time(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }

    async fn advance_time(&self, _time: OffsetDateTime) {
        // No-op in production mode
        warn!("advance_time is a no-op in production mode");
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
}

pub struct SimulationModeClock {
    current_time: Arc<RwLock<OffsetDateTime>>,
    end_time: OffsetDateTime,
}

impl SimulationModeClock {
    pub fn new(start_time: OffsetDateTime, end_time: OffsetDateTime) -> Self {
        Self {
            current_time: Arc::new(RwLock::new(start_time)),
            end_time,
        }
    }
}

#[async_trait]
impl SimulationClock for SimulationModeClock {
    async fn get_current_time(&self) -> OffsetDateTime {
        self.current_time.read().await.clone()
    }

    async fn advance_time(&self, time: OffsetDateTime) {
        self.current_time.write().await.clone_from(&time);
    }

    async fn is_final_hour(&self) -> bool {
        let current_time = self.current_time.read().await;
        let end_time_minus_one_hour = self.end_time - Duration::from_secs(3600);
        *current_time >= end_time_minus_one_hour
    }

    async fn is_finished(&self) -> bool {
        let current_time = self.current_time.read().await;
        *current_time >= self.end_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[tokio::test]
    async fn test_simulation_clock() {
        let start_time = datetime!(2023-10-01 12:00:00 UTC);
        let end_time = datetime!(2023-10-01 14:00:00 UTC);
        let clock = SimulationModeClock::new(start_time, end_time);

        assert_eq!(clock.get_current_time().await, start_time);
        assert!(!clock.is_finished().await);

        let new_time = datetime!(2023-10-01 13:00:00 UTC);
        clock.advance_time(new_time).await;

        assert_eq!(clock.get_current_time().await, new_time);
        assert!(!clock.is_finished().await);

        clock.advance_time(end_time).await;
        assert_eq!(clock.get_current_time().await, end_time);
        assert!(clock.is_finished().await);
    }
}
