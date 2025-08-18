use std::{pin::Pin, sync::Arc, time::Duration};

use async_trait::async_trait;
use time::UtcDateTime;

use arkin_core::prelude::*;
use tracing::info;

#[derive(Debug, Clone)]
pub struct CronInterval {
    start: UtcDateTime,
    frequency: Duration,
    event: EventType,
}

impl CronInterval {
    pub fn new(start: UtcDateTime, frequency: Duration, event: EventType) -> Self {
        Self {
            start,
            frequency,
            event,
        }
    }
}

pub struct Cron {
    intervals: Vec<CronInterval>,
}

impl Cron {
    pub fn new(intervals: Vec<CronInterval>) -> Self {
        Self { intervals }
    }
}

async fn interval_task(interval: CronInterval, service_ctx: Arc<ServiceCtx>, core_ctx: Arc<CoreCtx>) {
    let shutdown = service_ctx.get_shutdown_token();
    if !core_ctx.time.is_live().await {
        // No-op in sim; ticks handled reactively elsewhere (e.g., in event processor)
        return;
    }
    let freq = interval.frequency;
    if freq.is_zero() {
        // Avoid infinite loop/division by zero
        return;
    }
    let mut next_fire = interval.start;
    loop {
        let now = core_ctx.now().await;
        if now < next_fire {
            let delay = next_fire - now;
            let delay = std::time::Duration::from_nanos(delay.whole_nanoseconds() as u64);
            tokio::select! {
                _ = tokio::time::sleep(delay) => {},
                _ = shutdown.cancelled() => return,
            }
        } else {
            // Fast-forward to next future slot if past
            let elapsed = now - interval.start;
            let periods = (elapsed.whole_seconds() as u64 / freq.as_secs()) + 1;
            next_fire = interval.start + time::Duration::seconds((periods * freq.as_secs()) as i64);
            continue; // Re-check to sleep if still past (rare)
        }
        // Emit
        info!(target: "cron", "firing event {} at {}", interval.event, next_fire);
        core_ctx
            .publish(Event::InsightsTick(
                InsightsTick::builder()
                    .event_time(next_fire)
                    .frequency(interval.frequency)
                    .build()
                    .into(),
            ))
            .await;
        next_fire += freq;
    }
}

#[async_trait]
impl Runnable for Cron {
    async fn get_tasks(
        self: Arc<Self>,
        service_ctx: Arc<ServiceCtx>,
        core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        let mut tasks: Vec<Pin<Box<dyn Future<Output = ()> + Send>>> = Vec::new();
        for interval in self.intervals.iter() {
            let task = interval_task(interval.clone(), service_ctx.clone(), core_ctx.clone());
            tasks.push(Box::pin(task));
        }
        tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_log;

    #[tokio::test]
    #[test_log::test]
    async fn test_cron() {
        let time = LiveSystemTime::new();
        let publisher = MockPublisher::new();
        let persistence = MockPersistence::new();

        let start = time.now().await.replace_second(0).unwrap().replace_nanosecond(0).unwrap();
        let frequency = Duration::from_secs(10);
        let interval = CronInterval {
            start,
            frequency,
            event: EventType::InsightsTick,
        };
        let chron = Arc::new(Cron::new(vec![interval]));
        let service = Service::new("test", chron, time.clone(), publisher.clone(), None, persistence);
        service.start().await;
        tokio::time::sleep(Duration::from_secs(60)).await; // Allow one tick to fire
        let events = publisher.get_events().await;
        assert_eq!(events.len(), 6);
    }
}
