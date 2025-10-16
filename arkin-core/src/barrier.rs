use std::{sync::Arc, time::Duration};

use tokio::sync::Barrier;

pub struct SyncBarrier {
    barrier: Arc<Barrier>,
    window_duration: Duration,
}

impl SyncBarrier {
    pub fn new(parties: usize, window_duration: Duration) -> Self {
        Self {
            barrier: Arc::new(Barrier::new(parties)),
            window_duration,
        }
    }

    pub async fn confirm_and_wait(&self) {
        self.barrier.wait().await;
    }

    pub fn window_duration(&self) -> Duration {
        self.window_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::{sleep, Instant};
    use tracing::info;

    #[tokio::test]
    #[test_log::test]
    async fn test_sync_barrier_with_three_producers() {
        let parties = 5;
        let window_duration = Duration::from_millis(100);
        let barrier = Arc::new(SyncBarrier::new(parties, window_duration));

        // Shared state to track the order of arrivals and releases
        let arrival_times = Arc::new(Mutex::new(Vec::new()));
        let release_times = Arc::new(Mutex::new(Vec::new()));

        let mut handles = vec![];

        for i in 0..parties {
            let barrier_clone = Arc::clone(&barrier);
            let arrival_times_clone = Arc::clone(&arrival_times);
            let release_times_clone = Arc::clone(&release_times);

            let handle = tokio::spawn(async move {
                // Simulate different arrival times
                sleep(Duration::from_millis(i as u64 * 100)).await;

                let arrival_time = Instant::now();
                arrival_times_clone.lock().await.push((i, arrival_time));

                // Wait at the barrier
                barrier_clone.confirm_and_wait().await;

                let release_time = Instant::now();
                release_times_clone.lock().await.push((i, release_time));
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all producers arrived and were released
        let arrivals = arrival_times.lock().await;
        let releases = release_times.lock().await;

        assert_eq!(arrivals.len(), parties);
        assert_eq!(releases.len(), parties);

        info!("Arrival times: {:?}", *arrivals);
        info!("Release times: {:?}", *releases);

        // Verify that all producers were released at approximately the same time
        // (after the last one arrived at the barrier)
        let last_arrival_time = arrivals.iter().map(|(_, t)| t).max().unwrap();

        for (_, release_time) in releases.iter() {
            // All release times should be after the last arrival
            assert!(release_time >= last_arrival_time);

            // All releases should happen within a small time window (e.g., 50ms)
            let time_diff = release_time.duration_since(*last_arrival_time);
            assert!(time_diff < Duration::from_millis(50), "Release took too long: {:?}", time_diff);
        }
    }

    #[tokio::test]
    async fn test_window_duration_getter() {
        let window_duration = Duration::from_millis(500);
        let barrier = SyncBarrier::new(2, window_duration);

        assert_eq!(barrier.window_duration(), window_duration);
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_five_ingestors_with_hourly_windows() {
        use tokio::sync::mpsc;

        // Simulate 5 ingestors producing data for 1 day with hourly windows
        let num_ingestors = 5;
        let hours_per_day = 24;
        let window_duration = Duration::from_secs(3600); // 1 hour in real time (simulated)

        // Create barrier with 5 producers + 1 consumer = 6 parties
        let barrier = Arc::new(SyncBarrier::new(num_ingestors + 1, window_duration));

        // Create a channel for each ingestor to send data to the consumer
        let (tx, mut rx) = mpsc::channel::<String>(100);

        let mut handles = vec![];

        // Spawn 5 ingestor tasks
        for ingestor_id in 0..num_ingestors {
            let barrier_clone = Arc::clone(&barrier);
            let tx_clone = tx.clone();

            let handle = tokio::spawn(async move {
                info!("Ingestor {} started", ingestor_id);

                // Produce data for 24 hourly windows (1 day)
                for hour in 0..hours_per_day {
                    // Simulate variable processing time for each ingestor
                    let processing_delay = Duration::from_millis(10 + (ingestor_id * 5) as u64);
                    sleep(processing_delay).await;

                    // Produce data for this hour
                    let data = format!("Ingestor-{}_Hour-{}", ingestor_id, hour);
                    tx_clone.send(data.clone()).await.unwrap();
                    info!("Ingestor {} produced: {}", ingestor_id, data);

                    // Wait at the barrier - all ingestors must reach this point before proceeding
                    barrier_clone.confirm_and_wait().await;
                    info!("Ingestor {} released from barrier for hour {}", ingestor_id, hour);
                }

                info!("Ingestor {} completed all windows", ingestor_id);
            });

            handles.push(handle);
        }

        // Drop the original sender so the channel closes when all ingestors are done
        drop(tx);

        // Spawn consumer task
        let consumer_barrier = Arc::clone(&barrier);

        let consumer_handle = tokio::spawn(async move {
            info!("Consumer started");

            let mut consumed_windows = Vec::new();
            let mut hour_buffers: Vec<Vec<String>> = vec![Vec::new(); hours_per_day];

            for hour in 0..hours_per_day {
                // Collect data for this hour from all ingestors
                // Each ingestor will send one message before hitting the barrier
                for _ in 0..num_ingestors {
                    if let Some(data) = rx.recv().await {
                        // Parse the hour from the data to ensure it's for the current window
                        if let Some(data_hour) = data.split("Hour-").nth(1).and_then(|s| s.parse::<usize>().ok()) {
                            hour_buffers[data_hour].push(data.clone());
                            info!("Consumer received: {}", data);
                        }
                    }
                }

                // Wait at the barrier - consumer can only proceed when all producers have hit the barrier
                consumer_barrier.confirm_and_wait().await;
                info!("Consumer released from barrier for hour {}", hour);

                // Verify that all 5 ingestors have produced data for this hour
                assert_eq!(
                    hour_buffers[hour].len(),
                    num_ingestors,
                    "Consumer at hour {} should see data from all {} ingestors, but only saw {}",
                    hour,
                    num_ingestors,
                    hour_buffers[hour].len()
                );

                info!("Consumer processing hour {}: {:?}", hour, hour_buffers[hour]);

                // Simulate consumer processing
                sleep(Duration::from_millis(5)).await;

                consumed_windows.push(hour);
            }

            info!("Consumer completed all windows");

            // Return the consumed windows and hour buffers for verification
            (consumed_windows, hour_buffers)
        });

        // Wait for all ingestor tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Wait for consumer and get the results
        let (consumed_windows, hour_buffers) = consumer_handle.await.unwrap();

        // Verify all data was produced and organized correctly
        let total_messages: usize = hour_buffers.iter().map(|v: &Vec<String>| v.len()).sum();
        assert_eq!(
            total_messages,
            num_ingestors * hours_per_day,
            "Should have {} total data points",
            num_ingestors * hours_per_day
        );

        // Verify each hour has data from all ingestors
        for (hour, buffer) in hour_buffers.iter().enumerate() {
            assert_eq!(
                buffer.len(),
                num_ingestors,
                "Hour {} should have data from all {} ingestors",
                hour,
                num_ingestors
            );
        }

        // Verify all windows were consumed
        assert_eq!(
            consumed_windows.len(),
            hours_per_day,
            "Consumer should have processed all {} hourly windows",
            hours_per_day
        );

        // Verify windows were consumed in order
        for (i, &hour) in consumed_windows.iter().enumerate() {
            assert_eq!(hour, i, "Windows should be consumed in order");
        }
    }
}
