#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    // SECP256K1_P_MINUS_1 = 0xFFFFFFFEFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFC2E
    const SECP256K1_P_MINUS_1: [u8; 32] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF,
        0xFC, 0x2E,
    ];

    pub struct ModularAddition {
        a_buffer: Buffer<u8>,
        b_buffer: Buffer<u8>,
        result_buffer: Buffer<u8>,
        modular_addition_kernel: Kernel,
    }

    impl ModularAddition {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let a_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let b_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let result_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes

            let program = Self::build_program(device, context)?;

            // Create kernel
            let modular_addition_kernel = match Kernel::builder()
                .program(&program)
                .name("modular_addition_kernel")
                .queue(queue.clone())
                .arg(&a_buffer)
                .arg(&b_buffer)
                .arg(&result_buffer)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                a_buffer,
                b_buffer,
                result_buffer,
                modular_addition_kernel,
            })
        }

        fn new_buffer(queue: &Queue, len: usize) -> Result<Buffer<u8>, String> {
            let buffer = match Buffer::<u8>::builder()
                .queue(queue.clone())
                .len(len)
                .build()
            {
                Ok(buffer) => buffer,
                Err(e) => return Err("Error creating buffer: ".to_string() + &e.to_string()),
            };
            Ok(buffer)
        }

        fn build_program(device: Device, context: Context) -> Result<Program, String> {
            let src = include_str!(concat!(env!("OUT_DIR"), "/modular_addition_kernel"));

            let program = match Program::builder().src(src).devices(device).build(&context) {
                Ok(program) => program,
                Err(e) => {
                    return Err("Error building OpenCL program: ".to_string() + &e.to_string())
                }
            };

            Ok(program)
        }

        fn get_device_context_and_queue() -> Result<(Device, Context, Queue), String> {
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

            Ok((device, context, queue))
        }

        fn write_to_buffer(
            self: &mut Self,
            buffer: &Buffer<u8>,
            data: Vec<u8>,
        ) -> Result<(), String> {
            match buffer.write(&data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing to buffer: ".to_string() + &e.to_string()),
            };
            Ok(())
        }

        fn read_from_buffer(self: &mut Self, buffer: &Buffer<u8>) -> Result<Vec<u8>, String> {
            let mut data = vec![0u8; 32]; // Uint256 = 32 bytes
            match buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data)
        }

        fn addition(&mut self, a: Vec<u8>, b: Vec<u8>) -> Result<Vec<u8>, String> {
            if a.len() != 32 {
                return Err(format!("Input 'a' must be 32 bytes long, got: {}", a.len()));
            }
            if b.len() != 32 {
                return Err(format!("Input 'b' must be 32 bytes long, got: {}", b.len()));
            }

            // Clone the buffer to avoid borrowing issues
            self.write_to_buffer(&self.a_buffer.clone(), a)?;
            self.write_to_buffer(&self.b_buffer.clone(), b)?;

            // Execute kernel
            unsafe {
                match self.modular_addition_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let result_array = self.read_from_buffer(&self.result_buffer.clone())?;

            Ok(result_array)
        }
    }

    #[test]
    fn test_modular_addition_simple() {
        let mut ocl = ModularAddition::new().unwrap();

        // Test: 5 + 3 = 8 (mod secp256k1_p)
        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x05,
        ];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x08,
        ];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_addition_0_0() {
        let mut ocl = ModularAddition::new().unwrap();

        // Test: 0 + 0 = 0 (mod secp256k1_p)
        let a = vec![0x00; 32];
        let b = vec![0x00; 32];
        let expected = vec![0x00; 32];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_addition_1_1() {
        let mut ocl = ModularAddition::new().unwrap();

        // Test: 1 + 1 = 2 (mod secp256k1_p)
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
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_addition_2_3() {
        let mut ocl = ModularAddition::new().unwrap();

        // Test: 2 + 3 = 5 (mod secp256k1_p)
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
            0x00, 0x00, 0x00, 0x05,
        ];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_addition_p_minus_1_plus_1() {
        let mut ocl = ModularAddition::new().unwrap();

        // Test: (p-1) + 1 = 0 (mod secp256k1_p)
        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let expected = vec![0x00; 32];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_addition_p_minus_1_plus_2() {
        let mut ocl = ModularAddition::new().unwrap();

        // Test: (p-1) + 2 = 1 (mod secp256k1_p)
        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_addition_big_numbers_that_pass_p_but_dont_overflow_256_bits() {
        let mut ocl = ModularAddition::new().unwrap();

        // Test: Big numbers that result > p but don't overflow 256 bits
        let a = vec![
            0xf3, 0x48, 0xf7, 0xdf, 0x5f, 0xdb, 0xa0, 0xde, 0x80, 0x4d, 0xea, 0x13, 0x43, 0xdc,
            0x2f, 0x6d, 0xb7, 0x65, 0xcf, 0x9e, 0x55, 0x5c, 0x77, 0xa2, 0x08, 0x3b, 0x8a, 0x21,
            0x07, 0xe6, 0x73, 0xf7,
        ];
        let b = vec![
            0x0c, 0xb7, 0x08, 0x20, 0xa0, 0x24, 0x5f, 0x21, 0x7f, 0xb2, 0x15, 0xec, 0xbc, 0x23,
            0xd0, 0x92, 0x48, 0x9a, 0x30, 0x61, 0xaa, 0xa3, 0x88, 0x5d, 0xf7, 0xc4, 0x75, 0xde,
            0xeb, 0x64, 0x1c, 0xf2,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xf3, 0x4a, 0x94, 0xba,
        ];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_addition_big_numbers_that_overflow_256_bits() {
        let mut ocl = ModularAddition::new().unwrap();

        // Test: Big numbers that overflow 256 bits
        let a = vec![
            0xa5, 0x1d, 0x7e, 0xc7, 0x71, 0x44, 0x98, 0x3b, 0xce, 0xba, 0x07, 0x9e, 0x66, 0xc3,
            0x8d, 0xd6, 0x7b, 0x33, 0xc1, 0x57, 0x50, 0x5d, 0x33, 0xbe, 0x48, 0xa6, 0x57, 0xd7,
            0x21, 0x1f, 0x6c, 0x13,
        ];

        let b = vec![
            0xc4, 0x44, 0x48, 0x69, 0x2f, 0x9f, 0x3a, 0xd8, 0xd3, 0x25, 0x92, 0xee, 0xba, 0xb7,
            0x98, 0x9e, 0xb9, 0xe1, 0x55, 0x6e, 0xb5, 0xb2, 0x9b, 0xdf, 0xa1, 0x52, 0x17, 0x3e,
            0x01, 0x3f, 0xa4, 0xc2,
        ];
        let expected = vec![
            0x69, 0x61, 0xc7, 0x30, 0xa0, 0xe3, 0xd3, 0x14, 0xa1, 0xdf, 0x9a, 0x8d, 0x21, 0x7b,
            0x26, 0x75, 0x35, 0x15, 0x16, 0xc6, 0x06, 0x0f, 0xcf, 0x9d, 0xe9, 0xf8, 0x6f, 0x16,
            0x22, 0x5f, 0x14, 0xa6,
        ];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }
}
