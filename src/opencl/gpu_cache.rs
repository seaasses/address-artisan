use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

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
    lookup_kernel: Kernel,
    keys_buffer: Buffer<CacheKey>,
    values_buffer: Buffer<XPub>,
    capacity: usize,
    current_size: usize,
}

impl GpuCache {
    pub fn new(capacity: usize) -> Result<Self, String> {
        let (device, context, queue) = Self::get_device_context_and_queue()?;

        let keys_buffer = Self::new_buffer::<CacheKey>(&queue, capacity)?;
        let values_buffer = Self::new_buffer::<XPub>(&queue, capacity)?;

        let program = Self::build_program(device, context.clone())?;

        let dummy_search = Self::new_buffer::<CacheKey>(&queue, 1)?;
        let dummy_output = Self::new_buffer::<XPub>(&queue, 1)?;
        let dummy_flags = Self::new_buffer::<i32>(&queue, 1)?;

        let lookup_kernel = Kernel::builder()
            .program(&program)
            .name("cache_lookup")
            .queue(queue.clone())
            .global_work_size(1)
            .arg(&keys_buffer)
            .arg(&values_buffer)
            .arg(&dummy_search)
            .arg(&dummy_output)
            .arg(&dummy_flags)
            .arg(0u32)
            .arg(0u32)
            .build()
            .map_err(|e| format!("Error creating lookup kernel: {}", e))?;

        Ok(Self {
            _device: device,
            _context: context,
            queue,
            lookup_kernel,
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

    pub fn lookup(&self, search_keys: &[[u32; 2]]) -> Result<Vec<Option<XPub>>, String> {
        if search_keys.is_empty() {
            return Ok(vec![]);
        }

        let search_count = search_keys.len();

        let cache_search_keys: Vec<CacheKey> = search_keys
            .iter()
            .map(|k| CacheKey { b: k[0], a: k[1] })
            .collect();

        let search_buffer = Self::new_buffer::<CacheKey>(&self.queue, search_count)?;
        let output_buffer = Self::new_buffer::<XPub>(&self.queue, search_count)?;
        let found_flags_buffer = Self::new_buffer::<i32>(&self.queue, search_count)?;

        search_buffer
            .write(&cache_search_keys)
            .enq()
            .map_err(|e| format!("Error writing search keys: {}", e))?;
        self.lookup_kernel
            .set_arg(0, &self.keys_buffer)
            .map_err(|e| format!("Error setting keys_buffer arg: {}", e))?;
        self.lookup_kernel
            .set_arg(1, &self.values_buffer)
            .map_err(|e| format!("Error setting values_buffer arg: {}", e))?;
        self.lookup_kernel
            .set_arg(2, &search_buffer)
            .map_err(|e| format!("Error setting search_buffer arg: {}", e))?;
        self.lookup_kernel
            .set_arg(3, &output_buffer)
            .map_err(|e| format!("Error setting output_buffer arg: {}", e))?;
        self.lookup_kernel
            .set_arg(4, &found_flags_buffer)
            .map_err(|e| format!("Error setting found_flags_buffer arg: {}", e))?;
        self.lookup_kernel
            .set_arg(5, self.current_size as u32)
            .map_err(|e| format!("Error setting cache_size arg: {}", e))?;
        self.lookup_kernel
            .set_arg(6, search_count as u32)
            .map_err(|e| format!("Error setting search_count arg: {}", e))?;

        unsafe {
            self.lookup_kernel
                .cmd()
                .global_work_size(search_count)
                .enq()
                .map_err(|e| format!("Error executing lookup kernel: {}", e))?;
        }

        let mut output_values = vec![XPub::default(); search_count];
        let mut found_flags = vec![0i32; search_count];

        output_buffer
            .read(&mut output_values)
            .enq()
            .map_err(|e| format!("Error reading output values: {}", e))?;

        found_flags_buffer
            .read(&mut found_flags)
            .enq()
            .map_err(|e| format!("Error reading found flags: {}", e))?;

        let results: Vec<Option<XPub>> = output_values
            .into_iter()
            .zip(found_flags.into_iter())
            .map(|(value, flag)| if flag == 1 { Some(value) } else { None })
            .collect();

        Ok(results)
    }

    pub fn contains_key(&self, key: &[u32; 2]) -> Result<bool, String> {
        let results = self.lookup(&[*key])?;
        Ok(results[0].is_some())
    }

    pub fn clear(&mut self) {
        self.current_size = 0;
    }

    pub fn size(&self) -> usize {
        self.current_size
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    fn new_buffer<T: ocl::OclPrm>(queue: &Queue, len: usize) -> Result<Buffer<T>, String> {
        Buffer::<T>::builder()
            .queue(queue.clone())
            .len(len)
            .build()
            .map_err(|e| format!("Error creating buffer: {}", e))
    }

    fn build_program(device: Device, context: Context) -> Result<Program, String> {
        let src = include_str!(concat!(env!("OUT_DIR"), "/cache_lookup"));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let mut cache = GpuCache::new(100).unwrap();

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
        let cache = GpuCache::new(100).unwrap();

        // Lookup in empty cache
        let results = cache.lookup(&[[1, 2]]).unwrap();
        assert!(results[0].is_none());

        // contains_key on empty cache
        assert!(!cache.contains_key(&[1, 2]).unwrap());
    }

    #[test]
    fn test_cache_capacity() {
        let mut cache = GpuCache::new(10).unwrap();

        let keys: Vec<[u32; 2]> = (0..15).map(|i| [i, 0]).collect();
        let values: Vec<XPub> = (0..15).map(|_| XPub::default()).collect();

        // Should fail when exceeding capacity
        let result = cache.add_data(&keys, &values);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("capacity exceeded"));
    }

    #[test]
    fn test_multiple_lookups() {
        let mut cache = GpuCache::new(100).unwrap();

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
        let mut cache = GpuCache::new(100).unwrap();

        let keys = vec![[1, 2]];
        let values = vec![XPub::default()];
        cache.add_data(&keys, &values).unwrap();

        assert_eq!(cache.size(), 1);

        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_add_data_length_mismatch() {
        let mut cache = GpuCache::new(100).unwrap();

        let keys = vec![[1, 2], [4, 5]];
        let values = vec![XPub::default()]; // Only 1 value for 2 keys

        let result = cache.add_data(&keys, &values);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("length mismatch"));
    }
}
