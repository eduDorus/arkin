use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use time::UtcDateTime;
use tokio::sync::RwLock;
use tracing::error;

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

    async fn advance_time(&self, _time: UtcDateTime) {
        // No-op in production mode
        error!("advance_time is a no-op in production mode");
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

pub struct SimulationSystemTime {
    current_time: Arc<RwLock<UtcDateTime>>,
    end_time: UtcDateTime,
}

impl SimulationSystemTime {
    pub fn new(start_time: UtcDateTime, end_time: UtcDateTime) -> Arc<Self> {
        Self {
            current_time: Arc::new(RwLock::new(start_time)),
            end_time,
        }
        .into()
    }
}

#[async_trait]
impl SystemTime for SimulationSystemTime {
    async fn now(&self) -> UtcDateTime {
        self.current_time.read().await.clone()
    }

    async fn advance_time(&self, time: UtcDateTime) {
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

    async fn is_live(&self) -> bool {
        false
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
        let clock = SimulationSystemTime::new(start_time, end_time);

        assert_eq!(clock.now().await, start_time);
        assert!(!clock.is_finished().await);

        let new_time = datetime!(2023-10-01 13:00:00 UTC).to_utc();
        clock.advance_time(new_time).await;

        assert_eq!(clock.now().await, new_time);
        assert!(!clock.is_finished().await);

        clock.advance_time(end_time).await;
        assert_eq!(clock.now().await, end_time);
        assert!(clock.is_finished().await);
    }
}
