use crate::extended_public_key::ExtendedPubKey;
use crate::extended_public_key_deriver::ExtendedPublicKeyDeriver;
#[cfg(test)]
use crate::opencl::gpu_cache::GpuCache;
use crate::opencl::gpu_cache::{PointGpu, Uint256, XPub};
use std::thread;

pub struct CachePreloader;

impl CachePreloader {
    /// Preload cache with XPubs derived from cache keys (single-threaded).
    ///
    /// Production uses the parallel producer in GpuWorkbench; this stays as a
    /// simple, self-contained path exercised by the cache/kernel tests.
    #[cfg(test)]
    pub fn preload(
        cache: &mut GpuCache,
        cache_keys: &[[u32; 2]],
        deriver: &mut ExtendedPublicKeyDeriver,
        seed0: u32,
        seed1: u32,
    ) -> Result<bool, String> {
        if cache_keys.is_empty() {
            return Ok(false);
        }

        let xpubs = Self::derive_xpubs(cache_keys, deriver, seed0, seed1)?;

        // Replace cache data (GpuCache will only write to GPU if keys changed)
        cache.replace_data(cache_keys, &xpubs)
    }

    /// Derive the GPU-format parent XPub for each cache key, in order, using
    /// the provided deriver. Pure CPU work - no GPU involved.
    pub fn derive_xpubs(
        cache_keys: &[[u32; 2]],
        deriver: &mut ExtendedPublicKeyDeriver,
        seed0: u32,
        seed1: u32,
    ) -> Result<Vec<XPub>, String> {
        let mut xpubs = Vec::with_capacity(cache_keys.len());

        for &[b, a] in cache_keys {
            // Build path: [seed0, seed1, b, a, 0]
            let path = [seed0, seed1, b, a, 0];

            // Derive using CPU deriver - returns (chain_code, x, y)
            let (chain_code, x_bytes, y_bytes) = deriver
                .get_extended_key(&path)
                .map_err(|e| format!("Failed to derive key for [{}, {}]: {}", b, a, e))?;

            xpubs.push(Self::bytes_to_gpu_xpub(&chain_code, &x_bytes, &y_bytes));
        }

        Ok(xpubs)
    }

    /// Derive the parent XPubs across `num_threads` CPU threads and return
    /// them in the original key order.
    ///
    /// The keys are split into CONTIGUOUS chunks, one per thread, so each
    /// thread's deriver keeps the LRU prefix reuse that makes derivation
    /// cheap (a fresh deriver only re-derives the shared [seed0, seed1, ...]
    /// prefix once). Every thread gets its own deriver, so this is a pure
    /// fan-out with no shared state.
    ///
    /// `num_threads` is clamped to at least 1 and at most the key count.
    pub fn derive_xpubs_parallel(
        cache_keys: &[[u32; 2]],
        base_xpub: &ExtendedPubKey,
        seed0: u32,
        seed1: u32,
        num_threads: usize,
    ) -> Result<Vec<XPub>, String> {
        if cache_keys.is_empty() {
            return Ok(Vec::new());
        }

        let num_threads = num_threads.clamp(1, cache_keys.len());
        if num_threads == 1 {
            let mut deriver = ExtendedPublicKeyDeriver::new(base_xpub);
            return Self::derive_xpubs(cache_keys, &mut deriver, seed0, seed1);
        }

        // Ceil division so every key lands in exactly one contiguous chunk.
        let chunk_size = cache_keys.len().div_ceil(num_threads);

        let chunk_results: Vec<Result<Vec<XPub>, String>> = thread::scope(|scope| {
            let handles: Vec<_> = cache_keys
                .chunks(chunk_size)
                .map(|chunk| {
                    scope.spawn(move || {
                        let mut deriver = ExtendedPublicKeyDeriver::new(base_xpub);
                        Self::derive_xpubs(chunk, &mut deriver, seed0, seed1)
                    })
                })
                .collect();

            handles
                .into_iter()
                .map(|h| {
                    h.join()
                        .unwrap_or_else(|_| Err("Derivation thread panicked".to_string()))
                })
                .collect()
        });

        let mut xpubs = Vec::with_capacity(cache_keys.len());
        for chunk_result in chunk_results {
            xpubs.extend(chunk_result?);
        }

        Ok(xpubs)
    }

    fn bytes_to_gpu_xpub(chain_code: &[u8; 32], x_bytes: &[u8; 32], y_bytes: &[u8; 32]) -> XPub {
        let x = Self::bytes_to_uint256(x_bytes);
        let y = Self::bytes_to_uint256(y_bytes);

        XPub {
            chain_code: *chain_code,
            k_par: PointGpu { x, y },
        }
    }

    fn bytes_to_uint256(bytes: &[u8; 32]) -> Uint256 {
        const LIMB_COUNT: usize = 8;
        const BYTES_PER_LIMB: usize = 4;
        let mut limbs = [0u32; LIMB_COUNT];

        for (limb_idx, limb) in limbs.iter_mut().enumerate() {
            let byte_offset = limb_idx * BYTES_PER_LIMB;

            *limb = u32::from_be_bytes([
                bytes[byte_offset],
                bytes[byte_offset + 1],
                bytes[byte_offset + 2],
                bytes[byte_offset + 3],
            ]);
        }

        Uint256 { limbs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extended_public_key::ExtendedPubKey;
    use ocl::{Context, Device, Platform, Queue};

    const TEST_XPUB: &str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";

    // ============= Parallel derivation tests (no GPU needed) =============

    fn serial_xpubs(cache_keys: &[[u32; 2]], seed0: u32, seed1: u32) -> Vec<XPub> {
        let xpub = ExtendedPubKey::from_str(TEST_XPUB).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);
        CachePreloader::derive_xpubs(cache_keys, &mut deriver, seed0, seed1).unwrap()
    }

    fn assert_parallel_matches_serial(cache_keys: &[[u32; 2]], seed0: u32, seed1: u32) {
        let xpub = ExtendedPubKey::from_str(TEST_XPUB).unwrap();
        let expected = serial_xpubs(cache_keys, seed0, seed1);

        // Parallel result must be identical (same values, same order) for
        // any thread count - including more threads than keys.
        for num_threads in [1, 2, 3, 4, 8, 16, 64] {
            let got =
                CachePreloader::derive_xpubs_parallel(cache_keys, &xpub, seed0, seed1, num_threads)
                    .unwrap();
            assert_eq!(
                got.len(),
                cache_keys.len(),
                "num_threads={} produced wrong length",
                num_threads
            );
            assert_eq!(
                got, expected,
                "num_threads={} diverged from the serial derivation",
                num_threads
            );
        }
    }

    #[test]
    fn test_parallel_derivation_empty() {
        let xpub = ExtendedPubKey::from_str(TEST_XPUB).unwrap();
        let got = CachePreloader::derive_xpubs_parallel(&[], &xpub, 0, 0, 8).unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn test_parallel_derivation_single_key() {
        assert_parallel_matches_serial(&[[0, 0]], 111, 222);
    }

    #[test]
    fn test_parallel_derivation_contiguous_same_b() {
        let keys: Vec<[u32; 2]> = (0..100).map(|a| [0, a]).collect();
        assert_parallel_matches_serial(&keys, 12345, 67890);
    }

    #[test]
    fn test_parallel_derivation_crossing_b_boundary() {
        // Keys straddling the a -> b rollover, as a real batch would
        let mut keys: Vec<[u32; 2]> = (0x7FFFFFFB..=0x7FFFFFFF).map(|a| [3, a]).collect();
        keys.extend((0..5).map(|a| [4, a]));
        assert_parallel_matches_serial(&keys, 42, 4242);
    }

    #[test]
    fn test_parallel_derivation_odd_count_uneven_chunks() {
        // 97 is prime: chunk splitting never divides evenly
        let keys: Vec<[u32; 2]> = (0..97).map(|a| [7, 1000 + a]).collect();
        assert_parallel_matches_serial(&keys, 1, 2);
    }

    #[test]
    fn test_parallel_derivation_more_threads_than_keys() {
        let keys = vec![[5, 500], [5, 501], [5, 502]];
        // 64 threads for 3 keys must still be correct (clamped internally)
        assert_parallel_matches_serial(&keys, 9, 9);
    }

    #[test]
    fn test_parallel_derivation_matches_across_seeds() {
        let keys: Vec<[u32; 2]> = (0..50).map(|a| [2, a]).collect();
        assert_parallel_matches_serial(&keys, 0, 0);
        assert_parallel_matches_serial(&keys, 0x7FFFFFFF, 0x7FFFFFFF);
        assert_parallel_matches_serial(&keys, 1949567566 & 0x7FFFFFFF, 243133792);
    }

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

    #[test]
    fn test_preload_single_key() {
        let (device, context, queue) = create_test_opencl_context();
        let mut cache = GpuCache::new(device, context, queue, 100).unwrap();
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);

        let cache_keys = vec![[0, 0]];

        CachePreloader::preload(&mut cache, &cache_keys, &mut deriver, 0, 0).unwrap();

        assert_eq!(cache.size(), 1);
        assert!(cache.contains_key(&[0, 0]).unwrap());
    }

    #[test]
    fn test_preload_multiple_keys() {
        let (device, context, queue) = create_test_opencl_context();
        let mut cache = GpuCache::new(device, context, queue, 100).unwrap();
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);

        let cache_keys = vec![[0, 0], [0, 1], [0, 2]];

        CachePreloader::preload(&mut cache, &cache_keys, &mut deriver, 0, 0).unwrap();

        assert_eq!(cache.size(), 3);
        assert!(cache.contains_key(&[0, 0]).unwrap());
        assert!(cache.contains_key(&[0, 1]).unwrap());
        assert!(cache.contains_key(&[0, 2]).unwrap());
    }

    #[test]
    fn test_preload_empty() {
        let (device, context, queue) = create_test_opencl_context();
        let mut cache = GpuCache::new(device, context, queue, 100).unwrap();
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);

        let cache_keys: Vec<[u32; 2]> = vec![];

        CachePreloader::preload(&mut cache, &cache_keys, &mut deriver, 0, 0).unwrap();

        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_bytes_to_gpu_xpub() {
        let chain_code = [1u8; 32];
        let x_bytes = [2u8; 32];
        let y_bytes = [3u8; 32];

        let gpu_xpub = CachePreloader::bytes_to_gpu_xpub(&chain_code, &x_bytes, &y_bytes);

        assert_eq!(gpu_xpub.chain_code, chain_code);
        assert_ne!(gpu_xpub.k_par.x.limbs, [0u32; 8]);
        assert_ne!(gpu_xpub.k_par.y.limbs, [0u32; 8]);
    }
}
