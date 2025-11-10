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
    keys_buffer: Buffer<CacheKey>,
    values_buffer: Buffer<XPub>,
    capacity: usize,
    current_size: usize,
}

impl GpuCache {
    /// Create a new GPU cache using the provided OpenCL device, context, and queue
    pub fn new(device: Device, context: Context, queue: Queue, capacity: usize) -> Result<Self, String> {
        let keys_buffer = Self::new_buffer::<CacheKey>(&queue, capacity)?;
        let values_buffer = Self::new_buffer::<XPub>(&queue, capacity)?;

        Ok(Self {
            _device: device,
            _context: context,
            keys_buffer,
            values_buffer,
            capacity,
            current_size: 0,
        })
    }

    pub fn add_data(
        &mut self,
        keys: &[[u32; 2]],
        values: &[XPub],
    ) -> Result<(), String> {
        if keys.len() != values.len() {
            return Err(format!(
                "Keys and values length mismatch: {} vs {}",
                keys.len(),
                values.len()
            ));
        }

        if self.current_size + keys.len() > self.capacity {
            return Err(format!(
                "Cache capacity exceeded: trying to add {} items to cache with {} / {} items",
                keys.len(),
                self.current_size,
                self.capacity
            ));
        }

        let cache_keys: Vec<CacheKey> = keys
            .iter()
            .map(|k| CacheKey { b: k[0], a: k[1] })
            .collect();

        let key_offset = self.current_size;
        let value_offset = self.current_size;

        self.keys_buffer
            .create_sub_buffer(None, key_offset, cache_keys.len())
            .map_err(|e| format!("Error creating keys sub-buffer: {}", e))?
            .write(&cache_keys)
            .enq()
            .map_err(|e| format!("Error writing keys to GPU: {}", e))?;

        self.values_buffer
            .create_sub_buffer(None, value_offset, values.len())
            .map_err(|e| format!("Error creating values sub-buffer: {}", e))?
            .write(values)
            .enq()
            .map_err(|e| format!("Error writing values to GPU: {}", e))?;

        self.current_size += keys.len();

        Ok(())
    }

    pub fn clear(&mut self) {
        self.current_size = 0;
    }

    /// Get buffers and size for use in batch kernels
    pub fn get_buffers(&self) -> (&Buffer<CacheKey>, &Buffer<XPub>, usize) {
        (&self.keys_buffer, &self.values_buffer, self.current_size)
    }

    #[cfg(test)]
    pub fn size(&self) -> usize {
        self.current_size
    }

    #[cfg(test)]
    pub fn capacity(&self) -> usize {
        self.capacity
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
            let found = keys_data.iter()
                .take(self.current_size)  // Only search in valid entries
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
        Ok(keys_data.iter()
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
    fn test_cache_basic_operations() {
        let (device, context, queue) = create_test_opencl_context();
        let mut cache = GpuCache::new(device, context, queue, 100).unwrap();

        // Verify initial state
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.capacity(), 100);

        // Add data
        let keys = vec![[1, 2], [4, 5]];
        let values = vec![
            XPub {
                chain_code: [1u8; 32],
                k_par: PointGpu {
                    x: Uint256 { limbs: [10, 0, 0, 0] },
                    y: Uint256 { limbs: [20, 0, 0, 0] },
                },
            },
            XPub {
                chain_code: [2u8; 32],
                k_par: PointGpu {
                    x: Uint256 { limbs: [30, 0, 0, 0] },
                    y: Uint256 { limbs: [40, 0, 0, 0] },
                },
            },
        ];
        cache.add_data(&keys, &values).unwrap();

        assert_eq!(cache.size(), 2);

        // Lookup existing key
        let results = cache.lookup(&[[1, 2]]).unwrap();
        assert!(results[0].is_some());
        let found = results[0].unwrap();
        assert_eq!(found.chain_code[0], 1);
        assert_eq!(found.k_par.x.limbs[0], 10);
        assert_eq!(found.k_par.y.limbs[0], 20);

        // Lookup non-existing key
        let results = cache.lookup(&[[99, 99]]).unwrap();
        assert!(results[0].is_none());
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

    #[test]
    fn test_cache_capacity() {
        let (device, context, queue) = create_test_opencl_context();
        let mut cache = GpuCache::new(device, context, queue, 10).unwrap();

        let keys: Vec<[u32; 2]> = (0..15).map(|i| [i, 0]).collect();
        let values: Vec<XPub> = (0..15).map(|_| XPub::default()).collect();

        // Should fail when exceeding capacity
        let result = cache.add_data(&keys, &values);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("capacity exceeded"));
    }

    #[test]
    fn test_multiple_lookups() {
        let (device, context, queue) = create_test_opencl_context();
        let mut cache = GpuCache::new(device, context, queue, 100).unwrap();

        // Add multiple entries
        let keys = vec![[0, 0], [0, 1], [0, 2], [1, 0]];
        let values: Vec<XPub> = keys
            .iter()
            .map(|k| XPub {
                chain_code: [k[0] as u8; 32],
                k_par: PointGpu {
                    x: Uint256 {
                        limbs: [k[1] as u64, 0, 0, 0],
                    },
                    y: Uint256 {
                        limbs: [0, 0, 0, 0],
                    },
                },
            })
            .collect();

        cache.add_data(&keys, &values).unwrap();

        // Lookup multiple keys at once
        let search = vec![[0, 1], [1, 0], [99, 99]];
        let results = cache.lookup(&search).unwrap();

        assert_eq!(results.len(), 3);
        assert!(results[0].is_some());
        assert_eq!(results[0].unwrap().chain_code[0], 0);
        assert_eq!(results[0].unwrap().k_par.x.limbs[0], 1);

        assert!(results[1].is_some());
        assert_eq!(results[1].unwrap().chain_code[0], 1);
        assert_eq!(results[1].unwrap().k_par.x.limbs[0], 0);

        assert!(results[2].is_none());
    }

    #[test]
    fn test_cache_clear() {
        let (device, context, queue) = create_test_opencl_context();
        let mut cache = GpuCache::new(device, context, queue, 100).unwrap();

        let keys = vec![[1, 2]];
        let values = vec![XPub::default()];
        cache.add_data(&keys, &values).unwrap();

        assert_eq!(cache.size(), 1);

        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_add_data_length_mismatch() {
        let (device, context, queue) = create_test_opencl_context();
        let mut cache = GpuCache::new(device, context, queue, 100).unwrap();

        let keys = vec![[1, 2], [4, 5]];
        let values = vec![XPub::default()]; // Only 1 value for 2 keys

        let result = cache.add_data(&keys, &values);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("length mismatch"));
    }
}
