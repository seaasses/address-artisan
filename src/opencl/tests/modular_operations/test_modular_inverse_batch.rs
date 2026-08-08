#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    const MAX_BATCH: usize = 32;

    // secp256k1 field prime p = 2^256 - 2^32 - 977
    fn field_prime() -> BigUint {
        BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F",
            16,
        )
        .unwrap()
    }

    fn to_be_bytes_32(value: &BigUint) -> [u8; 32] {
        let bytes = value.to_bytes_be();
        let mut out = [0u8; 32];
        out[32 - bytes.len()..].copy_from_slice(&bytes);
        out
    }

    struct BatchInverse {
        values_buffer: Buffer<u8>,
        results_buffer: Buffer<u8>,
        kernel: Kernel,
    }

    impl BatchInverse {
        fn new() -> Result<Self, String> {
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            let values_buffer = Buffer::<u8>::builder()
                .queue(queue.clone())
                .len(MAX_BATCH * 32)
                .build()
                .map_err(|e| e.to_string())?;
            let results_buffer = Buffer::<u8>::builder()
                .queue(queue.clone())
                .len(MAX_BATCH * 32)
                .build()
                .map_err(|e| e.to_string())?;

            let src = include_str!(concat!(env!("OUT_DIR"), "/modular_inverse_batch_kernel"));
            let program = Program::builder()
                .src(src)
                .devices(device)
                .build(&context)
                .map_err(|e| e.to_string())?;

            let kernel = Kernel::builder()
                .program(&program)
                .name("modular_inverse_batch_kernel")
                .queue(queue.clone())
                .global_work_size(1)
                .arg(&values_buffer)
                .arg(0u32) // count
                .arg(&results_buffer)
                .build()
                .map_err(|e| e.to_string())?;

            Ok(Self {
                values_buffer,
                results_buffer,
                kernel,
            })
        }

        fn invert(&mut self, values: &[BigUint]) -> Result<Vec<BigUint>, String> {
            let count = values.len();
            assert!(count <= MAX_BATCH);

            let mut flat = vec![0u8; MAX_BATCH * 32];
            for (i, v) in values.iter().enumerate() {
                flat[i * 32..i * 32 + 32].copy_from_slice(&to_be_bytes_32(v));
            }

            self.values_buffer
                .write(&flat[..])
                .enq()
                .map_err(|e| e.to_string())?;
            self.kernel.set_arg(1, count as u32).map_err(|e| e.to_string())?;

            unsafe {
                self.kernel.enq().map_err(|e| e.to_string())?;
            }

            let mut out = vec![0u8; MAX_BATCH * 32];
            self.results_buffer
                .read(&mut out[..])
                .enq()
                .map_err(|e| e.to_string())?;

            Ok((0..count)
                .map(|i| BigUint::from_bytes_be(&out[i * 32..i * 32 + 32]))
                .collect())
        }

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
    }

    /// Ground-truth inverse via Fermat: a^(p-2) mod p, computed independently
    /// from the GPU with num-bigint.
    fn reference_inverse(a: &BigUint, p: &BigUint) -> BigUint {
        a.modpow(&(p - 2u32), p)
    }

    fn assert_batch_correct(values: &[BigUint]) {
        let p = field_prime();
        let mut ocl = BatchInverse::new().unwrap();
        let got = ocl.invert(values).unwrap();

        assert_eq!(got.len(), values.len());
        for (i, (v, inv)) in values.iter().zip(got.iter()).enumerate() {
            // 1. a * a^-1 == 1 mod p (defining property)
            assert_eq!(
                (v * inv) % &p,
                BigUint::from(1u32),
                "value #{} ({}): a * inv != 1 mod p",
                i,
                v
            );
            // 2. matches the independently computed Fermat inverse
            assert_eq!(
                *inv,
                reference_inverse(v, &p),
                "value #{} ({}): batch inverse != a^(p-2)",
                i,
                v
            );
        }
    }

    #[test]
    fn test_batch_of_one_equals_plain_inverse() {
        assert_batch_correct(&[BigUint::from(3u32)]);
    }

    #[test]
    fn test_small_values() {
        let values: Vec<BigUint> = (1u32..=10).map(BigUint::from).collect();
        assert_batch_correct(&values);
    }

    #[test]
    fn test_full_batch_of_32() {
        // 32 distinct large values derived deterministically
        let p = field_prime();
        let base = BigUint::parse_bytes(
            b"1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF",
            16,
        )
        .unwrap();
        let values: Vec<BigUint> = (0u32..32)
            .map(|i| (&base * BigUint::from(i + 1)) % &p)
            .collect();
        assert_batch_correct(&values);
    }

    #[test]
    fn test_edge_values() {
        let p = field_prime();
        let values = vec![
            BigUint::from(1u32),          // inverse is 1
            BigUint::from(2u32),
            &p - 1u32,                    // p-1, its own inverse
            &p - 2u32,
            BigUint::parse_bytes(b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2E", 16).unwrap(),
        ];
        assert_batch_correct(&values);
    }

    #[test]
    fn test_various_batch_sizes() {
        // Same leading values, different counts: the peeling loop must be
        // correct for every length, not just the max.
        let p = field_prime();
        let base = BigUint::parse_bytes(
            b"DEADBEEFCAFEBABE0123456789ABCDEFFEDCBA98765432100011223344556677",
            16,
        )
        .unwrap();
        let all: Vec<BigUint> = (0u32..MAX_BATCH as u32)
            .map(|i| (&base + BigUint::from(i)) % &p)
            .collect();

        for count in [1usize, 2, 3, 7, 8, 15, 16, 31, 32] {
            assert_batch_correct(&all[..count]);
        }
    }
}
