use std::collections::hash_map::DefaultHasher;
use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};

pub struct Deduplicator {
    seen_hashes: HashSet<u64>,
    recent_hashes: VecDeque<u64>,
    max_recent: usize,
}

impl Deduplicator {
    pub fn new(max_recent: usize) -> Self {
        Deduplicator {
            seen_hashes: HashSet::new(),
            recent_hashes: VecDeque::new(),
            max_recent,
        }
    }

    fn hash_string(data: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    pub fn check(&mut self, new_string: &str) -> bool {
        let hash = Deduplicator::hash_string(new_string);

        if self.seen_hashes.contains(&hash) {
            // The string is a duplicate
            false
        } else {
            // Add the new hash to the hash set and deque
            self.seen_hashes.insert(hash);
            self.recent_hashes.push_back(hash);

            // Ensure we only keep the last `max_recent` hashes
            if self.recent_hashes.len() > self.max_recent {
                if let Some(removed) = self.recent_hashes.pop_front() {
                    self.seen_hashes.remove(&removed);
                }
            }

            // The string is unique
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;
    use tracing::info;

    use super::*;

    #[test(test)]
    fn test_hash_string_consistency() {
        let input = "hello";
        let hash1 = Deduplicator::hash_string(input);
        let hash2 = Deduplicator::hash_string(input);
        assert_eq!(hash1, hash2, "Hashes should be consistent for the same input");
        info!("Hash for '{}': {}", input, hash1)
    }

    #[test(test)]
    fn test_hash_string_uniqueness() {
        let input1 = "hello";
        let input2 = "world";
        let hash1 = Deduplicator::hash_string(input1);
        let hash2 = Deduplicator::hash_string(input2);
        assert_ne!(hash1, hash2, "Hashes should be different for different inputs");
    }

    #[test(test)]
    fn test_deduplicator() {
        let mut deduplicator = Deduplicator::new(3);

        let input1 = "hello";
        let input2 = "world";
        let input3 = "rust";
        let input4 = "hello"; // Duplicate

        assert!(deduplicator.check(input1), "First input should be unique");
        assert!(deduplicator.check(input2), "Second input should be unique");
        assert!(deduplicator.check(input3), "Third input should be unique");
        assert!(!deduplicator.check(input4), "Duplicate input should be detected");

        // Adding more inputs to test sliding window
        let input5 = "programming";
        assert!(deduplicator.check(input5), "Fourth unique input should be accepted");

        // Now the first input should be out of the sliding window
        assert!(
            deduplicator.check(input1),
            "First input should be unique again after sliding window moves"
        );
    }
}
