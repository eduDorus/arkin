use moka2::future::Cache;

pub struct Deduplicator {
    cache: Cache<String, ()>,
}

impl Deduplicator {
    pub fn new(max_recent: u64) -> Self {
        Deduplicator {
            cache: Cache::new(max_recent),
        }
    }

    pub async fn has(&self, new_string: &str) -> bool {
        if self.cache.contains_key(new_string) {
            true
        } else {
            self.cache.insert(new_string.to_owned(), ()).await;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use test_log;

    use super::*;

    #[tokio::test]
    #[test_log::test]
    async fn test_deduplicator() {
        let deduplicator = Deduplicator::new(10);

        let input1 = "hello";
        let input2 = "world";
        let input3 = "rust";
        let input4 = "baby"; // Duplicate

        assert!(!deduplicator.has(input1).await, "First input should be unique");
        assert!(!deduplicator.has(input2).await, "Second input should be unique");
        assert!(!deduplicator.has(input3).await, "Third input should be unique");
        assert!(deduplicator.has(input3).await, "Duplicate input should be detected");
        assert!(!deduplicator.has(input4).await, "Fourth unique input should be accepted");

        // Now the first input should be out of the sliding window
        assert!(deduplicator.has(input1).await, "First input should be evicted");
        assert!(deduplicator.has(input2).await, "Second input should be evicted");
        assert!(deduplicator.has(input3).await, "Third input should be evicted");
        assert!(deduplicator.has(input4).await, "Third input should be evicted");
    }
}
