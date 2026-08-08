#[cfg(test)]
mod tests {
    use crate::extended_public_key::ExtendedPubKey;
    use crate::extended_public_key_deriver::{ExtendedPublicKeyDeriver, KeyDeriver};
    use crate::opencl::cache_preloader::CachePreloader;
    use crate::opencl::g_tables;
    use crate::opencl::gpu_cache::{CacheKey, GpuCache, Hash160RangeGpu, PointGpu, XPub};
    use crate::prefix::Prefix;
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    fn create_test_opencl_context() -> (Device, Context, Queue) {
        let platform = Platform::first().expect("No OpenCL platform found");
        let device = Device::first(platform).expect("No OpenCL device found");
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()
            .expect("Failed to create context");
        let queue = Queue::new(&context, device, None).expect("Failed to create queue");
        (device, context, queue)
    }

    struct BatchAddressSearch {
        kernel: Kernel,
        cache_keys_buffer: Buffer<CacheKey>,
        cache_values_buffer: Buffer<XPub>,
        cache_size_buffer: Buffer<u32>,
        ranges_buffer: Buffer<Hash160RangeGpu>,
        matches_hash160_buffer: Buffer<u8>,
        _matches_b_buffer: Buffer<u32>, // Needed for kernel args but not read in tests
        _matches_a_buffer: Buffer<u32>, // Needed for kernel args but not read in tests
        _matches_index_buffer: Buffer<u32>, // Needed for kernel args but not read in tests
        _matches_prefix_id_buffer: Buffer<u8>, // Needed for kernel args but not read in tests
        match_count_buffer: Buffer<u32>,
        cache_miss_error_buffer: Buffer<u32>,
        _g_times_tables_buffer: Buffer<PointGpu>, // Needed for kernel args but not read in tests
        ppt: u64,
    }

    impl BatchAddressSearch {
        fn new() -> Result<Self, String> {
            Self::new_with_ppt(1)
        }

        fn new_with_ppt(ppt: u64) -> Result<Self, String> {
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            let cache_keys_buffer = Self::new_buffer::<CacheKey>(&queue, 1000)?;
            let cache_values_buffer = Self::new_buffer::<XPub>(&queue, 1000)?;
            let cache_size_buffer = Self::new_buffer::<u32>(&queue, 1)?; // Create cache size buffer
            let ranges_buffer = Self::new_buffer::<Hash160RangeGpu>(&queue, 10)?;
            let matches_hash160_buffer = Self::new_buffer::<u8>(&queue, 1000 * 20)?;
            let matches_b_buffer = Self::new_buffer::<u32>(&queue, 1000)?;
            let matches_a_buffer = Self::new_buffer::<u32>(&queue, 1000)?;
            let matches_index_buffer = Self::new_buffer::<u32>(&queue, 1000)?;
            let matches_prefix_id_buffer = Self::new_buffer::<u8>(&queue, 1000)?;
            let match_count_buffer = Self::new_buffer::<u32>(&queue, 1)?;
            let cache_miss_error_buffer = Self::new_buffer::<u32>(&queue, 1)?;
            let g_times_tables_buffer = g_tables::create_g_tables_buffer(&queue)?;

            let program = Self::build_program(device, context.clone(), ppt)?;

            let mut kernel_builder = Kernel::builder();
            kernel_builder
                .program(&program)
                .name("batch_address_search")
                .queue(queue.clone())
                .global_work_size(1000)
                .arg(&cache_keys_buffer)
                .arg(&cache_values_buffer)
                .arg(&ranges_buffer)
                .arg(0u32) // range_count
                .arg(&cache_size_buffer) // Now using buffer instead of scalar
                .arg(0u64) // start_counter
                .arg(0u32) // max_depth
                .arg(&matches_hash160_buffer)
                .arg(&matches_b_buffer)
                .arg(&matches_a_buffer)
                .arg(&matches_index_buffer)
                .arg(&matches_prefix_id_buffer)
                .arg(&match_count_buffer)
                .arg(&cache_miss_error_buffer)
                .arg(&g_times_tables_buffer);

            // ocl's arg type check parses "Point*" as an int pointer
            // ("Point" contains "int"), rejecting the tables buffer.
            unsafe {
                kernel_builder.disable_arg_type_check();
            }

            let kernel = kernel_builder
                .build()
                .map_err(|e| format!("Error creating kernel: {}", e))?;

            Ok(Self {
                kernel,
                cache_keys_buffer,
                cache_values_buffer,
                cache_size_buffer,
                ranges_buffer,
                matches_hash160_buffer,
                _matches_b_buffer: matches_b_buffer,
                _matches_a_buffer: matches_a_buffer,
                _matches_index_buffer: matches_index_buffer,
                _matches_prefix_id_buffer: matches_prefix_id_buffer,
                match_count_buffer,
                cache_miss_error_buffer,
                _g_times_tables_buffer: g_times_tables_buffer,
                ppt,
            })
        }

        fn load_cache(&mut self, cache: &GpuCache) -> Result<(), String> {
            let size = cache.size();

            // Read data from GpuCache buffers
            let mut keys = vec![CacheKey::default(); size];
            let mut values = vec![XPub::default(); size];

            cache
                .keys_buffer()
                .read(&mut keys)
                .enq()
                .map_err(|e| format!("Error reading cache keys: {}", e))?;

            cache
                .values_buffer()
                .read(&mut values)
                .enq()
                .map_err(|e| format!("Error reading cache values: {}", e))?;

            // Write to test buffers
            self.cache_keys_buffer
                .write(&keys)
                .enq()
                .map_err(|e| format!("Error writing cache keys: {}", e))?;

            self.cache_values_buffer
                .write(&values)
                .enq()
                .map_err(|e| format!("Error writing cache values: {}", e))?;

            // Update cache size in buffer
            let cache_size_data = vec![size as u32];
            self.cache_size_buffer
                .write(&cache_size_data)
                .enq()
                .map_err(|e| format!("Error writing cache size: {}", e))?;

            Ok(())
        }

        fn load_ranges(&mut self, prefix: &Prefix) -> Result<(), String> {
            let mut gpu_ranges = Vec::new();

            for range in &prefix.ranges {
                gpu_ranges.push(Hash160RangeGpu {
                    low: range.low,
                    high: range.high,
                    prefix_id: 0, // Single prefix in tests, always use id 0
                });
            }

            self.ranges_buffer
                .write(&gpu_ranges)
                .enq()
                .map_err(|e| format!("Error writing ranges: {}", e))?;

            Ok(())
        }

        fn execute(
            &mut self,
            range_count: u32,
            cache_size: u32,
            start_counter: u64,
            work_size: usize,
            max_depth: u32,
        ) -> Result<(), String> {
            // Reset match count and cache miss counter
            let zero = vec![0u32; 1];
            self.match_count_buffer
                .write(&zero)
                .enq()
                .map_err(|e| format!("Error resetting match count: {}", e))?;
            self.cache_miss_error_buffer
                .write(&zero)
                .enq()
                .map_err(|e| format!("Error resetting cache miss counter: {}", e))?;

            self.kernel
                .set_arg(3, range_count)
                .map_err(|e| format!("Error setting range_count: {}", e))?;

            // Update cache size in buffer instead of setting kernel arg
            let cache_size_data = vec![cache_size];
            self.cache_size_buffer
                .write(&cache_size_data)
                .enq()
                .map_err(|e| format!("Error writing cache_size: {}", e))?;

            self.kernel
                .set_arg(5, start_counter)
                .map_err(|e| format!("Error setting start_counter: {}", e))?;
            self.kernel
                .set_arg(6, max_depth)
                .map_err(|e| format!("Error setting max_depth: {}", e))?;

            // Each thread processes `ppt` counters, so the launch needs
            // work_size / ppt threads. Callers pass work_size as a multiple.
            let threads = work_size / self.ppt as usize;
            unsafe {
                self.kernel
                    .cmd()
                    .global_work_size(threads)
                    .enq()
                    .map_err(|e| format!("Error executing kernel: {}", e))?;
            }

            Ok(())
        }

        fn read_cache_miss_errors(&self) -> Result<u32, String> {
            let mut miss_count = vec![0u32; 1];
            self.cache_miss_error_buffer
                .read(&mut miss_count)
                .enq()
                .map_err(|e| format!("Error reading cache miss counter: {}", e))?;
            Ok(miss_count[0])
        }

        fn read_matches(&self) -> Result<(Vec<[u8; 20]>, u32), String> {
            let mut match_count = vec![0u32; 1];
            self.match_count_buffer
                .read(&mut match_count)
                .enq()
                .map_err(|e| format!("Error reading match count: {}", e))?;

            let count = match_count[0] as usize;
            if count == 0 {
                return Ok((vec![], 0));
            }

            let mut matches_flat = vec![0u8; count.min(1000) * 20];
            self.matches_hash160_buffer
                .read(&mut matches_flat)
                .enq()
                .map_err(|e| format!("Error reading matches: {}", e))?;

            let mut matches = Vec::new();
            for i in 0..count.min(1000) {
                let mut hash = [0u8; 20];
                hash.copy_from_slice(&matches_flat[i * 20..(i + 1) * 20]);
                matches.push(hash);
            }

            Ok((matches, match_count[0]))
        }

        fn new_buffer<T: ocl::OclPrm>(queue: &Queue, len: usize) -> Result<Buffer<T>, String> {
            Buffer::<T>::builder()
                .queue(queue.clone())
                .len(len)
                .build()
                .map_err(|e| format!("Error creating buffer: {}", e))
        }

        fn build_program(device: Device, context: Context, ppt: u64) -> Result<Program, String> {
            let src = include_str!(concat!(env!("OUT_DIR"), "/batch_address_search"));

            Program::builder()
                .cmplr_opt(format!("-D POINTS_PER_THREAD={}", ppt))
                .src(src)
                .devices(device)
                .build(&context)
                .map_err(|e| format!("Error building OpenCL program: {}", e))
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

    #[test]
    fn test_batch_address_search_basic() {
        let (device, context, queue) = create_test_opencl_context();
        let mut gpu_cache = GpuCache::new(device, context, queue, 100).unwrap();
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);

        let cache_keys = vec![[0, 0]];
        CachePreloader::preload(&mut gpu_cache, &cache_keys, &mut deriver, 0, 0).unwrap();

        let prefix = Prefix::new("1").unwrap();
        assert!(!prefix.ranges.is_empty());

        // For now just verify setup works
        assert_eq!(gpu_cache.size(), 1);
        assert!(gpu_cache.contains_key(&[0, 0]).unwrap());
    }

    #[test]
    fn test_batch_address_search_kernel_execution() {
        let mut search = BatchAddressSearch::new().unwrap();

        let prefix = Prefix::new("1").unwrap();
        search.load_ranges(&prefix).unwrap();

        // Execute with minimal params (no cache, should find nothing)
        search
            .execute(
                prefix.ranges.len() as u32,
                0, // cache_size = 0
                0,
                100, // work_size
                10000,
            )
            .unwrap();

        let (matches, count) = search.read_matches().unwrap();
        assert_eq!(count, 0);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_batch_address_search_impossible_prefix() {
        let mut search = BatchAddressSearch::new().unwrap();

        let prefix = Prefix::new("1ZZZZZZZZZ").unwrap();
        search.load_ranges(&prefix).unwrap();

        search
            .execute(prefix.ranges.len() as u32, 0, 0, 1000, 10000)
            .unwrap();

        let (matches, count) = search.read_matches().unwrap();
        assert_eq!(count, 0);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_batch_address_search_with_cache() {
        // Setup GPU cache
        let (device, context, queue) = create_test_opencl_context();
        let mut gpu_cache = GpuCache::new(device, context, queue, 100).unwrap();
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);

        // Preload cache with [0, 0], [0, 1]
        let cache_keys = vec![[0, 0], [0, 1]];
        CachePreloader::preload(&mut gpu_cache, &cache_keys, &mut deriver, 0, 0).unwrap();

        assert_eq!(gpu_cache.size(), 2);

        // Setup search kernel
        let mut search = BatchAddressSearch::new().unwrap();
        search.load_cache(&gpu_cache).unwrap();

        // Use broad prefix "1" (matches most addresses)
        let prefix = Prefix::new("1").unwrap();
        search.load_ranges(&prefix).unwrap();

        // Search in first 1000 addresses (covers indices 0-999 for [0,0])
        let max_depth = 10000;
        search
            .execute(
                prefix.ranges.len() as u32,
                gpu_cache.size() as u32,
                0,    // start_counter
                1000, // work_size
                max_depth,
            )
            .unwrap();

        // Should find some matches with prefix "1"
        let (matches, count) = search.read_matches().unwrap();
        println!("Found {} matches", count);

        // Prefix "1" is very broad, should find at least one match
        assert!(count > 0, "Should find at least one match with prefix '1'");
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_batch_address_search_abc_at_index_0() {
        let xpub_str = "xpub6DK1UMgy8RpXQYaE6PmRfEMf2tkTzz8wBHreDSriH5bXQb2KE4f9MzEnAMMbpoQ4HcaUyMytM7d2cBLXvtEMJXgmofNCaRh8Ah5HzwiRHLD";
        let seed0 = 140551173u32;
        let seed1 = 529078484u32;
        let b = 0u32;
        let a = 2367619u32;
        let index = 0u32;
        let max_depth = 1u32;

        // Setup cache
        let (device, context, queue) = create_test_opencl_context();
        let mut gpu_cache = GpuCache::new(device, context, queue, 100).unwrap();
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);

        // Preload cache with [b, a]
        let cache_keys = vec![[b, a]];
        CachePreloader::preload(&mut gpu_cache, &cache_keys, &mut deriver, seed0, seed1).unwrap();

        let mut search = BatchAddressSearch::new().unwrap();
        search.load_cache(&gpu_cache).unwrap();

        let prefix = Prefix::new("1abc").unwrap();
        search.load_ranges(&prefix).unwrap();

        let non_hardened_count = 0x7FFFFFFFu64 + 1;
        let counter = (b as u64) * (max_depth as u64 * non_hardened_count)
            + (a as u64) * (max_depth as u64)
            + (index as u64);

        search
            .execute(
                prefix.ranges.len() as u32,
                gpu_cache.size() as u32,
                counter,
                1,
                max_depth,
            )
            .unwrap();

        let (matches, count) = search.read_matches().unwrap();
        assert_eq!(
            count, 1,
            "Should find exactly 1 match for prefix '1abc' at index {}",
            index
        );
        assert_eq!(matches.len(), 1);
    }

    /// Ordinal of a cache key: the position of (b, a) in the global key
    /// sequence that CacheRangeAnalyzer walks.
    fn key_ordinal(b: u32, a: u32) -> u64 {
        ((b as u64) << 31) + a as u64
    }

    #[test]
    fn test_batch_search_fetches_correct_parents_across_rollover() {
        use std::collections::HashSet;

        // Cache crossing the a -> b rollover, with max_depth = 1 so every
        // thread uses a DIFFERENT parent xpub: any indexing mistake in the
        // O(1) cache lookup produces a wrong hash160 and fails the test.
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let seed0 = 111u32;
        let seed1 = 222u32;
        let max_depth = 1u32;
        let cache_keys = vec![[0u32, 0x7FFFFFFE], [0, 0x7FFFFFFF], [1, 0], [1, 1]];

        let (device, context, queue) = create_test_opencl_context();
        let mut gpu_cache = GpuCache::new(device, context, queue, 100).unwrap();
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);
        CachePreloader::preload(&mut gpu_cache, &cache_keys, &mut deriver, seed0, seed1).unwrap();

        let mut search = BatchAddressSearch::new().unwrap();
        search.load_cache(&gpu_cache).unwrap();

        // Prefix "1" matches every P2PKH address: all 4 threads must report
        let prefix = Prefix::new("1").unwrap();
        search.load_ranges(&prefix).unwrap();

        let start_counter = key_ordinal(0, 0x7FFFFFFE) * max_depth as u64;
        search
            .execute(
                prefix.ranges.len() as u32,
                gpu_cache.size() as u32,
                start_counter,
                cache_keys.len(),
                max_depth,
            )
            .unwrap();

        assert_eq!(search.read_cache_miss_errors().unwrap(), 0);

        let (matches, count) = search.read_matches().unwrap();
        assert_eq!(count, 4, "all 4 threads must match prefix '1'");

        // Ground truth: derive each thread's hash160 on the CPU
        let expected: HashSet<[u8; 20]> = cache_keys
            .iter()
            .map(|&[b, a]| {
                deriver
                    .get_pubkey_hash_160(&[seed0, seed1, b, a, 0, 0])
                    .unwrap()
            })
            .collect();
        assert_eq!(expected.len(), 4, "parents must produce distinct hashes");

        let got: HashSet<[u8; 20]> = matches.into_iter().collect();
        assert_eq!(got, expected, "GPU fetched a wrong parent from the cache");
    }

    #[test]
    fn test_batch_search_value_alignment_multiple_parents() {
        use std::collections::HashSet;

        // 5 contiguous parents with max_depth = 2: 10 counters covering
        // (parent, index) pairs. The full hash160 set must match the CPU.
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let seed0 = 42u32;
        let seed1 = 4242u32;
        let max_depth = 2u32;
        let cache_keys = vec![[7u32, 100], [7, 101], [7, 102], [7, 103], [7, 104]];

        let (device, context, queue) = create_test_opencl_context();
        let mut gpu_cache = GpuCache::new(device, context, queue, 100).unwrap();
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);
        CachePreloader::preload(&mut gpu_cache, &cache_keys, &mut deriver, seed0, seed1).unwrap();

        let mut search = BatchAddressSearch::new().unwrap();
        search.load_cache(&gpu_cache).unwrap();

        let prefix = Prefix::new("1").unwrap();
        search.load_ranges(&prefix).unwrap();

        let work_size = cache_keys.len() * max_depth as usize;
        let start_counter = key_ordinal(7, 100) * max_depth as u64;
        search
            .execute(
                prefix.ranges.len() as u32,
                gpu_cache.size() as u32,
                start_counter,
                work_size,
                max_depth,
            )
            .unwrap();

        assert_eq!(search.read_cache_miss_errors().unwrap(), 0);

        let (matches, count) = search.read_matches().unwrap();
        assert_eq!(count, work_size as u32);

        let mut expected: HashSet<[u8; 20]> = HashSet::new();
        for &[b, a] in &cache_keys {
            for index in 0..max_depth {
                expected.insert(
                    deriver
                        .get_pubkey_hash_160(&[seed0, seed1, b, a, 0, index])
                        .unwrap(),
                );
            }
        }
        assert_eq!(expected.len(), work_size);

        let got: HashSet<[u8; 20]> = matches.into_iter().collect();
        assert_eq!(got, expected, "GPU (parent, index) mapping is misaligned");
    }

    #[test]
    fn test_batch_search_counter_before_cache_reports_miss() {
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let max_depth = 1u32;
        let cache_keys = vec![[3u32, 1000]];

        let (device, context, queue) = create_test_opencl_context();
        let mut gpu_cache = GpuCache::new(device, context, queue, 100).unwrap();
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);
        CachePreloader::preload(&mut gpu_cache, &cache_keys, &mut deriver, 0, 0).unwrap();

        let mut search = BatchAddressSearch::new().unwrap();
        search.load_cache(&gpu_cache).unwrap();
        let prefix = Prefix::new("1").unwrap();
        search.load_ranges(&prefix).unwrap();

        // Counter points to (3, 999): the key immediately BEFORE the cache
        let start_counter = key_ordinal(3, 999) * max_depth as u64;
        search
            .execute(
                prefix.ranges.len() as u32,
                gpu_cache.size() as u32,
                start_counter,
                1,
                max_depth,
            )
            .unwrap();

        let (_, count) = search.read_matches().unwrap();
        assert_eq!(count, 0, "no address may be produced from a missing key");
        assert_eq!(search.read_cache_miss_errors().unwrap(), 1);
    }

    #[test]
    fn test_batch_search_counter_after_cache_reports_miss() {
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let max_depth = 5u32;
        let cache_keys = vec![[3u32, 1000], [3, 1001]];

        let (device, context, queue) = create_test_opencl_context();
        let mut gpu_cache = GpuCache::new(device, context, queue, 100).unwrap();
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);
        CachePreloader::preload(&mut gpu_cache, &cache_keys, &mut deriver, 0, 0).unwrap();

        let mut search = BatchAddressSearch::new().unwrap();
        search.load_cache(&gpu_cache).unwrap();
        let prefix = Prefix::new("1").unwrap();
        search.load_ranges(&prefix).unwrap();

        // 10 counters cover the cache; work_size 13 overshoots by 3 threads
        let start_counter = key_ordinal(3, 1000) * max_depth as u64;
        search
            .execute(
                prefix.ranges.len() as u32,
                gpu_cache.size() as u32,
                start_counter,
                13,
                max_depth,
            )
            .unwrap();

        let (_, count) = search.read_matches().unwrap();
        assert_eq!(count, 10, "the 10 in-range threads must match prefix '1'");
        assert_eq!(
            search.read_cache_miss_errors().unwrap(),
            3,
            "the 3 overshooting threads must report a miss"
        );
    }

    // ================= Batched inversion (POINTS_PER_THREAD) =================

    /// Run the same counter range through the kernel built with a given
    /// POINTS_PER_THREAD and return the set of matched hash160s.
    #[allow(clippy::too_many_arguments)]
    fn run_match_set(
        ppt: u64,
        cache_keys: &[[u32; 2]],
        seed0: u32,
        seed1: u32,
        max_depth: u32,
        start_counter: u64,
        work_size: usize,
        prefix: &Prefix,
    ) -> std::collections::HashSet<[u8; 20]> {
        use std::collections::HashSet;

        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let (device, context, queue) = create_test_opencl_context();
        let mut gpu_cache = GpuCache::new(device, context, queue, 100).unwrap();
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);
        CachePreloader::preload(&mut gpu_cache, cache_keys, &mut deriver, seed0, seed1).unwrap();

        let mut search = BatchAddressSearch::new_with_ppt(ppt).unwrap();
        search.load_cache(&gpu_cache).unwrap();
        search.load_ranges(prefix).unwrap();

        search
            .execute(
                prefix.ranges.len() as u32,
                gpu_cache.size() as u32,
                start_counter,
                work_size,
                max_depth,
            )
            .unwrap();

        assert_eq!(
            search.read_cache_miss_errors().unwrap(),
            0,
            "ppt={} produced cache misses",
            ppt
        );

        let (matches, count) = search.read_matches().unwrap();
        assert_eq!(count as usize, matches.len().min(1000));
        matches.into_iter().collect::<HashSet<[u8; 20]>>()
    }

    #[test]
    fn test_ppt_batched_matches_ppt1_broad_prefix() {
        // Broad prefix "1": every address matches, so this compares the full
        // per-thread output of the batched-inversion path (ppt=8) against the
        // one-inversion-per-thread path (ppt=1) over the same counters.
        let cache_keys = vec![[0u32, 0], [0, 1], [0, 2], [0, 3]];
        let max_depth = 250u32;
        let work_size = cache_keys.len() * max_depth as usize; // 1000 counters
        let prefix = Prefix::new("1").unwrap();

        let baseline = run_match_set(1, &cache_keys, 7, 9, max_depth, 0, work_size, &prefix);
        assert_eq!(
            baseline.len(),
            work_size,
            "broad prefix should match every address"
        );

        for ppt in [2u64, 4, 8] {
            let batched = run_match_set(ppt, &cache_keys, 7, 9, max_depth, 0, work_size, &prefix);
            assert_eq!(
                batched, baseline,
                "ppt={} produced a different match set than ppt=1",
                ppt
            );
        }
    }

    #[test]
    fn test_ppt_batched_matches_ppt1_across_rollover() {
        // Counters spanning the a -> b rollover with max_depth = 1, so every
        // counter uses a distinct parent: any misalignment in the batched
        // path changes the hashes.
        let cache_keys = vec![
            [0u32, 0x7FFFFFFE],
            [0, 0x7FFFFFFF],
            [1, 0],
            [1, 1],
            [1, 2],
            [1, 3],
            [1, 4],
            [1, 5],
        ];
        let max_depth = 1u32;
        let work_size = cache_keys.len(); // 8 counters
        // ordinal of first key = b * 2^31 + a, with b = 0, a = 0x7FFFFFFE
        #[allow(clippy::identity_op)]
        let start_counter = (0u64 << 31) + 0x7FFFFFFE;
        let prefix = Prefix::new("1").unwrap();

        let baseline = run_match_set(
            1,
            &cache_keys,
            123,
            456,
            max_depth,
            start_counter,
            work_size,
            &prefix,
        );
        assert_eq!(baseline.len(), 8);

        for ppt in [2u64, 4, 8] {
            let batched = run_match_set(
                ppt,
                &cache_keys,
                123,
                456,
                max_depth,
                start_counter,
                work_size,
                &prefix,
            );
            assert_eq!(
                batched, baseline,
                "ppt={} diverged from ppt=1 across the rollover",
                ppt
            );
        }
    }

    #[test]
    fn test_ppt_batched_matches_ppt1_selective_prefix() {
        // A longer prefix: most addresses do NOT match. Verifies the batched
        // path reports the same rare matches (and no spurious ones) as ppt=1.
        let cache_keys: Vec<[u32; 2]> = (0..8).map(|a| [5u32, 1000 + a]).collect();
        let max_depth = 256u32;
        let work_size = cache_keys.len() * max_depth as usize; // 2048 counters
        let start_counter = ((5u64) << 31 | 1000) * max_depth as u64;
        let prefix = Prefix::new("1a").unwrap();

        let baseline = run_match_set(
            1,
            &cache_keys,
            11,
            22,
            max_depth,
            start_counter,
            work_size,
            &prefix,
        );

        for ppt in [2u64, 4, 8] {
            let batched = run_match_set(
                ppt,
                &cache_keys,
                11,
                22,
                max_depth,
                start_counter,
                work_size,
                &prefix,
            );
            assert_eq!(
                batched, baseline,
                "ppt={} diverged from ppt=1 with a selective prefix",
                ppt
            );
        }
    }
}
