use crate::extended_public_key_deriver::ExtendedPublicKeyDeriver;
use crate::opencl::gpu_cache::{GpuCache, PointGpu, Uint256, XPub};

pub struct CachePreloader;

impl CachePreloader {
    /// Preload cache with XPubs derived from cache keys
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

        let mut xpubs = Vec::with_capacity(cache_keys.len());

        for &[b, a] in cache_keys {
            // Build path: [seed0, seed1, b, a, 0]
            let path = vec![seed0, seed1, b, a, 0];

            // Derive using CPU deriver - returns (chain_code, x, y)
            let (chain_code, x_bytes, y_bytes) = deriver
                .get_extended_key(&path)
                .map_err(|e| format!("Failed to derive key for [{}, {}]: {}", b, a, e))?;

            // Convert to GPU format
            let xpub = Self::bytes_to_gpu_xpub(&chain_code, &x_bytes, &y_bytes);
            xpubs.push(xpub);
        }

        // Replace cache data (GpuCache will only write to GPU if keys changed)
        cache.replace_data(cache_keys, &xpubs)
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
        const LIMB_COUNT: usize = 4;
        const BYTES_PER_LIMB: usize = 8;
        let mut limbs = [0u64; LIMB_COUNT];

        for limb_idx in 0..LIMB_COUNT {
            let byte_offset = limb_idx * BYTES_PER_LIMB;

            limbs[limb_idx] = u64::from_be_bytes([
                bytes[byte_offset],
                bytes[byte_offset + 1],
                bytes[byte_offset + 2],
                bytes[byte_offset + 3],
                bytes[byte_offset + 4],
                bytes[byte_offset + 5],
                bytes[byte_offset + 6],
                bytes[byte_offset + 7],
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
        assert_ne!(gpu_xpub.k_par.x.limbs, [0u64; 4]);
        assert_ne!(gpu_xpub.k_par.y.limbs, [0u64; 4]);
    }
}
