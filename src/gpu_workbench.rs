use crate::events::EventSender;
use crate::extended_public_key_deriver::ExtendedPublicKeyDeriver;
use crate::opencl::cache_preloader::CachePreloader;
use crate::opencl::cache_range_analyzer::CacheRangeAnalyzer;
use crate::opencl::gpu_cache::GpuCache;
use crate::workbench::Workbench;
use crate::workbench_config::WorkbenchConfig;
use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// GPU processing constants
const GPU_WORK_SIZE: u64 = 524_288;
const CACHE_CAPACITY: usize = 1_000_000; // ~ 100 MB
const MAX_MATCHES: usize = 1000; // Max matches per kernel call
const REPORT_INTERVAL: Duration = Duration::from_millis(1000);

pub struct GpuWorkbench {
    config: WorkbenchConfig,
    event_sender: EventSender,
    stop_signal: Arc<AtomicBool>,
    global_generated: Arc<AtomicU64>,
    device_index: usize,
    platform_index: usize,
}

impl GpuWorkbench {
    pub fn new(
        config: WorkbenchConfig,
        event_sender: EventSender,
        stop_signal: Arc<AtomicBool>,
        device_index: usize,
        platform_index: usize,
    ) -> Self {
        Self {
            config,
            event_sender,
            stop_signal,
            global_generated: Arc::new(AtomicU64::new(0)),
            device_index,
            platform_index,
        }
    }

    fn worker_loop(
        config: WorkbenchConfig,
        stop_signal: Arc<AtomicBool>,
        global_generated: Arc<AtomicU64>,
        event_sender: EventSender,
        device_index: usize,
        platform_index: usize,
    ) {
        let start_time = Instant::now();

        // Initialize OpenCL context and queue FIRST
        let (device, context, queue) = match Self::init_opencl(device_index, platform_index) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to initialize OpenCL: {}", e);
                return;
            }
        };

        // Initialize GPU cache using the same device/context/queue
        let mut gpu_cache =
            match GpuCache::new(device, context.clone(), queue.clone(), CACHE_CAPACITY) {
                Ok(cache) => cache,
                Err(e) => {
                    eprintln!("Failed to create GPU cache: {}", e);
                    return;
                }
            };

        let mut deriver = ExtendedPublicKeyDeriver::new(&config.xpub);

        // Build kernel program (EXPENSIVE - do it once!)
        let program = match Self::build_kernel_program(device, context.clone()) {
            Ok(prog) => prog,
            Err(e) => {
                eprintln!("Failed to build kernel program: {}", e);
                return;
            }
        };

        // Prepare range buffers from prefix
        let (range_lows_data, range_highs_data) = Self::prepare_range_buffers(&config.prefix);
        let range_count = config.prefix.ranges.len() as u32;

        // Create ALL GPU buffers ONCE before the loop
        let range_lows_buffer = match Buffer::<u8>::builder()
            .queue(queue.clone())
            .len(range_lows_data.len())
            .copy_host_slice(&range_lows_data)
            .build()
        {
            Ok(buf) => buf,
            Err(_) => return,
        };

        let range_highs_buffer = match Buffer::<u8>::builder()
            .queue(queue.clone())
            .len(range_highs_data.len())
            .copy_host_slice(&range_highs_data)
            .build()
        {
            Ok(buf) => buf,
            Err(_) => return,
        };

        let matches_hash160_buffer = match Buffer::<u8>::builder()
            .queue(queue.clone())
            .len(MAX_MATCHES * 20)
            .build()
        {
            Ok(buf) => buf,
            Err(_) => return,
        };

        let matches_b_buffer = match Buffer::<u32>::builder()
            .queue(queue.clone())
            .len(MAX_MATCHES)
            .build()
        {
            Ok(buf) => buf,
            Err(_) => return,
        };

        let matches_a_buffer = match Buffer::<u32>::builder()
            .queue(queue.clone())
            .len(MAX_MATCHES)
            .build()
        {
            Ok(buf) => buf,
            Err(_) => return,
        };

        let matches_index_buffer = match Buffer::<u32>::builder()
            .queue(queue.clone())
            .len(MAX_MATCHES)
            .build()
        {
            Ok(buf) => buf,
            Err(_) => return,
        };

        let match_count_buffer = match Buffer::<u32>::builder().queue(queue.clone()).len(1).build()
        {
            Ok(buf) => buf,
            Err(_) => return,
        };

        let cache_miss_error_buffer =
            match Buffer::<u32>::builder().queue(queue.clone()).len(1).build() {
                Ok(buf) => buf,
                Err(_) => return,
            };

        // Get initial cache buffers for kernel creation
        let (initial_cache_keys, initial_cache_values, initial_cache_size) =
            gpu_cache.get_buffers();

        // Create kernel ONCE before the loop
        let kernel = match Kernel::builder()
            .program(&program)
            .name("batch_address_search")
            .queue(queue.clone())
            .global_work_size(GPU_WORK_SIZE as usize)
            .arg(initial_cache_keys) // arg 0 - will update in loop
            .arg(initial_cache_values) // arg 1 - will update in loop
            .arg(&range_lows_buffer) // arg 2 - fixed
            .arg(&range_highs_buffer) // arg 3 - fixed
            .arg(range_count) // arg 4 - fixed
            .arg(initial_cache_size as u32) // arg 5 - will update in loop
            .arg(0u64) // arg 6 - start_counter, will update in loop
            .arg(config.max_depth) // arg 7 - fixed
            .arg(&matches_hash160_buffer) // arg 8 - fixed
            .arg(&matches_b_buffer) // arg 9 - fixed
            .arg(&matches_a_buffer) // arg 10 - fixed
            .arg(&matches_index_buffer) // arg 11 - fixed
            .arg(&match_count_buffer) // arg 12 - fixed (reset separately)
            .arg(&cache_miss_error_buffer) // arg 13 - fixed (reset separately)
            .build()
        {
            Ok(k) => k,
            Err(_) => return,
        };

        let mut counter = 0u64;
        let mut last_report = Instant::now();
        let mut generated_since_last_report = 0u64;

        while !stop_signal.load(Ordering::Relaxed) {
            // Clear and reload cache for this batch
            gpu_cache.clear();

            let cache_keys =
                CacheRangeAnalyzer::analyze_counter_range(counter, GPU_WORK_SIZE, config.max_depth);

            if let Err(e) = CachePreloader::preload(
                &mut gpu_cache,
                &cache_keys,
                &mut deriver,
                config.seed0,
                config.seed1,
            ) {
                eprintln!("Cache preload error: {}", e);
                break;
            }
            // CRITICAL: Ensure cache writes completed before kernel execution
            if let Err(e) = queue.finish() {
                eprintln!("Failed to sync cache writes: {}", e);
                break;
            }

            // Reset match counters for this batch
            if let Err(e) = match_count_buffer.cmd().fill(0u32, None).enq() {
                eprintln!("Failed to reset match_count: {}", e);
                break;
            }
            if let Err(e) = cache_miss_error_buffer.cmd().fill(0u32, None).enq() {
                eprintln!("Failed to reset cache_miss_error: {}", e);
                break;
            }

            // CRITICAL: Ensure reset operations completed
            if let Err(e) = queue.finish() {
                eprintln!("Failed to sync buffer resets: {}", e);
                break;
            }

            // Get current cache buffers and update kernel args
            let (cache_keys_buffer, cache_values_buffer, cache_size) = gpu_cache.get_buffers();

            // Update only the args that change each iteration
            if let Err(e) = kernel.set_arg(0, cache_keys_buffer) {
                eprintln!("Failed to set cache_keys arg: {}", e);
                break;
            }
            if let Err(e) = kernel.set_arg(1, cache_values_buffer) {
                eprintln!("Failed to set cache_values arg: {}", e);
                break;
            }
            if let Err(e) = kernel.set_arg(5, cache_size as u32) {
                eprintln!("Failed to set cache_size arg: {}", e);
                break;
            }
            if let Err(e) = kernel.set_arg(6, counter) {
                eprintln!("Failed to set start_counter arg: {}", e);
                break;
            }

            if let Err(e) = unsafe { kernel.enq() } {
                eprintln!("Failed to execute kernel: {}", e);
                break;
            }

            // Wait for kernel completion before timing
            if let Err(e) = queue.finish() {
                eprintln!("Failed to finish queue: {}", e);
                break;
            }

            // Check for cache miss errors
            let mut cache_miss_error = vec![0u32; 1];
            if let Err(e) = cache_miss_error_buffer.read(&mut cache_miss_error).enq() {
                eprintln!("Failed to read cache_miss_error: {}", e);
                break;
            }

            if cache_miss_error[0] != 0 {
                eprintln!(
                    "CACHE MISS ERROR: {} lookups failed! Stopping.",
                    cache_miss_error[0]
                );
                break;
            }

            // Read match count
            let mut match_count = vec![0u32; 1];
            if let Err(e) = match_count_buffer.read(&mut match_count).enq() {
                eprintln!("Failed to read match count: {}", e);
                break;
            }

            let num_matches = match_count[0].min(MAX_MATCHES as u32) as usize;

            if num_matches > 0 {
                // Read matches
                let mut matches_hash160_data = vec![0u8; num_matches * 20];
                let mut matches_b_data = vec![0u32; num_matches];
                let mut matches_a_data = vec![0u32; num_matches];
                let mut matches_index_data = vec![0u32; num_matches];

                if let Err(e) = matches_hash160_buffer.read(&mut matches_hash160_data).enq() {
                    eprintln!("Failed to read hash160: {}", e);
                    break;
                }
                if let Err(e) = matches_b_buffer.read(&mut matches_b_data).enq() {
                    eprintln!("Failed to read b: {}", e);
                    break;
                }
                if let Err(e) = matches_a_buffer.read(&mut matches_a_data).enq() {
                    eprintln!("Failed to read a: {}", e);
                    break;
                }
                if let Err(e) = matches_index_buffer.read(&mut matches_index_data).enq() {
                    eprintln!("Failed to read index: {}", e);
                    break;
                }

                // Process matches
                for i in 0..num_matches {
                    let b = matches_b_data[i];
                    let a = matches_a_data[i];
                    let index = matches_index_data[i];
                    let path = [config.seed0, config.seed1, b, a, 0, index];
                    event_sender.potential_match(path);
                }
            }

            // Update counters
            generated_since_last_report += GPU_WORK_SIZE;
            global_generated.fetch_add(GPU_WORK_SIZE, Ordering::Relaxed);

            // Report progress
            if last_report.elapsed() >= REPORT_INTERVAL {
                event_sender.progress(generated_since_last_report);
                generated_since_last_report = 0;
                last_report = Instant::now();
            }

            counter += GPU_WORK_SIZE;
        }

        // Send stopped event with final stats
        let total_generated = global_generated.load(Ordering::Relaxed);
        let elapsed = start_time.elapsed();
        event_sender.stopped(total_generated, elapsed);
    }

    fn init_opencl(
        device_index: usize,
        platform_index: usize,
    ) -> Result<(Device, Context, Queue), String> {
        // Get platform by index
        let platforms = Platform::list();
        let platform = platforms.get(platform_index).ok_or_else(|| {
            format!(
                "Platform index {} not found (only {} platforms available)",
                platform_index,
                platforms.len()
            )
        })?;

        // Get device by index
        let devices =
            Device::list_all(*platform).map_err(|e| format!("Failed to list devices: {}", e))?;
        let device = devices.get(device_index).ok_or_else(|| {
            format!(
                "Device index {} not found (only {} devices available)",
                device_index,
                devices.len()
            )
        })?;

        let context = Context::builder()
            .platform(*platform)
            .devices(*device)
            .build()
            .map_err(|e| format!("Failed to create context: {}", e))?;
        let queue = Queue::new(&context, *device, None)
            .map_err(|e| format!("Failed to create queue: {}", e))?;

        Ok((*device, context, queue))
    }

    fn build_kernel_program(device: Device, context: Context) -> Result<Program, String> {
        let batch_search_src = include_str!(concat!(env!("OUT_DIR"), "/batch_address_search"));

        Program::builder()
            .devices(device)
            .src(batch_search_src)
            .build(&context)
            .map_err(|e| format!("Failed to build program: {}", e))
    }

    fn prepare_range_buffers(prefix: &crate::prefix::Prefix) -> (Vec<u8>, Vec<u8>) {
        let mut lows = Vec::with_capacity(prefix.ranges.len() * 20);
        let mut highs = Vec::with_capacity(prefix.ranges.len() * 20);

        for range in &prefix.ranges {
            lows.extend_from_slice(&range.low);
            highs.extend_from_slice(&range.high);
        }

        (lows, highs)
    }
}

impl Workbench for GpuWorkbench {
    fn start(&self) {
        Self::worker_loop(
            self.config.clone(),
            Arc::clone(&self.stop_signal),
            Arc::clone(&self.global_generated),
            self.event_sender.clone(),
            self.device_index,
            self.platform_index,
        );
    }

    fn wait(&self) {
        // No-op: work is already done in start()
    }

    fn total_generated(&self) -> u64 {
        self.global_generated.load(Ordering::Relaxed)
    }
}
