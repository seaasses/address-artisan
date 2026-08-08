#[cfg(test)]
mod tests {
    use crate::extended_public_key::ExtendedPubKey;
    use crate::extended_public_key_deriver::ExtendedPublicKeyDeriver;
    use crate::opencl::cache_preloader::CachePreloader;
    use crate::opencl::cache_range_analyzer::CacheRangeAnalyzer;
    use crate::opencl::g_tables;
    use crate::opencl::gpu_cache::{GpuCache, Hash160RangeGpu};
    use crate::prefix::Prefix;
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};
    use std::time::Instant;

    const WORK_SIZE: usize = 524_288; // same batch size as GpuWorkbench
    const MAX_MATCHES: usize = 1000;
    const WARMUP_RUNS: usize = 2;
    const TIMED_RUNS: usize = 10;

    fn ctx() -> (Device, Context, Queue) {
        let platform = Platform::first().expect("No OpenCL platform");
        let device = Device::first(platform).expect("No OpenCL device");
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()
            .unwrap();
        let queue = Queue::new(&context, device, None).unwrap();
        (device, context, queue)
    }

    fn bench_one(ppt: u64, max_depth: u32) {
        let (device, context, queue) = ctx();

        // Realistic cache: exactly what GpuWorkbench preloads for one batch.
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);
        let cache_keys = CacheRangeAnalyzer::analyze_counter_range(0, WORK_SIZE as u64, max_depth);

        let mut gpu_cache = GpuCache::new(device, context.clone(), queue.clone(), 1_000_000).unwrap();
        CachePreloader::preload(&mut gpu_cache, &cache_keys, &mut deriver, 12345, 67890).unwrap();

        // Near-impossible prefix: whole pipeline runs, ~zero matches stored.
        let prefix = Prefix::new("1ZZZZZZZZZ").unwrap();
        let gpu_ranges: Vec<Hash160RangeGpu> = prefix
            .ranges
            .iter()
            .map(|r| Hash160RangeGpu {
                low: r.low,
                high: r.high,
                prefix_id: 0,
            })
            .collect();

        let ranges_buffer = Buffer::<Hash160RangeGpu>::builder()
            .queue(queue.clone())
            .len(gpu_ranges.len())
            .copy_host_slice(&gpu_ranges)
            .build()
            .unwrap();
        let nb = |len: usize| -> Buffer<u8> {
            Buffer::<u8>::builder().queue(queue.clone()).len(len).build().unwrap()
        };
        let nu = |len: usize| -> Buffer<u32> {
            Buffer::<u32>::builder().queue(queue.clone()).len(len).build().unwrap()
        };
        let matches_hash160 = nb(MAX_MATCHES * 20);
        let matches_b = nu(MAX_MATCHES);
        let matches_a = nu(MAX_MATCHES);
        let matches_index = nu(MAX_MATCHES);
        let matches_prefix_id = nb(MAX_MATCHES);
        let match_count = nu(1);
        let cache_miss = nu(1);
        match_count.cmd().fill(0u32, None).enq().unwrap();
        cache_miss.cmd().fill(0u32, None).enq().unwrap();
        let g_tables_buf = g_tables::create_g_tables_buffer(&queue).unwrap();

        let src = include_str!(concat!(env!("OUT_DIR"), "/batch_address_search"));
        let program = match Program::builder()
            .cmplr_opt(format!("-D POINTS_PER_THREAD={}", ppt))
            .src(src)
            .devices(device)
            .build(&context)
        {
            Ok(p) => p,
            Err(e) => {
                println!("  ppt={:>2}  BUILD FAILED: {}", ppt, e);
                return;
            }
        };

        let (ck, cv, cs) = gpu_cache.get_buffers();
        let mut kb = Kernel::builder();
        kb.program(&program)
            .name("batch_address_search")
            .queue(queue.clone())
            .global_work_size(WORK_SIZE / ppt as usize)
            .arg(ck)
            .arg(cv)
            .arg(&ranges_buffer)
            .arg(gpu_ranges.len() as u32)
            .arg(cs)
            .arg(0u64)
            .arg(max_depth)
            .arg(&matches_hash160)
            .arg(&matches_b)
            .arg(&matches_a)
            .arg(&matches_index)
            .arg(&matches_prefix_id)
            .arg(&match_count)
            .arg(&cache_miss)
            .arg(&g_tables_buf);
        unsafe {
            kb.disable_arg_type_check();
        }
        let kernel = kb.build().unwrap();

        for _ in 0..WARMUP_RUNS {
            unsafe { kernel.enq().unwrap() };
        }
        queue.finish().unwrap();

        let start = Instant::now();
        for _ in 0..TIMED_RUNS {
            unsafe { kernel.enq().unwrap() };
        }
        queue.finish().unwrap();
        let elapsed = start.elapsed();

        let mut miss = vec![0u32; 1];
        cache_miss.read(&mut miss).enq().unwrap();
        assert_eq!(miss[0], 0, "cache misses invalidate the benchmark");

        let total = (WORK_SIZE * TIMED_RUNS) as f64;
        let rate = total / elapsed.as_secs_f64();
        println!("  ppt={:>2}  {:>10.0} addr/s", ppt, rate);
    }

    /// Sweeps POINTS_PER_THREAD to find the batched-inversion sweet spot.
    /// cargo test --release -- --ignored bench_batch_search_sweep --nocapture --test-threads=1
    #[test]
    #[ignore]
    fn bench_batch_search_sweep() {
        let (device, _, _) = ctx();
        println!("Device: {}", device.name().unwrap_or_default());
        for max_depth in [1000u32, 100_000] {
            println!("max_depth = {}:", max_depth);
            for ppt in [1u64, 2, 4, 8, 16] {
                bench_one(ppt, max_depth);
            }
        }
    }
}
