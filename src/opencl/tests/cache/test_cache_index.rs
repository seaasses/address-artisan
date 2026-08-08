#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    /// Non-hardened BIP32 indexes go up to this value; `a` is always <= it.
    const MAX_A: u32 = 0x7FFFFFFF;
    const HARNESS_CAPACITY: usize = 4096;

    struct CacheIndexHarness {
        query_b_buffer: Buffer<u32>,
        query_a_buffer: Buffer<u32>,
        result_index_buffer: Buffer<u32>,
        result_found_buffer: Buffer<i32>,
        kernel: Kernel,
    }

    impl CacheIndexHarness {
        fn new() -> Result<Self, String> {
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            let query_b_buffer = Self::new_buffer::<u32>(&queue, HARNESS_CAPACITY)?;
            let query_a_buffer = Self::new_buffer::<u32>(&queue, HARNESS_CAPACITY)?;
            let result_index_buffer = Self::new_buffer::<u32>(&queue, HARNESS_CAPACITY)?;
            let result_found_buffer = Self::new_buffer::<i32>(&queue, HARNESS_CAPACITY)?;

            let src = include_str!(concat!(env!("OUT_DIR"), "/cache_index_kernel"));
            let program = Program::builder()
                .src(src)
                .devices(device)
                .build(&context)
                .map_err(|e| format!("Error building OpenCL program: {}", e))?;

            let kernel = Kernel::builder()
                .program(&program)
                .name("cache_index_kernel")
                .queue(queue.clone())
                .global_work_size(HARNESS_CAPACITY)
                .arg(&query_b_buffer)
                .arg(&query_a_buffer)
                .arg(0u32) // first_b
                .arg(0u32) // first_a
                .arg(0u32) // cache_size
                .arg(0u32) // query_count
                .arg(&result_index_buffer)
                .arg(&result_found_buffer)
                .build()
                .map_err(|e| format!("Error creating kernel: {}", e))?;

            Ok(Self {
                query_b_buffer,
                query_a_buffer,
                result_index_buffer,
                result_found_buffer,
                kernel,
            })
        }

        /// Runs the kernel for all queries against a cache starting at
        /// `first` with `cache_size` entries. Returns (index, found) pairs.
        fn run(
            &mut self,
            queries: &[(u32, u32)],
            first: (u32, u32),
            cache_size: u32,
        ) -> Result<Vec<(u32, bool)>, String> {
            assert!(queries.len() <= HARNESS_CAPACITY);

            let query_b: Vec<u32> = queries.iter().map(|q| q.0).collect();
            let query_a: Vec<u32> = queries.iter().map(|q| q.1).collect();

            self.query_b_buffer
                .write(&query_b)
                .enq()
                .map_err(|e| format!("Error writing query_b: {}", e))?;
            self.query_a_buffer
                .write(&query_a)
                .enq()
                .map_err(|e| format!("Error writing query_a: {}", e))?;

            self.kernel
                .set_arg(2, first.0)
                .map_err(|e| format!("Error setting first_b: {}", e))?;
            self.kernel
                .set_arg(3, first.1)
                .map_err(|e| format!("Error setting first_a: {}", e))?;
            self.kernel
                .set_arg(4, cache_size)
                .map_err(|e| format!("Error setting cache_size: {}", e))?;
            self.kernel
                .set_arg(5, queries.len() as u32)
                .map_err(|e| format!("Error setting query_count: {}", e))?;

            unsafe {
                self.kernel
                    .enq()
                    .map_err(|e| format!("Error executing kernel: {}", e))?;
            }

            let mut indexes = vec![0u32; HARNESS_CAPACITY];
            let mut found = vec![0i32; HARNESS_CAPACITY];
            self.result_index_buffer
                .read(&mut indexes)
                .enq()
                .map_err(|e| format!("Error reading indexes: {}", e))?;
            self.result_found_buffer
                .read(&mut found)
                .enq()
                .map_err(|e| format!("Error reading found flags: {}", e))?;

            Ok(queries
                .iter()
                .enumerate()
                .map(|(i, _)| (indexes[i], found[i] != 0))
                .collect())
        }

        fn new_buffer<T: ocl::OclPrm>(queue: &Queue, len: usize) -> Result<Buffer<T>, String> {
            Buffer::<T>::builder()
                .queue(queue.clone())
                .len(len)
                .build()
                .map_err(|e| format!("Error creating buffer: {}", e))
        }

        fn get_device_context_and_queue() -> Result<(Device, Context, Queue), String> {
            let platform =
                Platform::first().map_err(|e| format!("Error getting OpenCL platform: {}", e))?;
            let device = Device::first(platform)
                .map_err(|e| format!("Error getting OpenCL device: {}", e))?;
            let context = Context::builder()
                .platform(platform)
                .devices(device)
                .build()
                .map_err(|e| format!("Error creating OpenCL context: {}", e))?;
            let queue = Queue::new(&context, device, None)
                .map_err(|e| format!("Error creating OpenCL queue: {}", e))?;
            Ok((device, context, queue))
        }
    }

    // ===================== Reference implementation =====================
    //
    // The semantic ground truth, independent from the ordinal formula used
    // by the kernel: explicitly build the contiguous key list (the same way
    // CacheRangeAnalyzer::next_cache_key walks it) and scan for the query.

    fn next_key(key: (u32, u32)) -> (u32, u32) {
        if key.1 == MAX_A {
            (key.0 + 1, 0)
        } else {
            (key.0, key.1 + 1)
        }
    }

    fn build_keys(first: (u32, u32), cache_size: u32) -> Vec<(u32, u32)> {
        let mut keys = Vec::with_capacity(cache_size as usize);
        let mut current = first;
        for _ in 0..cache_size {
            keys.push(current);
            current = next_key(current);
        }
        keys
    }

    fn reference_lookup(query: (u32, u32), first: (u32, u32), cache_size: u32) -> Option<u32> {
        build_keys(first, cache_size)
            .iter()
            .position(|k| *k == query)
            .map(|i| i as u32)
    }

    fn assert_matches_reference(
        harness: &mut CacheIndexHarness,
        queries: &[(u32, u32)],
        first: (u32, u32),
        cache_size: u32,
        context: &str,
    ) {
        let results = harness.run(queries, first, cache_size).unwrap();
        for (query, (index, found)) in queries.iter().zip(results.iter()) {
            match reference_lookup(*query, first, cache_size) {
                Some(expected_index) => {
                    assert!(
                        *found,
                        "{}: query {:?} (first {:?}, size {}) should be found",
                        context, query, first, cache_size
                    );
                    assert_eq!(
                        *index, expected_index,
                        "{}: query {:?} (first {:?}, size {}) wrong index",
                        context, query, first, cache_size
                    );
                }
                None => {
                    assert!(
                        !*found,
                        "{}: query {:?} (first {:?}, size {}) should NOT be found",
                        context, query, first, cache_size
                    );
                }
            }
        }
    }

    // ========================= Deterministic cases =========================

    #[test]
    fn test_first_key_is_index_0() {
        let mut harness = CacheIndexHarness::new().unwrap();
        let results = harness.run(&[(7, 42)], (7, 42), 10).unwrap();
        assert_eq!(results[0], (0, true));
    }

    #[test]
    fn test_sequential_keys_map_to_sequential_indexes() {
        let mut harness = CacheIndexHarness::new().unwrap();
        let first = (3, 1000);
        let cache_size = 100u32;
        let queries = build_keys(first, cache_size);

        let results = harness.run(&queries, first, cache_size).unwrap();
        for (i, (index, found)) in results.iter().enumerate() {
            assert!(*found, "key {} should be found", i);
            assert_eq!(*index, i as u32, "key {} has wrong index", i);
        }
    }

    #[test]
    fn test_last_key_and_one_past_last() {
        let mut harness = CacheIndexHarness::new().unwrap();
        let first = (5, 500);
        let cache_size = 42u32;
        let last = build_keys(first, cache_size)[cache_size as usize - 1];
        let one_past = next_key(last);

        let results = harness.run(&[last, one_past], first, cache_size).unwrap();
        assert_eq!(results[0], (cache_size - 1, true), "last key");
        assert!(!results[1].1, "one past last must not be found");
    }

    #[test]
    fn test_keys_before_first_are_not_found() {
        let mut harness = CacheIndexHarness::new().unwrap();
        let first = (5, 500);
        let queries = [
            (5, 499),   // immediately before
            (5, 0),     // same b, much smaller a
            (4, 500),   // smaller b, same a
            (4, MAX_A), // ordinal immediately before (5, 0)
            (0, 0),     // far before
            (4, 501),   // smaller b, larger a (ordinal still smaller)
        ];
        let results = harness.run(&queries, first, 1000).unwrap();
        for (query, (_, found)) in queries.iter().zip(results.iter()) {
            assert!(!found, "query {:?} is before the range", query);
        }
    }

    #[test]
    fn test_boundary_rollover_a_to_b() {
        let mut harness = CacheIndexHarness::new().unwrap();

        // Cache crosses the a -> b rollover: [(9, MAX-1), (9, MAX), (10, 0), (10, 1)]
        let first = (9, MAX_A - 1);
        let cache_size = 4u32;

        let queries = [
            (9, MAX_A - 1),
            (9, MAX_A),
            (10, 0),
            (10, 1),
            (10, 2),        // one past last
            (9, MAX_A - 2), // one before first
        ];
        let results = harness.run(&queries, first, cache_size).unwrap();

        assert_eq!(results[0], (0, true));
        assert_eq!(results[1], (1, true));
        assert_eq!(results[2], (2, true));
        assert_eq!(results[3], (3, true));
        assert!(!results[4].1);
        assert!(!results[5].1);
    }

    #[test]
    fn test_cache_starting_exactly_at_rollover() {
        let mut harness = CacheIndexHarness::new().unwrap();

        // First key is exactly (b, 0), the entry right after a rollover
        let first = (123, 0);
        let results = harness
            .run(&[(123, 0), (122, MAX_A), (123, 1)], first, 2)
            .unwrap();
        assert_eq!(results[0], (0, true));
        assert!(!results[1].1, "(122, MAX_A) is the ordinal before (123, 0)");
        assert_eq!(results[2], (1, true));
    }

    #[test]
    fn test_cache_size_one() {
        let mut harness = CacheIndexHarness::new().unwrap();
        let first = (2, 3);
        let results = harness.run(&[(2, 3), (2, 4), (2, 2)], first, 1).unwrap();
        assert_eq!(results[0], (0, true));
        assert!(!results[1].1);
        assert!(!results[2].1);
    }

    #[test]
    fn test_cache_size_zero_finds_nothing() {
        let mut harness = CacheIndexHarness::new().unwrap();
        let results = harness.run(&[(0, 0), (1, 1)], (0, 0), 0).unwrap();
        assert!(!results[0].1);
        assert!(!results[1].1);
    }

    #[test]
    fn test_max_a_value_is_valid_key() {
        let mut harness = CacheIndexHarness::new().unwrap();
        let first = (0, MAX_A);
        let results = harness.run(&[(0, MAX_A), (1, 0)], first, 2).unwrap();
        assert_eq!(results[0], (0, true));
        assert_eq!(results[1], (1, true));
    }

    #[test]
    fn test_huge_b_values() {
        let mut harness = CacheIndexHarness::new().unwrap();

        // Ordinals near the top of the (b, a) space must not overflow
        let first = (u32::MAX - 1, MAX_A - 5);
        let cache_size = 10u32; // crosses into b = u32::MAX
        let queries = build_keys(first, cache_size);

        assert_matches_reference(&mut harness, &queries, first, cache_size, "huge_b");

        // And out-of-range queries around it
        let last = queries[cache_size as usize - 1];
        let results = harness
            .run(&[next_key(last), (0, 0)], first, cache_size)
            .unwrap();
        assert!(!results[0].1);
        assert!(!results[1].1);
    }

    #[test]
    fn test_query_far_after_range() {
        let mut harness = CacheIndexHarness::new().unwrap();
        let first = (10, 0);
        let queries = [
            (10, 5000),        // same b, past the range
            (11, 0),           // next b
            (u32::MAX, MAX_A), // far away
        ];
        let results = harness.run(&queries, first, 100).unwrap();
        for (query, (_, found)) in queries.iter().zip(results.iter()) {
            assert!(!found, "query {:?} is after the range", query);
        }
    }

    #[test]
    fn test_large_cache_spot_checks() {
        let mut harness = CacheIndexHarness::new().unwrap();

        // A cache the size the GPU actually uses per batch at low max_depth
        let first = (0, 12345);
        let cache_size = 600u32;

        // Spot-check every 37th key plus the extremes
        let keys = build_keys(first, cache_size);
        let mut queries: Vec<(u32, u32)> = keys.iter().step_by(37).copied().collect();
        queries.push(keys[0]);
        queries.push(keys[keys.len() - 1]);

        let results = harness.run(&queries, first, cache_size).unwrap();
        for (query, (index, found)) in queries.iter().zip(results.iter()) {
            let expected = keys.iter().position(|k| k == query).unwrap() as u32;
            assert!(*found, "query {:?} should be found", query);
            assert_eq!(*index, expected, "query {:?} wrong index", query);
        }
    }

    // ========================= Property-based tests =========================

    #[test]
    fn test_random_scenarios_match_reference_scan() {
        let mut harness = CacheIndexHarness::new().unwrap();

        for scenario in 0..60 {
            // Random first key (b bounded to leave room for rollovers)
            let first_b = rand::random::<u32>() % 1_000_000;
            let first_a = rand::random::<u32>() % (MAX_A + 1);
            let first = (first_b, first_a);
            let cache_size = 1 + rand::random::<u32>() % 512;

            let keys = build_keys(first, cache_size);

            let mut queries: Vec<(u32, u32)> = Vec::new();
            // In-range queries
            for _ in 0..32 {
                queries.push(keys[rand::random::<u32>() as usize % keys.len()]);
            }
            // Out-of-range queries: shifted before/after by random offsets
            for _ in 0..16 {
                let offset = 1 + rand::random::<u32>() % 10_000;
                let target = if rand::random::<bool>() {
                    // after the last key
                    let mut k = keys[keys.len() - 1];
                    for _ in 0..(offset % 64) + 1 {
                        k = next_key(k);
                    }
                    k
                } else {
                    // random key with random a (usually far away)
                    (
                        rand::random::<u32>() % 1_000_100,
                        rand::random::<u32>() % (MAX_A + 1),
                    )
                };
                queries.push(target);
            }

            assert_matches_reference(
                &mut harness,
                &queries,
                first,
                cache_size,
                &format!("scenario {}", scenario),
            );
        }
    }

    #[test]
    fn test_random_rollover_scenarios() {
        let mut harness = CacheIndexHarness::new().unwrap();

        // Force every scenario to cross the a -> b rollover
        for scenario in 0..30 {
            let first_b = rand::random::<u32>() % 1_000_000;
            let back = 1 + rand::random::<u32>() % 40; // keys before the rollover
            let size = back + 1 + rand::random::<u32>() % 40; // and after it
            let first = (first_b, MAX_A - back + 1);

            let keys = build_keys(first, size);
            assert!(
                keys.iter().any(|k| k.0 == first_b + 1),
                "scenario must cross the rollover"
            );

            assert_matches_reference(
                &mut harness,
                &keys,
                first,
                size,
                &format!("rollover scenario {}", scenario),
            );
        }
    }
}
