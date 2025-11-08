use crate::constants::NON_HARDENED_MAX_INDEX;

pub struct CacheRangeAnalyzer;

impl CacheRangeAnalyzer {
    pub fn analyze_counter_range(start_counter: u64, count: u64, max_depth: u32) -> Vec<[u32; 2]> {
        if count == 0 {
            return vec![];
        }

        let first = Self::counter_to_cache_key(start_counter, max_depth);
        let last = Self::counter_to_cache_key(start_counter + count - 1, max_depth);

        Self::calculate_required_caches(first, last)
    }

    /// Convert counter to [b, a] cache key
    fn counter_to_cache_key(counter: u64, max_depth: u32) -> [u32; 2] {
        let non_hardened_count = NON_HARDENED_MAX_INDEX as u64 + 1; // we do %, so +1

        let a = ((counter / max_depth as u64) % non_hardened_count) as u32;
        let b = (counter / (max_depth as u64 * non_hardened_count)) as u32;

        [b, a]
    }

    /// Calculate all [b, a] keys between first and last
    fn calculate_required_caches(first: [u32; 2], last: [u32; 2]) -> Vec<[u32; 2]> {
        let (b_start, a_start) = (first[0], first[1]);
        let (b_end, a_end) = (last[0], last[1]);

        let mut cache_keys = Vec::new();

        // Guard against generating billions of keys when b changes
        const MAX_REASONABLE_KEYS: usize = 100_000_000; // 100M max

        if b_start == b_end {
            // Same b - just iterate a values
            for a in a_start..=a_end {
                cache_keys.push([b_start, a]);
            }
        } else {
            // Different b values - this should be EXTREMELY rare with reasonable batch sizes
            // Calculate how many keys would be generated
            let keys_in_first_b = (NON_HARDENED_MAX_INDEX - a_start + 1) as u64;
            let keys_in_last_b = (a_end + 1) as u64;
            let keys_in_middle_bs =
                (b_end.saturating_sub(b_start).saturating_sub(1)) as u64 * (NON_HARDENED_MAX_INDEX as u64 + 1);
            let total_keys = keys_in_first_b + keys_in_middle_bs + keys_in_last_b;

            if total_keys > MAX_REASONABLE_KEYS as u64 {
                panic!(
                    "Cache range analyzer would generate {} billion keys! This suggests max_depth is too large \
                     or batch_size is absurdly huge. first=[{}, {}], last=[{}, {}]",
                    total_keys / 1_000_000_000,
                    b_start,
                    a_start,
                    b_end,
                    a_end
                );
            }

            // Generate keys (we already validated it won't be too many)
            for b in b_start..=b_end {
                let a_min = if b == b_start { a_start } else { 0 };
                let a_max = if b == b_end {
                    a_end
                } else {
                    NON_HARDENED_MAX_INDEX
                };

                for a in a_min..=a_max {
                    cache_keys.push([b, a]);
                }
            }
        }

        cache_keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_range() {
        let keys = CacheRangeAnalyzer::analyze_counter_range(0, 0, 10_000);
        assert_eq!(keys.len(), 0);
    }

    #[test]
    fn test_single_cache_needed() {
        let keys = CacheRangeAnalyzer::analyze_counter_range(0, 100, 10_000);
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], [0, 0]);
    }

    #[test]
    fn test_same_b_multiple_a_values() {
        let keys = CacheRangeAnalyzer::analyze_counter_range(0, 250_000, 10_000);
        assert_eq!(keys.len(), 25);
        assert_eq!(keys[0], [0, 0]);
        assert_eq!(keys[24], [0, 24]);

        for i in 0..25 {
            assert_eq!(keys[i], [0, i as u32]);
        }
    }

    #[test]
    fn test_multiple_b_values() {
        let max_depth = 10_000u32;
        let counters_per_b = (max_depth as u64) * (NON_HARDENED_MAX_INDEX as u64 + 1);

        let keys = CacheRangeAnalyzer::analyze_counter_range(counters_per_b - 100, 200, max_depth);

        assert!(keys.iter().any(|k| k[0] == 0));
        assert!(keys.iter().any(|k| k[0] == 1));
    }

    #[test]
    fn test_middle_range() {
        let keys = CacheRangeAnalyzer::analyze_counter_range(50_000, 100_000, 10_000);

        assert_eq!(keys.len(), 10);
        assert_eq!(keys[0][1], 5);
        assert_eq!(keys[9][1], 14);
    }

    #[test]
    fn test_large_range_efficiency() {
        let start = std::time::Instant::now();
        let keys = CacheRangeAnalyzer::analyze_counter_range(0, 100_000_000, 10_000);
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 100,
            "Should be fast even for huge ranges"
        );
        println!("Calculated {} caches in {:?}", keys.len(), elapsed);
    }

    #[test]
    fn test_counter_to_cache_key() {
        let max_depth = 10_000;

        let key0 = CacheRangeAnalyzer::counter_to_cache_key(0, max_depth);
        assert_eq!(key0, [0, 0]);

        let key1 = CacheRangeAnalyzer::counter_to_cache_key(9_999, max_depth);
        assert_eq!(key1, [0, 0]);

        let key2 = CacheRangeAnalyzer::counter_to_cache_key(10_000, max_depth);
        assert_eq!(key2, [0, 1]);

        let key3 = CacheRangeAnalyzer::counter_to_cache_key(20_000, max_depth);
        assert_eq!(key3, [0, 2]);
    }

    #[test]
    fn test_consecutive_ranges_have_unique_keys() {
        let range1 = CacheRangeAnalyzer::analyze_counter_range(0, 10_000, 10_000);
        let range2 = CacheRangeAnalyzer::analyze_counter_range(10_000, 10_000, 10_000);

        assert_eq!(range1, vec![[0, 0]]);
        assert_eq!(range2, vec![[0, 1]]);
    }
}
