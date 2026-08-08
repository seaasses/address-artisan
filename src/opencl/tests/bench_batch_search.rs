#[cfg(test)]
mod tests {
    use crate::extended_public_key::ExtendedPubKey;
    use crate::extended_public_key_deriver::ExtendedPublicKeyDeriver;
    use crate::opencl::cache_preloader::CachePreloader;
    use crate::opencl::cache_range_analyzer::CacheRangeAnalyzer;
    use crate::opencl::g_tables;
    use crate::opencl::gpu_cache::{GpuCache, Hash160RangeGpu};
    use crate::prefix::Prefix;
    use ocl::enums::{
        KernelWorkGroupInfo, KernelWorkGroupInfoResult, ProgramBuildInfo, ProgramBuildInfoResult,
    };
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

    /// True for NVIDIA devices, which support `-cl-nv-verbose` (ptxas
    /// register/spill dump) and allocate local memory per *resident* thread.
    fn is_nvidia(device: Device) -> bool {
        device
            .name()
            .map(|n| n.to_ascii_uppercase().contains("NVIDIA"))
            .unwrap_or(false)
    }

    /// Threads for the constant-threads sweep: enough to fully saturate the
    /// device, but not so many that a spill-heavy high-PPT kernel exhausts a
    /// small GPU's memory. NVIDIA allocates local memory per resident thread, so
    /// a big launch is safe there; a 4 GB mobile part is not, so we cap it.
    fn saturating_threads(device: Device) -> usize {
        if is_nvidia(device) {
            524_288
        } else {
            65_536
        }
    }

    /// Builds the kernel program for a given PPT. On NVIDIA adds `-cl-nv-verbose`
    /// so ptxas prints register/spill counts into the build log. The flag is
    /// gated on vendor (not try-fail-rebuild) so non-NVIDIA devices compile the
    /// huge kernel exactly once. Returns the program and its build log.
    fn build_program(context: &Context, device: Device, ppt: u64) -> Option<(Program, String)> {
        let src = include_str!(concat!(env!("OUT_DIR"), "/batch_address_search"));
        let mut builder = Program::builder();
        builder
            .cmplr_opt(format!("-D POINTS_PER_THREAD={}", ppt))
            .src(src)
            .devices(device);
        if is_nvidia(device) {
            builder.cmplr_opt("-cl-nv-verbose");
        }

        let program = match builder.build(context) {
            Ok(p) => p,
            Err(e) => {
                println!("  ppt={:>2}  BUILD FAILED: {}", ppt, e);
                return None;
            }
        };

        let log = match program.build_info(device, ProgramBuildInfo::BuildLog) {
            Ok(ProgramBuildInfoResult::BuildLog(s)) => s,
            _ => String::new(),
        };
        Some((program, log))
    }

    /// Prints per-PPT occupancy signals: private (register/local) memory the
    /// runtime reports for the kernel, plus any register/spill lines NVIDIA's
    /// verbose ptxas dumps into the build log. This is what tells us whether a
    /// rising PPT is actually spilling registers (→ kernel split would help) vs.
    /// just running fewer threads (→ a benchmark artifact).
    fn report_registers(program: &Program, kernel: &Kernel, device: Device, ppt: u64) {
        let priv_mem = match kernel.wg_info(device, KernelWorkGroupInfo::PrivateMemSize) {
            Ok(KernelWorkGroupInfoResult::PrivateMemSize(n)) => n,
            _ => 0,
        };
        println!("  ppt={:>2}  private_mem={} bytes/work-item", ppt, priv_mem);

        if let Ok(ProgramBuildInfoResult::BuildLog(log)) =
            program.build_info(device, ProgramBuildInfo::BuildLog)
        {
            for line in log.lines() {
                let l = line.to_ascii_lowercase();
                if l.contains("register") || l.contains("spill") || l.contains("stack") {
                    println!("           {}", line.trim());
                }
            }
        }
    }

    /// Runs one configuration.
    ///
    /// `threads` is the OpenCL global work size; each thread processes `ppt`
    /// points, so the batch covers `threads * ppt` counters. The throughput
    /// denominator is that same `threads * ppt`, so addr/s is comparable across
    /// configs regardless of how work is split between threads and PPT.
    fn bench_one(ppt: u64, max_depth: u32, threads: usize) {
        let (device, context, queue) = ctx();
        let total_points = threads * ppt as usize;

        // Realistic cache: exactly what GpuWorkbench would preload for this batch.
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let mut deriver = ExtendedPublicKeyDeriver::new(&xpub);
        let cache_keys =
            CacheRangeAnalyzer::analyze_counter_range(0, total_points as u64, max_depth);

        let mut gpu_cache =
            GpuCache::new(device, context.clone(), queue.clone(), 1_000_000).unwrap();
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
            Buffer::<u8>::builder()
                .queue(queue.clone())
                .len(len)
                .build()
                .unwrap()
        };
        let nu = |len: usize| -> Buffer<u32> {
            Buffer::<u32>::builder()
                .queue(queue.clone())
                .len(len)
                .build()
                .unwrap()
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

        let (program, _log) = match build_program(&context, device, ppt) {
            Some(p) => p,
            None => return,
        };

        let (ck, cv, cs) = gpu_cache.get_buffers();
        let mut kb = Kernel::builder();
        kb.program(&program)
            .name("batch_address_search")
            .queue(queue.clone())
            .global_work_size(threads)
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

        report_registers(&program, &kernel, device, ppt);

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

        let total = (total_points * TIMED_RUNS) as f64;
        let rate = total / elapsed.as_secs_f64();
        println!(
            "  ppt={:>2}  threads={:>7}  {:>12.0} addr/s",
            ppt, threads, rate
        );
    }

    /// ORIGINAL sweep: fixed total work (WORK_SIZE), threads = WORK_SIZE / ppt.
    /// This is the "fair per-batch" view but at high PPT the thread count drops
    /// (e.g. 32k threads at ppt=16), which can starve a big GPU and mask the true
    /// PPT effect. Compare against the constant-threads sweep below.
    ///
    /// cargo test --release -- --ignored bench_batch_search_sweep --nocapture --test-threads=1
    #[test]
    #[ignore]
    fn bench_batch_search_sweep() {
        let (device, _, _) = ctx();
        println!("Device: {}", device.name().unwrap_or_default());
        println!("== fixed total work (threads = {} / ppt) ==", WORK_SIZE);
        for max_depth in [1000u32, 100_000] {
            println!("max_depth = {}:", max_depth);
            for ppt in [1u64, 2, 4, 8, 16] {
                bench_one(ppt, max_depth, WORK_SIZE / ppt as usize);
            }
        }
    }

    /// CONSTANT-THREADS sweep: a fixed, device-saturating thread count for every
    /// PPT, so the GPU is always fully occupied and the only thing changing is
    /// how many points each thread batches through one shared inversion. This
    /// isolates the batched inversion's real effect from the thread-starvation
    /// confound. If addr/s still climbs with PPT here, there is a genuine win
    /// beyond ppt=2 that has nothing to do with running fewer threads.
    ///
    /// cargo test --release -- --ignored bench_batch_search_constant_threads --nocapture --test-threads=1
    #[test]
    #[ignore]
    fn bench_batch_search_constant_threads() {
        let (device, _, _) = ctx();
        let threads = saturating_threads(device);
        println!("Device: {}", device.name().unwrap_or_default());
        println!(
            "== constant threads = {} (batch = threads * ppt) ==",
            threads
        );
        for max_depth in [1000u32, 100_000] {
            println!("max_depth = {}:", max_depth);
            for ppt in [1u64, 2, 4, 8, 16] {
                bench_one(ppt, max_depth, threads);
            }
        }
    }
}
