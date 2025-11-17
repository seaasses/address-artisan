use crate::constants::NON_HARDENED_MAX_INDEX;

pub trait PathWalker {
    type Iterator: Iterator<Item = [u32; 6]>;
    fn iter_from_counter(&self, start_counter: u64, chunk_size: u64) -> Self::Iterator;
}

pub struct ExtendedPublicKeyPathWalker {
    seed0: u32,
    seed1: u32,
    max_depth: u32,
}

impl ExtendedPublicKeyPathWalker {
    pub fn new(seed0: u32, seed1: u32, max_depth: u32) -> Self {
        Self {
            seed0,
            seed1,
            max_depth,
        }
    }
}

impl PathWalker for ExtendedPublicKeyPathWalker {
    type Iterator = PathIterator;

    fn iter_from_counter(&self, start_counter: u64, chunk_size: u64) -> PathIterator {
        PathIterator {
            seed0: self.seed0,
            seed1: self.seed1,
            max_depth: self.max_depth,
            current_counter: start_counter,
            end_counter: start_counter + chunk_size,
        }
    }
}

pub struct PathIterator {
    seed0: u32,
    seed1: u32,
    max_depth: u32,
    current_counter: u64,
    end_counter: u64,
}

impl PathIterator {
    fn counter_to_path(&self, counter: u64) -> [u32; 6] {
        let index = (counter % self.max_depth as u64) as u32;
        let a = ((counter / self.max_depth as u64) % NON_HARDENED_MAX_INDEX as u64) as u32;
        let b = (counter / (self.max_depth as u64 * NON_HARDENED_MAX_INDEX as u64)) as u32;

        [self.seed0, self.seed1, b, a, 0, index]
    }
}

impl Iterator for PathIterator {
    type Item = [u32; 6];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_counter >= self.end_counter {
            return None;
        }

        let path = self.counter_to_path(self.current_counter);
        self.current_counter += 1;
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_iteration() {
        let walker = ExtendedPublicKeyPathWalker::new(1000, 2000, 100);

        let paths: Vec<[u32; 6]> = walker.iter_from_counter(0, 3).collect();

        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], [1000, 2000, 0, 0, 0, 0]);
        assert_eq!(paths[1], [1000, 2000, 0, 0, 0, 1]);
        assert_eq!(paths[2], [1000, 2000, 0, 0, 0, 2]);
    }

    #[test]
    fn test_counter_to_path_basic() {
        let walker = ExtendedPublicKeyPathWalker::new(1000, 2000, 10000);

        let mut iter = walker.iter_from_counter(0, 1);
        assert_eq!(iter.next(), Some([1000, 2000, 0, 0, 0, 0]));

        let mut iter = walker.iter_from_counter(5000, 1);
        assert_eq!(iter.next(), Some([1000, 2000, 0, 0, 0, 5000]));

        let mut iter = walker.iter_from_counter(10000, 1);
        assert_eq!(iter.next(), Some([1000, 2000, 0, 1, 0, 0]));
    }

    #[test]
    fn test_chunk_iteration() {
        let walker = ExtendedPublicKeyPathWalker::new(100, 200, 10);

        let paths: Vec<[u32; 6]> = walker.iter_from_counter(5, 3).collect();

        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], [100, 200, 0, 0, 0, 5]);
        assert_eq!(paths[1], [100, 200, 0, 0, 0, 6]);
        assert_eq!(paths[2], [100, 200, 0, 0, 0, 7]);
    }

    #[test]
    fn test_a_increments() {
        let walker = ExtendedPublicKeyPathWalker::new(0, 0, 100);

        let mut iter = walker.iter_from_counter(100, 1);
        assert_eq!(iter.next(), Some([0, 0, 0, 1, 0, 0]));

        let mut iter = walker.iter_from_counter(200, 1);
        assert_eq!(iter.next(), Some([0, 0, 0, 2, 0, 0]));
    }

    #[test]
    fn test_fixed_zero_at_position_4() {
        let walker = ExtendedPublicKeyPathWalker::new(123, 456, 1000);

        for counter in [0, 100, 1000, 10000, 1_000_000] {
            let mut iter = walker.iter_from_counter(counter, 1);
            let path = iter.next().unwrap();
            assert_eq!(
                path[4], 0,
                "Position 4 should always be 0 for BIP49 compatibility"
            );
        }
    }

    #[test]
    fn test_max_depth_boundary() {
        let walker = ExtendedPublicKeyPathWalker::new(0, 0, 100);

        // Just before max_depth
        let mut iter = walker.iter_from_counter(99, 1);
        assert_eq!(iter.next(), Some([0, 0, 0, 0, 0, 99]));

        // At max_depth - should roll over to a=1, index=0
        let mut iter = walker.iter_from_counter(100, 1);
        assert_eq!(iter.next(), Some([0, 0, 0, 1, 0, 0]));

        // One after max_depth
        let mut iter = walker.iter_from_counter(101, 1);
        assert_eq!(iter.next(), Some([0, 0, 0, 1, 0, 1]));
    }

    #[test]
    fn test_continuous_iteration_across_boundary() {
        let walker = ExtendedPublicKeyPathWalker::new(0, 0, 10);

        // Start at 8, iterate across the boundary
        let paths: Vec<[u32; 6]> = walker.iter_from_counter(8, 5).collect();

        assert_eq!(paths.len(), 5);
        assert_eq!(paths[0], [0, 0, 0, 0, 0, 8]); // counter=8
        assert_eq!(paths[1], [0, 0, 0, 0, 0, 9]); // counter=9
        assert_eq!(paths[2], [0, 0, 0, 1, 0, 0]); // counter=10, a increments!
        assert_eq!(paths[3], [0, 0, 0, 1, 0, 1]); // counter=11
        assert_eq!(paths[4], [0, 0, 0, 1, 0, 2]); // counter=12
    }

    #[test]
    fn test_a_increments_correctly() {
        let walker = ExtendedPublicKeyPathWalker::new(0, 0, 100);

        // Start at a=1000, index=0
        let counter_start = 1000 * 100; // a=1000, index=0
        let paths: Vec<[u32; 6]> = walker.iter_from_counter(counter_start, 150).collect();

        // First 100 should have a=1000, index 0-99
        for (i, path) in paths.iter().enumerate().take(100) {
            assert_eq!(path[3], 1000, "At position {}", i);
            assert_eq!(path[5], i as u32, "At position {}", i);
        }

        // Next 50 should have a=1001, index 0-49
        for (i, path) in paths.iter().enumerate().skip(100).take(50) {
            assert_eq!(path[3], 1001, "At position {}", i);
            assert_eq!(path[5], (i - 100) as u32, "At position {}", i);
        }
    }

    #[test]
    fn test_large_counter() {
        let walker = ExtendedPublicKeyPathWalker::new(1000, 2000, 10000);

        // Large counter that should set both a and b
        let counter = 5_000_000_000u64;

        let mut iter = walker.iter_from_counter(counter, 1);
        let path = iter.next().unwrap();

        // Verify structure: [seed0, seed1, b, a, 0, index]
        assert_eq!(path[0], 1000); // seed0
        assert_eq!(path[1], 2000); // seed1
        assert_eq!(path[4], 0); // fixed 0

        // Verify decomposition - NOW WITH CORRECT ORDER
        let expected_index = (counter % 10000) as u32;
        let expected_a = ((counter / 10000) % 0x7FFFFFFF) as u32;
        let expected_b = (counter / (10000 * 0x7FFFFFFF)) as u32;

        assert_eq!(path[5], expected_index);
        assert_eq!(path[3], expected_a);
        assert_eq!(path[2], expected_b);
    }

    #[test]
    fn test_seeds_preserved() {
        let walker = ExtendedPublicKeyPathWalker::new(12345, 67890, 100);

        for counter in [0, 99, 100, 1000, 1_000_000] {
            let mut iter = walker.iter_from_counter(counter, 1);
            let path = iter.next().unwrap();
            assert_eq!(path[0], 12345, "seed0 should be preserved");
            assert_eq!(path[1], 67890, "seed1 should be preserved");
        }
    }

    #[test]
    fn test_iterator_stops_at_chunk_size() {
        let walker = ExtendedPublicKeyPathWalker::new(0, 0, 100);

        let paths: Vec<[u32; 6]> = walker.iter_from_counter(0, 5).collect();

        assert_eq!(paths.len(), 5, "Should stop exactly at chunk_size");

        // Verify no extra iterations
        let mut iter = walker.iter_from_counter(0, 3);
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_none()); // Should be None after chunk_size
    }

    #[test]
    fn test_path_always_6_levels() {
        let walker = ExtendedPublicKeyPathWalker::new(100, 200, 1000);

        for counter in [0, 500, 999, 1000, 5000, 1_000_000, 5_000_000_000] {
            let mut iter = walker.iter_from_counter(counter, 1);
            let path = iter.next().unwrap();
            assert_eq!(
                path.len(),
                6,
                "Path should always have exactly 6 levels at counter {}",
                counter
            );
        }
    }

    #[test]
    fn test_bip32_limits_never_exceeded() {
        let walker = ExtendedPublicKeyPathWalker::new(0, 0, 10000);
        let max_bip32 = 0x7FFFFFFF;

        // Test various large counters
        for counter in [
            1_000_000u64,
            100_000_000,
            1_000_000_000,
            10_000_000_000,
            100_000_000_000,
        ] {
            let mut iter = walker.iter_from_counter(counter, 1);
            let path = iter.next().unwrap();

            // Check all levels respect BIP32 limit
            for (i, &level) in path.iter().enumerate() {
                assert!(
                    level <= max_bip32,
                    "Level {} (value {}) exceeds BIP32 limit at counter {}",
                    i,
                    level,
                    counter
                );
            }
        }
    }
}
