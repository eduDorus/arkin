use time::UtcDateTime;

pub fn should_infer(event_time: UtcDateTime, interval: u64) -> bool {
    let total_seconds = event_time.unix_timestamp();
    (total_seconds % (interval as i64)) == 0
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use test_log::test;
    use time::macros::datetime;

    #[test(tokio::test)]
    async fn test_interval() {
        // Test case 1: Interval 300 seconds (5 minutes), on-mark times (should trigger)
        assert!(should_infer(datetime!(2025-08-21 10:00 UTC).into(), 300));
        assert!(should_infer(datetime!(2025-08-21 10:05 UTC).into(), 300));
        assert!(should_infer(datetime!(2025-08-21 23:55 UTC).into(), 300));
        assert!(should_infer(datetime!(2025-08-22 00:00 UTC).into(), 300)); // Day boundary

        // Test case 2: Interval 300 seconds (5 minutes), off-mark times (should not trigger)
        assert!(!should_infer(datetime!(2025-08-21 10:01 UTC).into(), 300));
        assert!(!should_infer(datetime!(2025-08-21 10:06 UTC).into(), 300));
        assert!(!should_infer(datetime!(2025-08-21 23:59 UTC).into(), 300));

        // Test case 3: Interval 60 seconds (1 minute, every minute)
        assert!(should_infer(datetime!(2025-08-21 10:00 UTC).into(), 60));
        assert!(should_infer(datetime!(2025-08-21 10:01 UTC).into(), 60));

        // Test case 4: Interval 600 seconds (10 minutes), mixed
        assert!(should_infer(datetime!(2025-08-21 10:00 UTC).into(), 600));
        assert!(should_infer(datetime!(2025-08-21 10:10 UTC).into(), 600));
        assert!(!should_infer(datetime!(2025-08-21 10:05 UTC).into(), 600));

        // Test case 5: Interval 5 seconds, granular checks
        let base = datetime!(2025-08-21 10:00:00 UTC);
        assert!(should_infer(base.into(), 5));
        let one_sec = base + Duration::from_secs(1);
        assert!(!should_infer(one_sec.into(), 5));
        let five_sec = base + Duration::from_secs(5);
        assert!(should_infer(five_sec.into(), 5));
        let six_sec = base + Duration::from_secs(6);
        assert!(!should_infer(six_sec.into(), 5));
    }
}
