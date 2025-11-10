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
        let mut cache_keys = Vec::new();
        let mut current = first;

        // Guard against generating too many keys
        const MAX_REASONABLE_KEYS: usize = 100_000_000; // 100M max

        // Iterate from first to last (inclusive)
        loop {
            cache_keys.push(current);

            if current == last {
                break;
            }

            current = Self::next_cache_key(current);

            // Safety check
            if cache_keys.len() > MAX_REASONABLE_KEYS {
                panic!(
                    "Cache range analyzer would generate more than {} million keys! \
                     This suggests max_depth is too large or batch_size is absurdly huge. \
                     first=[{}, {}], last=[{}, {}]",
                    MAX_REASONABLE_KEYS / 1_000_000,
                    first[0], first[1],
                    last[0], last[1]
                );
            }
        }

        cache_keys
    }

    /// Get the next cache key in sequence
    fn next_cache_key(current: [u32; 2]) -> [u32; 2] {
        let [b, a] = current;
        let non_hardened_count = NON_HARDENED_MAX_INDEX as u64 + 1;

        let new_a = a as u64 + 1;
        let new_b = b + (new_a / non_hardened_count) as u32;
        let new_a = (new_a % non_hardened_count) as u32;

        [new_b, new_a]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Helper function to compare cache keys as sets (order doesn't matter)
    fn assert_cache_keys_match(actual: Vec<[u32; 2]>, expected: Vec<[u32; 2]>) {
        let actual_set: HashSet<[u32; 2]> = actual.into_iter().collect();
        let expected_set: HashSet<[u32; 2]> = expected.into_iter().collect();

        assert_eq!(
            actual_set.len(),
            expected_set.len(),
            "Should have {} cache keys, but got {}",
            expected_set.len(),
            actual_set.len()
        );

        assert_eq!(
            actual_set, expected_set,
            "Cache keys should match expected values (order doesn't matter)"
        );
    }

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

    #[test]
    fn test_vector_1_max_depth_1() {
        let keys = CacheRangeAnalyzer::analyze_counter_range(0, 10, 1);

        let expected = vec![
            [0, 0], [0, 1], [0, 2], [0, 3], [0, 4],
            [0, 5], [0, 6], [0, 7], [0, 8], [0, 9]
        ];

        assert_eq!(keys.len(), 10, "Should have exactly 10 cache keys");
        assert_cache_keys_match(keys, expected);
    }

    #[test]
    fn test_vector_2_near_max_index() {
        let keys = CacheRangeAnalyzer::analyze_counter_range(2147483638, 10, 1);

        let expected = vec![
            [0, 2147483638], [0, 2147483639], [0, 2147483640],
            [0, 2147483641], [0, 2147483642], [0, 2147483643],
            [0, 2147483644], [0, 2147483645], [0, 2147483646],
            [0, 2147483647]
        ];

        assert_eq!(keys.len(), 10, "Should have exactly 10 cache keys");
        assert_cache_keys_match(keys, expected);
    }

    #[test]
    fn test_vector_3_crossing_b_boundary() {
        let keys = CacheRangeAnalyzer::analyze_counter_range(2147483638, 11, 1);

        let expected = vec![
            [0, 2147483638], [0, 2147483639], [0, 2147483640],
            [0, 2147483641], [0, 2147483642], [0, 2147483643],
            [0, 2147483644], [0, 2147483645], [0, 2147483646],
            [0, 2147483647], [1, 0]
        ];

        assert_eq!(keys.len(), 11, "Should have exactly 11 cache keys");
        assert_cache_keys_match(keys, expected);
    }

    #[test]
    fn test_vector_4_large_max_depth() {
        let keys = CacheRangeAnalyzer::analyze_counter_range(2147483638, 11, 100000);
        let expected = vec![[0, 21474]];
        assert_eq!(keys.len(), 1, "Should have exactly 1 cache key");
        assert_cache_keys_match(keys, expected);
    }

    #[test]
    fn test_vector_5_large_counter_small_depth() {
        let keys = CacheRangeAnalyzer::analyze_counter_range(1152921504606846966, 1000, 123);

        let expected = vec![
            [4364804, 349184332], [4364804, 349184333], [4364804, 349184334],
            [4364804, 349184335], [4364804, 349184336], [4364804, 349184337],
            [4364804, 349184338], [4364804, 349184339], [4364804, 349184340],
            [4364804, 349184341]
        ];

        assert_eq!(keys.len(), 10, "Should have exactly 10 cache keys");
        assert_cache_keys_match(keys, expected);
    }

    #[test]
    fn test_vector_6_large_counter_large_count() {
        let keys = CacheRangeAnalyzer::analyze_counter_range(1152921504606846966, 10000000, 123456);

        let mut expected = vec![];
        for a in 1465053644..=1465053725 {
            expected.push([4348, a]);
        }

        assert_eq!(keys.len(), 82, "Should have exactly 82 cache keys");
        assert_eq!(expected.len(), 82);
        assert_cache_keys_match(keys, expected);
    }

    #[test]
    fn test_vector_7_crossing_boundary_large_values() {
        let keys = CacheRangeAnalyzer::analyze_counter_range(1325598705305344, 1000000, 123456);

        let expected = vec![
            [4, 2147483640], [4, 2147483641], [4, 2147483642],
            [4, 2147483643], [4, 2147483644], [4, 2147483645],
            [4, 2147483646], [4, 2147483647], [5, 0]
        ];

        assert_eq!(keys.len(), 9, "Should have exactly 9 cache keys");
        assert_cache_keys_match(keys, expected);
    }

    #[test]
    fn test_result_size_guarantees() {
        // Test that result size matches expected size for various scenarios

        // Single key when range fits in one cache
        let keys1 = CacheRangeAnalyzer::analyze_counter_range(0, 1000, 10000);
        assert_eq!(keys1.len(), 1, "Small range should produce 1 key");

        // Multiple keys when range spans multiple caches
        let keys2 = CacheRangeAnalyzer::analyze_counter_range(0, 100000, 1000);
        assert_eq!(keys2.len(), 100, "Should produce exactly 100 keys");

        // Edge case: single counter
        let keys3 = CacheRangeAnalyzer::analyze_counter_range(12345, 1, 100);
        assert_eq!(keys3.len(), 1, "Single counter should produce 1 key");

        // Large max_depth reduces number of keys needed
        let keys4 = CacheRangeAnalyzer::analyze_counter_range(0, 1000000, 1000000);
        assert_eq!(keys4.len(), 1, "Large max_depth should produce 1 key");

        // Small max_depth increases number of keys needed
        let keys5 = CacheRangeAnalyzer::analyze_counter_range(0, 100, 1);
        assert_eq!(keys5.len(), 100, "max_depth=1 should produce 100 keys for 100 counters");
    }
}
