#[cfg(test)]
mod tests {
    use crate::extended_public_key::ExtendedPubKey;
    use crate::extended_public_key_deriver::ExtendedPublicKeyDeriver;
    use crate::opencl::cache_preloader::CachePreloader;
    use crate::opencl::gpu_cache::{CacheKey, GpuCache, XPub};
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
        range_lows_buffer: Buffer<u8>,
        range_highs_buffer: Buffer<u8>,
        matches_hash160_buffer: Buffer<u8>,
        _matches_b_buffer: Buffer<u32>,        // Needed for kernel args but not read in tests
        _matches_a_buffer: Buffer<u32>,        // Needed for kernel args but not read in tests
        _matches_index_buffer: Buffer<u32>,    // Needed for kernel args but not read in tests
        match_count_buffer: Buffer<u32>,
        _cache_miss_error_buffer: Buffer<u32>, // Needed for kernel args but not read in tests
    }

    impl BatchAddressSearch {
        fn new() -> Result<Self, String> {
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            let cache_keys_buffer = Self::new_buffer::<CacheKey>(&queue, 1000)?;
            let cache_values_buffer = Self::new_buffer::<XPub>(&queue, 1000)?;
            let range_lows_buffer = Self::new_buffer::<u8>(&queue, 10 * 20)?;
            let range_highs_buffer = Self::new_buffer::<u8>(&queue, 10 * 20)?;
            let matches_hash160_buffer = Self::new_buffer::<u8>(&queue, 1000 * 20)?;
            let matches_b_buffer = Self::new_buffer::<u32>(&queue, 1000)?;
            let matches_a_buffer = Self::new_buffer::<u32>(&queue, 1000)?;
            let matches_index_buffer = Self::new_buffer::<u32>(&queue, 1000)?;
            let match_count_buffer = Self::new_buffer::<u32>(&queue, 1)?;
            let cache_miss_error_buffer = Self::new_buffer::<u32>(&queue, 1)?;

            let program = Self::build_program(device, context.clone())?;

            let kernel = Kernel::builder()
                .program(&program)
                .name("batch_address_search")
                .queue(queue.clone())
                .global_work_size(1000)
                .arg(&cache_keys_buffer)
                .arg(&cache_values_buffer)
                .arg(&range_lows_buffer)
                .arg(&range_highs_buffer)
                .arg(0u32) // range_count
                .arg(0u32) // cache_size
                .arg(0u64) // start_counter
                .arg(0u32) // max_depth
                .arg(&matches_hash160_buffer)
                .arg(&matches_b_buffer)
                .arg(&matches_a_buffer)
                .arg(&matches_index_buffer)
                .arg(&match_count_buffer)
                .arg(&cache_miss_error_buffer)
                .build()
                .map_err(|e| format!("Error creating kernel: {}", e))?;

            Ok(Self {
                kernel,
                cache_keys_buffer,
                cache_values_buffer,
                range_lows_buffer,
                range_highs_buffer,
                matches_hash160_buffer,
                _matches_b_buffer: matches_b_buffer,
                _matches_a_buffer: matches_a_buffer,
                _matches_index_buffer: matches_index_buffer,
                match_count_buffer,
                _cache_miss_error_buffer: cache_miss_error_buffer,
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

            Ok(())
        }

        fn load_ranges(&mut self, prefix: &Prefix) -> Result<(), String> {
            let mut lows = Vec::new();
            let mut highs = Vec::new();

            for range in &prefix.ranges {
                lows.extend_from_slice(&range.low);
                highs.extend_from_slice(&range.high);
            }

            self.range_lows_buffer
                .write(&lows)
                .enq()
                .map_err(|e| format!("Error writing lows: {}", e))?;

            self.range_highs_buffer
                .write(&highs)
                .enq()
                .map_err(|e| format!("Error writing highs: {}", e))?;

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
            // Reset match count
            let zero = vec![0u32; 1];
            self.match_count_buffer
                .write(&zero)
                .enq()
                .map_err(|e| format!("Error resetting match count: {}", e))?;

            self.kernel
                .set_arg(4, range_count)
                .map_err(|e| format!("Error setting range_count: {}", e))?;
            self.kernel
                .set_arg(5, cache_size)
                .map_err(|e| format!("Error setting cache_size: {}", e))?;
            self.kernel
                .set_arg(6, start_counter)
                .map_err(|e| format!("Error setting start_counter: {}", e))?;
            self.kernel
                .set_arg(7, max_depth)
                .map_err(|e| format!("Error setting max_depth: {}", e))?;

            unsafe {
                self.kernel
                    .cmd()
                    .global_work_size(work_size)
                    .enq()
                    .map_err(|e| format!("Error executing kernel: {}", e))?;
            }

            Ok(())
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

        fn build_program(device: Device, context: Context) -> Result<Program, String> {
            let src = include_str!(concat!(env!("OUT_DIR"), "/batch_address_search"));

            Program::builder()
                .src(src)
                .devices(device)
                .build(&context)
                .map_err(|e| format!("Error building OpenCL program: {}", e))
        }

        fn get_device_context_and_queue() -> Result<(Device, Context, Queue), String> {
            let platform = Platform::first()
                .map_err(|e| format!("Error getting OpenCL platform: {}", e))?;

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

        let prefix = Prefix::new("1");
        assert!(prefix.ranges.len() > 0);

        // For now just verify setup works
        assert_eq!(gpu_cache.size(), 1);
        assert!(gpu_cache.contains_key(&[0, 0]).unwrap());
    }

    #[test]
    fn test_batch_address_search_kernel_execution() {
        let mut search = BatchAddressSearch::new().unwrap();

        let prefix = Prefix::new("1");
        search.load_ranges(&prefix).unwrap();

        // Execute with minimal params (no cache, should find nothing)
        search.execute(
            prefix.ranges.len() as u32,
            0,  // cache_size = 0
            0,
            100, // work_size
            10000,
        ).unwrap();

        let (matches, count) = search.read_matches().unwrap();
        assert_eq!(count, 0);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_batch_address_search_impossible_prefix() {
        let mut search = BatchAddressSearch::new().unwrap();

        let prefix = Prefix::new("1ZZZZZZZZZ");
        search.load_ranges(&prefix).unwrap();

        search.execute(
            prefix.ranges.len() as u32,
            0,
            0,
            1000,
            10000,
        ).unwrap();

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
        let prefix = Prefix::new("1");
        search.load_ranges(&prefix).unwrap();

        // Search in first 1000 addresses (covers indices 0-999 for [0,0])
        let max_depth = 10000;
        search.execute(
            prefix.ranges.len() as u32,
            gpu_cache.size() as u32,
            0,      // start_counter
            1000,   // work_size
            max_depth,
        ).unwrap();

        // Should find some matches with prefix "1"
        let (matches, count) = search.read_matches().unwrap();
        println!("Found {} matches", count);

        // Prefix "1" is very broad, should find at least one match
        assert!(count > 0, "Should find at least one match with prefix '1'");
        assert!(matches.len() > 0);
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

        let prefix = Prefix::new("1abc");
        search.load_ranges(&prefix).unwrap();

        let non_hardened_count = 0x7FFFFFFFu64 + 1;
        let counter = (b as u64) * (max_depth as u64 * non_hardened_count)
            + (a as u64) * (max_depth as u64)
            + (index as u64);

        search.execute(
            prefix.ranges.len() as u32,
            gpu_cache.size() as u32,
            counter,
            1,
            max_depth,
        ).unwrap();

        let (matches, count) = search.read_matches().unwrap();
        assert_eq!(count, 1, "Should find exactly 1 match for prefix '1abc' at index {}", index);
        assert_eq!(matches.len(), 1);
    }
}
