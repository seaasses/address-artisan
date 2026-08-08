#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};
    use std::time::Instant;

    const WORK_SIZE: usize = 1 << 20;
    const WARMUP_RUNS: usize = 2;
    const TIMED_RUNS: usize = 10;

    fn get_device_context_and_queue() -> Result<(Device, Context, Queue), String> {
        let platform = Platform::first().map_err(|e| e.to_string())?;
        let device = Device::first(platform).map_err(|e| e.to_string())?;
        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()
            .map_err(|e| e.to_string())?;
        let queue = Queue::new(&context, device, None).map_err(|e| e.to_string())?;
        Ok((device, context, queue))
    }

    /// Measures raw CKDpub throughput (the hot operation of the GPU search).
    /// Ignored by default; run with:
    /// cargo test --release -- --ignored bench_ckdpub_throughput --nocapture
    #[test]
    #[ignore]
    fn bench_ckdpub_throughput() {
        let (device, context, queue) = get_device_context_and_queue().unwrap();
        println!("Device: {}", device.name().unwrap_or_default());

        // Arbitrary but valid parent point: use the secp256k1 generator G
        let chain_code = [0x42u8; 32];
        let k_par_x: [u8; 32] = [
            0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87,
            0x0B, 0x07, 0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B,
            0x16, 0xF8, 0x17, 0x98,
        ];
        let k_par_y: [u8; 32] = [
            0x48, 0x3A, 0xDA, 0x77, 0x26, 0xA3, 0xC4, 0x65, 0x5D, 0xA4, 0xFB, 0xFC, 0x0E, 0x11,
            0x08, 0xA8, 0xFD, 0x17, 0xB4, 0x48, 0xA6, 0x85, 0x54, 0x19, 0x9C, 0x47, 0xD0, 0x8F,
            0xFB, 0x10, 0xD4, 0xB8,
        ];

        let new_u8_buffer = |data: &[u8]| -> Buffer<u8> {
            Buffer::<u8>::builder()
                .queue(queue.clone())
                .len(data.len())
                .copy_host_slice(data)
                .build()
                .unwrap()
        };

        let chain_code_buffer = new_u8_buffer(&chain_code);
        let k_par_x_buffer = new_u8_buffer(&k_par_x);
        let k_par_y_buffer = new_u8_buffer(&k_par_y);

        let counter_buffer = Buffer::<u32>::builder()
            .queue(queue.clone())
            .len(1)
            .fill_val(0u32)
            .build()
            .unwrap();

        let g_times_tables_buffer =
            crate::opencl::g_tables::create_g_tables_buffer(&queue).unwrap();

        let src = include_str!(concat!(
            env!("OUT_DIR"),
            "/ckdpub_throughput_benchmark_kernel"
        ));
        let program = Program::builder()
            .src(src)
            .devices(device)
            .build(&context)
            .unwrap();

        let mut kernel_builder = Kernel::builder();
        kernel_builder
            .program(&program)
            .name("ckdpub_throughput_benchmark_kernel")
            .queue(queue.clone())
            .global_work_size(WORK_SIZE)
            .arg(&chain_code_buffer)
            .arg(&k_par_x_buffer)
            .arg(&k_par_y_buffer)
            .arg(WORK_SIZE as u32)
            .arg(&counter_buffer)
            .arg(&g_times_tables_buffer);

        // ocl's arg type check parses "Point*" as an int pointer
        // ("Point" contains "int"), rejecting the tables buffer.
        unsafe {
            kernel_builder.disable_arg_type_check();
        }

        let kernel = kernel_builder.build().unwrap();

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

        let total_keys = (WORK_SIZE * TIMED_RUNS) as f64;
        let rate = total_keys / elapsed.as_secs_f64();
        println!(
            "CKDpub throughput: {:.0} keys/s ({} keys in {:.3}s)",
            rate,
            total_keys as u64,
            elapsed.as_secs_f64()
        );
    }
}
