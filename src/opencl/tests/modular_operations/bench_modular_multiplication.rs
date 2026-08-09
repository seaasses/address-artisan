#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};
    use std::time::Instant;

    const WORK_SIZE: usize = 1 << 20;
    const ITERATIONS: u32 = 1000;
    const WARMUP: usize = 2;
    const TIMED: usize = 10;

    fn ctx() -> (Device, Context, Queue) {
        let platform = Platform::first().unwrap();
        let device = Device::first(platform).unwrap();
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()
            .unwrap();
        let queue = Queue::new(&context, device, None).unwrap();
        (device, context, queue)
    }

    /// Dependent-chain modular multiplication throughput (ALU-bound, not memory):
    /// each work-item runs ITERATIONS back-to-back modmuls where each depends on
    /// the previous. This is the go/no-go instrument for the 32-bit-limb rewrite.
    /// cargo test --release -- --ignored bench_modular_multiplication --nocapture --test-threads=1
    #[test]
    #[ignore]
    fn bench_modular_multiplication() {
        let (device, context, queue) = ctx();
        println!("Device: {}", device.name().unwrap_or_default());

        let a = [
            0x2eu8, 0x91, 0xa4, 0xf9, 0x33, 0xe5, 0x54, 0x1b, 0xfb, 0x13, 0xb2, 0x82, 0xb7, 0x44,
            0x67, 0x66, 0xdd, 0xed, 0x2e, 0xdd, 0x82, 0x5d, 0x3a, 0x88, 0xce, 0x88, 0x2f, 0x31,
            0x93, 0xa2, 0xcf, 0x1a,
        ];
        let b = [
            0x76u8, 0xba, 0x21, 0xd8, 0x24, 0x55, 0xfe, 0x6b, 0x7b, 0x64, 0xec, 0xe6, 0x41, 0x5b,
            0xcd, 0x77, 0xd4, 0xda, 0xc0, 0x60, 0x1a, 0xc6, 0xc3, 0x15, 0x6a, 0xfa, 0xb7, 0x48,
            0x5c, 0xc9, 0xe8, 0x3a,
        ];

        let nb = |d: &[u8]| {
            Buffer::<u8>::builder()
                .queue(queue.clone())
                .len(d.len())
                .copy_host_slice(d)
                .build()
                .unwrap()
        };
        let a_buf = nb(&a);
        let b_buf = nb(&b);
        let counter = Buffer::<u32>::builder()
            .queue(queue.clone())
            .len(1)
            .fill_val(0u32)
            .build()
            .unwrap();

        let src = include_str!(concat!(
            env!("OUT_DIR"),
            "/modular_multiplication_benchmark_kernel"
        ));
        let program = Program::builder()
            .src(src)
            .devices(device)
            .build(&context)
            .unwrap();

        let kernel = Kernel::builder()
            .program(&program)
            .name("modular_multiplication_benchmark_kernel")
            .queue(queue.clone())
            .global_work_size(WORK_SIZE)
            .arg(&a_buf)
            .arg(&b_buf)
            .arg(WORK_SIZE as u32)
            .arg(ITERATIONS)
            .arg(&counter)
            .build()
            .unwrap();

        for _ in 0..WARMUP {
            unsafe { kernel.enq().unwrap() };
        }
        queue.finish().unwrap();

        let start = Instant::now();
        for _ in 0..TIMED {
            unsafe { kernel.enq().unwrap() };
        }
        queue.finish().unwrap();
        let elapsed = start.elapsed();

        let total = (WORK_SIZE as f64) * (ITERATIONS as f64) * (TIMED as f64);
        println!(
            "modmul throughput: {:.0} modmul/s ({:.3}s)",
            total / elapsed.as_secs_f64(),
            elapsed.as_secs_f64()
        );
    }
}
