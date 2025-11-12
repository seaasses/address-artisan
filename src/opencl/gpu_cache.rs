use ocl::{Buffer, Context, Device, Queue};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CacheKey {
    pub b: u32,
    pub a: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uint256 {
    pub limbs: [u64; 4],
}

impl Default for Uint256 {
    fn default() -> Self {
        Self { limbs: [0u64; 4] }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointGpu {
    pub x: Uint256,
    pub y: Uint256,
}

impl Default for PointGpu {
    fn default() -> Self {
        Self {
            x: Uint256::default(),
            y: Uint256::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XPub {
    pub chain_code: [u8; 32],
    pub k_par: PointGpu,
}

impl Default for XPub {
    fn default() -> Self {
        Self {
            chain_code: [0u8; 32],
            k_par: PointGpu::default(),
        }
    }
}

unsafe impl ocl::OclPrm for CacheKey {}
unsafe impl ocl::OclPrm for Uint256 {}
unsafe impl ocl::OclPrm for PointGpu {}
unsafe impl ocl::OclPrm for XPub {}

pub struct GpuCache {
    _device: Device,
    _context: Context,
    queue: Queue,
    keys_buffer: Buffer<CacheKey>,
    values_buffer: Buffer<XPub>,
    cache_size_buffer: Buffer<u32>, // GPU buffer for cache size
    capacity: usize,
    current_size: usize,
    // Cache the last keys to avoid unnecessary GPU writes
    last_keys: Vec<[u32; 2]>,
}

impl GpuCache {
    /// Create a new GPU cache using the provided OpenCL device, context, and queue
    pub fn new(
        device: Device,
        context: Context,
        queue: Queue,
        capacity: usize,
    ) -> Result<Self, String> {
        let keys_buffer = Self::new_buffer::<CacheKey>(&queue, capacity)?;
        let values_buffer = Self::new_buffer::<XPub>(&queue, capacity)?;

        // Create buffer for cache size (single u32)
        let cache_size_buffer = Buffer::<u32>::builder()
            .queue(queue.clone())
            .len(1)
            .build()
            .map_err(|e| format!("Error creating cache_size buffer: {}", e))?;

        // Initialize cache size to 0
        cache_size_buffer
            .cmd()
            .fill(0u32, None)
            .enq()
            .map_err(|e| format!("Error initializing cache_size buffer: {}", e))?;

        Ok(Self {
            _device: device,
            _context: context,
            queue: queue.clone(),
            keys_buffer,
            values_buffer,
            cache_size_buffer,
            capacity,
            current_size: 0,
            last_keys: Vec::new(),
        })
    }

    /// Replace cache data, but only write to GPU if the keys actually changed
    /// This avoids expensive GPU writes when the same data is being loaded
    pub fn replace_data(&mut self, keys: &[[u32; 2]], values: &[XPub]) -> Result<bool, String> {
        if keys.len() != values.len() {
            return Err(format!(
                "Keys and values length mismatch: {} vs {}",
                keys.len(),
                values.len()
            ));
        }

        if keys.len() > self.capacity {
            return Err(format!(
                "Cache capacity exceeded: trying to replace with {} items but capacity is {}",
                keys.len(),
                self.capacity
            ));
        }

        // Check if keys are the same as last time
        let same_keys = self.last_keys.len() == keys.len()
            && self.last_keys.iter().zip(keys.iter()).all(|(a, b)| a == b);

        if same_keys {
            // Keys haven't changed, no need to write to GPU
            return Ok(false);
        }

        // Keys changed, need to update GPU
        let cache_keys: Vec<CacheKey> =
            keys.iter().map(|k| CacheKey { b: k[0], a: k[1] }).collect();

        // Write data directly to buffers (no sub-buffers needed for replacement)
        self.keys_buffer
            .write(&cache_keys)
            .enq()
            .map_err(|e| format!("Error writing keys to GPU: {}", e))?;

        self.values_buffer
            .write(values)
            .enq()
            .map_err(|e| format!("Error writing values to GPU: {}", e))?;

        // Update cache size in GPU
        self.current_size = keys.len();
        self.cache_size_buffer
            .cmd()
            .fill(self.current_size as u32, None)
            .enq()
            .map_err(|e| format!("Error updating cache_size in GPU: {}", e))?;

        // CRITICAL: Ensure all cache writes completed before returning
        self.queue
            .finish()
            .map_err(|e| format!("Error syncing cache writes: {}", e))?;

        // Save keys for next comparison
        self.last_keys = keys.to_vec();

        Ok(true) // Return true indicating data was written
    }

    /// Get buffers for use in batch kernels test
    pub fn get_buffers(&self) -> (&Buffer<CacheKey>, &Buffer<XPub>, &Buffer<u32>) {
        (
            &self.keys_buffer,
            &self.values_buffer,
            &self.cache_size_buffer,
        )
    }

    #[cfg(test)]
    pub fn size(&self) -> usize {
        self.current_size
    }

    #[cfg(test)]
    pub fn lookup(&self, search_keys: &[[u32; 2]]) -> Result<Vec<Option<XPub>>, String> {
        // If cache is empty, return None for all search keys
        if self.current_size == 0 {
            return Ok(vec![None; search_keys.len()]);
        }

        // Read ALL data from GPU buffers (up to capacity)
        // We'll only use the first current_size elements
        let mut keys_data = vec![CacheKey::default(); self.capacity];
        let mut values_data = vec![XPub::default(); self.capacity];

        self.keys_buffer
            .read(&mut keys_data)
            .enq()
            .map_err(|e| format!("Error reading keys from GPU: {}", e))?;

        self.values_buffer
            .read(&mut values_data)
            .enq()
            .map_err(|e| format!("Error reading values from GPU: {}", e))?;

        // Search for each key (only in the first current_size elements)
        let mut results = Vec::with_capacity(search_keys.len());
        for search_key in search_keys {
            let found = keys_data
                .iter()
                .take(self.current_size) // Only search in valid entries
                .position(|k| k.b == search_key[0] && k.a == search_key[1])
                .map(|idx| values_data[idx]);
            results.push(found);
        }

        Ok(results)
    }

    #[cfg(test)]
    pub fn contains_key(&self, key: &[u32; 2]) -> Result<bool, String> {
        // If cache is empty, key is not present
        if self.current_size == 0 {
            return Ok(false);
        }

        // Read all keys from GPU buffer
        let mut keys_data = vec![CacheKey::default(); self.capacity];

        self.keys_buffer
            .read(&mut keys_data)
            .enq()
            .map_err(|e| format!("Error reading keys from GPU: {}", e))?;

        // Only search in the first current_size elements
        Ok(keys_data
            .iter()
            .take(self.current_size)
            .any(|k| k.b == key[0] && k.a == key[1]))
    }

    #[cfg(test)]
    pub fn keys_buffer(&self) -> &Buffer<CacheKey> {
        &self.keys_buffer
    }

    #[cfg(test)]
    pub fn values_buffer(&self) -> &Buffer<XPub> {
        &self.values_buffer
    }

    fn new_buffer<T: ocl::OclPrm>(queue: &Queue, len: usize) -> Result<Buffer<T>, String> {
        Buffer::<T>::builder()
            .queue(queue.clone())
            .len(len)
            .build()
            .map_err(|e| format!("Error creating buffer: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocl::Platform;

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
    fn test_cache_miss() {
        let (device, context, queue) = create_test_opencl_context();
        let cache = GpuCache::new(device, context, queue, 100).unwrap();

        // Lookup in empty cache
        let results = cache.lookup(&[[1, 2]]).unwrap();
        assert!(results[0].is_none());

        // contains_key on empty cache
        assert!(!cache.contains_key(&[1, 2]).unwrap());
    }
}
