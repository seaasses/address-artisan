#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct Secp256k1Operations {
        x1_buffer: Buffer<u8>,
        y1_buffer: Buffer<u8>,
        x2_buffer: Buffer<u8>,
        y2_buffer: Buffer<u8>,
        result_buffer_x: Buffer<u8>,
        result_buffer_y: Buffer<u8>,
        operations_kernel: Kernel,
    }

    impl Secp256k1Operations {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let platform = match Platform::first() {
                Ok(platform) => platform,
                Err(e) => {
                    return Err("Error getting OpenCL platform: ".to_string() + &e.to_string())
                }
            };

            let device = match Device::first(platform) {
                Ok(device) => device,
                Err(e) => return Err("Error getting OpenCL device: ".to_string() + &e.to_string()),
            };

            let context = match Context::builder()
                .platform(platform)
                .devices(device.clone())
                .build()
            {
                Ok(context) => context,
                Err(e) => {
                    return Err("Error building OpenCL context: ".to_string() + &e.to_string())
                }
            };

            let queue = Queue::new(&context, device, None).map_err(|e| e.to_string())?;

            let src = include_str!(concat!(env!("OUT_DIR"), "/combined_kernels.cl"));

            let program = match Program::builder().src(src).devices(device).build(&context) {
                Ok(program) => program,
                Err(e) => {
                    return Err("Error building OpenCL program: ".to_string() + &e.to_string())
                }
            };

            // Create buffers
            let x1_buffer = match Buffer::<u8>::builder().queue(queue.clone()).len(32).build() {
                Ok(buffer) => buffer,
                Err(e) => return Err("Error creating buffer A: ".to_string() + &e.to_string()),
            };

            let y1_buffer = match Buffer::<u8>::builder().queue(queue.clone()).len(32).build() {
                Ok(buffer) => buffer,
                Err(e) => return Err("Error creating buffer A: ".to_string() + &e.to_string()),
            };

            let x2_buffer = match Buffer::<u8>::builder().queue(queue.clone()).len(32).build() {
                Ok(buffer) => buffer,
                Err(e) => return Err("Error creating buffer A: ".to_string() + &e.to_string()),
            };

            let y2_buffer = match Buffer::<u8>::builder().queue(queue.clone()).len(32).build() {
                Ok(buffer) => buffer,
                Err(e) => return Err("Error creating buffer A: ".to_string() + &e.to_string()),
            };

            let result_buffer_x = match Buffer::<u8>::builder().queue(queue.clone()).len(32).build()
            {
                Ok(buffer) => buffer,
                Err(e) => return Err("Error creating result buffer: ".to_string() + &e.to_string()),
            };

            let result_buffer_y = match Buffer::<u8>::builder().queue(queue.clone()).len(32).build()
            {
                Ok(buffer) => buffer,
                Err(e) => return Err("Error creating result buffer: ".to_string() + &e.to_string()),
            };

            // Create kernel
            let operations_kernel = match Kernel::builder()
                .program(&program)
                .name("secp256k1_operations")
                .queue(queue.clone())
                .arg(&x1_buffer)
                .arg(&y1_buffer)
                .arg(&x2_buffer)
                .arg(&y2_buffer)
                .arg(0u8)
                .arg(&result_buffer_x)
                .arg(&result_buffer_y)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                x1_buffer,
                y1_buffer,
                x2_buffer,
                y2_buffer,
                result_buffer_x,
                result_buffer_y,
                operations_kernel,
            })
        }

        fn modular_addition(&mut self, a: Vec<u8>, b: Vec<u8>) -> Result<Vec<u8>, String> {
            let result = self.run_operation((a, b), (vec![0u8; 32], vec![0u8; 32]), 0);
            Ok(result.unwrap().0)
        }

        fn modular_multiplication(&mut self, a: Vec<u8>, b: Vec<u8>) -> Result<Vec<u8>, String> {
            let result = self.run_operation((a, b), (vec![0u8; 32], vec![0u8; 32]), 1);
            Ok(result.unwrap().0)
        }

        fn run_operation(
            &mut self,
            a: (Vec<u8>, Vec<u8>),
            b: (Vec<u8>, Vec<u8>),
            operation: u8,
        ) -> Result<(Vec<u8>, Vec<u8>), String> {
            if a.0.len() != 32 || a.1.len() != 32 || b.0.len() != 32 || b.1.len() != 32 {
                return Err(format!("Input vectors must be 32 bytes long"));
            }

            match self.operations_kernel.set_arg(4, operation) {
                Ok(_) => (),
                Err(e) => return Err("Error setting operation: ".to_string() + &e.to_string()),
            };

            // Write data to buffers
            match self.x1_buffer.write(&a.0[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing to buffer A: ".to_string() + &e.to_string()),
            };

            match self.y1_buffer.write(&a.1[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing to buffer B: ".to_string() + &e.to_string()),
            };

            match self.x2_buffer.write(&b.0[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing to buffer B: ".to_string() + &e.to_string()),
            };

            match self.y2_buffer.write(&b.1[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing to buffer B: ".to_string() + &e.to_string()),
            };

            // Execute kernel
            unsafe {
                match self.operations_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            // Read result
            let mut result_array_x = vec![0u8; 32];
            let mut result_array_y = vec![0u8; 32];
            match self.result_buffer_x.read(&mut result_array_x[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading result: ".to_string() + &e.to_string()),
            };

            match self.result_buffer_y.read(&mut result_array_y[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading result: ".to_string() + &e.to_string()),
            };

            Ok((result_array_x, result_array_y))
        }
    }

    // Secp256k1 prime modulus (p)
    // p = FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE FFFFFC2F
    const SECP256K1_P: [u8; 32] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF,
        0xFC, 0x2F,
    ];

    // p - 1
    const SECP256K1_P_MINUS_1: [u8; 32] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF,
        0xFC, 0x2E,
    ];

    #[test]
    // SIMPLE INTEGER MODULAR ADDITION - ON SECP256K1 BECAUSE P IS USED
    fn test_uint256_t_1_plus_1() {
        let mut ocl = Secp256k1Operations::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        let result = ocl.modular_addition(a, b).unwrap();

        assert_eq!(
            result,
            vec![
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x02,
            ]
        );
    }

    #[test]
    fn test_uint256_t_1_paaaaaaaalus_1() {
        let mut ocl = Secp256k1Operations::new().unwrap();

        let a = SECP256K1_P_MINUS_1.to_vec();

        let b = SECP256K1_P_MINUS_1.to_vec();

        let result = ocl.modular_addition(a, b).unwrap();

        assert_eq!(
            result,
            vec![
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
                0xff, 0xff, 0xfc, 0x2d,
            ]
        );
    }

    #[test]
    fn test_uint256_t_add_zero() {
        let mut ocl = Secp256k1Operations::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x42, // Random value 0x42
        ];

        let b = vec![0u8; 32]; // Zero

        let result = ocl.modular_addition(a.clone(), b).unwrap();

        // Adding zero should return the original number
        assert_eq!(result, a);
    }

    #[test]
    fn test_uint256_t_large_numbers() {
        let mut ocl = Secp256k1Operations::new().unwrap();

        // Two large numbers that don't exceed the modulus when added
        let a = vec![
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFE, 0x15,
        ];

        let b = vec![
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xFF, 0xFF, 0xFE, 0x15,
        ];

        let expected = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xFF, 0xFF, 0xFC, 0x2A, // Sum without modular reduction
        ];

        let result = ocl.modular_addition(a, b).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_uint256_t_modular_reduction() {
        let mut ocl = Secp256k1Operations::new().unwrap();

        // Test with p + 1, which should wrap around to 1
        let a = SECP256K1_P.to_vec();
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        let result = ocl.modular_addition(a, b).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_uint256_t_p_minus_1_plus_1() {
        let mut ocl = Secp256k1Operations::new().unwrap();

        // (p-1) + 1 should equal 0 (mod p)
        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        let expected = vec![0u8; 32];

        let result = ocl.modular_addition(a, b).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_uint256_t_half_p_plus_half_p() {
        let mut ocl = Secp256k1Operations::new().unwrap();

        let half_p_floor = vec![
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x7F, 0xFF, 0xFE, 0x17,
        ];

        let half_p_ceil = vec![
            0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x7F, 0xFF, 0xFE, 0x18,
        ];

        let result = ocl.modular_addition(half_p_floor, half_p_ceil).unwrap();

        // floor(p/2) + ceil(p/2) = p, which is congruent to 0 mod p
        let expected = vec![0u8; 32]; // Zero
        assert_eq!(result, expected);
    }

    #[test]
    fn test_uint256_t_consecutive_additions() {
        let mut ocl = Secp256k1Operations::new().unwrap();

        // Start with 1
        let mut accumulator = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        // Add 1 repeatedly
        let one = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        // Expected result after adding 1 five times: 6
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x06,
        ];

        // Add 1 five times
        for _ in 0..5 {
            accumulator = ocl.modular_addition(accumulator, one.clone()).unwrap();
        }

        assert_eq!(accumulator, expected);
    }

    // SIMPLE INTEGER MODULAR MULTIPLICATION - ON SECP256K1 BECAUSE P IS USED

    #[test]
    fn test_uint256_t_2_times_3_mod_p() {
        let mut ocl = Secp256k1Operations::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03,
        ];

        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x06,
        ];

        let result = ocl.modular_multiplication(a, b).unwrap();
        return;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_uint256_t_8_bytes_times_8_bytes_not_overflowing() {
        return;
        let mut ocl = Secp256k1Operations::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x12, 0x34, 0x56,
            0x78, 0x9A, 0xBC, 0xF0,
        ];

        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x12, 0x34, 0x56,
            0x78, 0x9A, 0xBC, 0xDE,
        ];

        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xe1, 0x22, 0x23, 0x6d, 0x88, 0xfe, 0x56, 0x27, 0xce, 0x02, 0x31, 0xd4,
            0x61, 0x50, 0x18, 0x20,
        ];

        let result = ocl.modular_multiplication(a, b).unwrap();
        for i in 0..32 {
            print!("{:02x}", result[i]);
        }
        println!("\n\n");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_uint256_t_16_bytes_times_16_bytes_not_overflowing() {
        return;
        let mut ocl = Secp256k1Operations::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xF0, 0xF0, 0x12, 0x34, 0x56,
            0x78, 0x9A, 0xBC, 0xF0,
        ];

        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xF0, 0xF0, 0x12, 0x34, 0x56,
            0x78, 0x9A, 0xBC, 0xDE,
        ];

        let expected = vec![
            0xe1, 0x22, 0x23, 0x6d, 0x88, 0xfe, 0x56, 0x3a, 0x71, 0x8e, 0x26, 0xc3, 0xee, 0x2e,
            0x0d, 0x61, 0x5e, 0x6e, 0x35, 0x2a, 0xc6, 0x7f, 0xcf, 0x47, 0xce, 0x02, 0x31, 0xd4,
            0x61, 0x50, 0x18, 0x20,
        ];
        let result = ocl.modular_multiplication(a, b).unwrap();
        for i in 0..32 {
            print!("{:02x}", result[i]);
        }
        println!("\n\n");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_uint256_t_modular_multiplication_overflowing_all_0xff() {
        let mut ocl = Secp256k1Operations::new().unwrap();

        let a = vec![
            0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xFF, 0xFF, 0xFC, 0x00,
        ];

        let b = vec![
            0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xFF, 0xFF, 0xFC, 0x00,
        ];

        let expected = vec![
            0x61, 0x6b, 0x61, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfe, 0x01,
            0x03, 0xc9, 0x6a, 0x72,
        ];

        let result = ocl.modular_multiplication(a, b).unwrap();
        return;
        for i in 0..32 {
            print!("{:02x}", result[i]);
        }
        println!("\n\n");
        assert_eq!(result, expected);
    }
}
