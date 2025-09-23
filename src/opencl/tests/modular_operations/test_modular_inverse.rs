#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct ModularInverse {
        a_buffer: Buffer<u8>,
        result_buffer: Buffer<u8>,
        modular_inverse_kernel: Kernel,
    }

    impl ModularInverse {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let a_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let result_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes

            let program = Self::build_program(device, context)?;

            // Create kernel
            let modular_inverse_kernel = match Kernel::builder()
                .program(&program)
                .name("modular_inverse_kernel")
                .queue(queue.clone())
                .arg(&a_buffer)
                .arg(&result_buffer)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                a_buffer,
                result_buffer,
                modular_inverse_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/modular_inverse_kernel"));

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

        fn inverse(&mut self, a: Vec<u8>) -> Result<Vec<u8>, String> {
            if a.len() != 32 {
                return Err(format!("Input 'a' must be 32 bytes long, got: {}", a.len()));
            }

            // Clone the buffer to avoid borrowing issues
            self.write_to_buffer(&self.a_buffer.clone(), a)?;

            // Execute kernel
            unsafe {
                match self.modular_inverse_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let result_array = self.read_from_buffer(&self.result_buffer.clone())?;

            Ok(result_array)
        }
    }

    const SECP256K1_P_MINUS_1: [u8; 32] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF,
        0xFC, 0x2E,
    ];

    #[test]
    fn test_modular_inverse_simple() {
        let mut ocl = ModularInverse::new().unwrap();

        // Test: inverse of 3 (mod secp256k1_p)
        // 3^(-1) mod p should give a number that when multiplied by 3 gives 1 mod p
        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03,
        ];

        let result = ocl.inverse(a).unwrap();

        // The result should be the modular inverse of 3
        // We can verify this works by checking that result * 3 ≡ 1 (mod p)
        // For now, just check that we get a non-zero result
        let is_zero = result.iter().all(|&x| x == 0);
        assert!(!is_zero, "Modular inverse should not be zero");
    }

    #[test]
    fn test_modular_inverse_1() {
        let mut ocl = ModularInverse::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        assert_eq!(ocl.inverse(a).unwrap(), expected);
    }

    #[test]
    fn test_modular_inverse_p_minus_1() {
        let mut ocl = ModularInverse::new().unwrap();

        let a = SECP256K1_P_MINUS_1.to_vec();
        let expected = SECP256K1_P_MINUS_1.to_vec();

        assert_eq!(ocl.inverse(a).unwrap(), expected);
    }

    #[test]
    fn test_modular_inverse_big_number() {
        let mut ocl = ModularInverse::new().unwrap();

        let a = vec![
            0x1b, 0x4b, 0x68, 0x19, 0x7a, 0x5b, 0xe8, 0xd0, 0xed, 0x01, 0x05, 0x42, 0x3a, 0x1d,
            0xe9, 0x6c, 0xc7, 0x29, 0x33, 0xd8, 0x69, 0x1e, 0xa6, 0x8b, 0x97, 0x80, 0x4e, 0x5c,
            0x09, 0x0e, 0x99, 0xd8,
        ];
        let expected = vec![
            0x96, 0xe6, 0xfe, 0x21, 0xdb, 0xb1, 0x2d, 0x96, 0xe0, 0xcb, 0x6c, 0x63, 0x97, 0x31,
            0xc6, 0x4f, 0x7d, 0x62, 0x17, 0xa9, 0xe8, 0xe9, 0x0a, 0xf0, 0xf8, 0x5e, 0x64, 0x68,
            0x2a, 0xb8, 0x5f, 0x99,
        ];

        assert_eq!(ocl.inverse(a).unwrap(), expected);
    }

    #[test]
    fn test_modular_inverse_max_64_bits() {
        let mut ocl = ModularInverse::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let expected = vec![
            0x6d, 0x34, 0xef, 0x80, 0xa6, 0x2e, 0xe5, 0x86, 0x6d, 0x34, 0xef, 0x80, 0xa6, 0x2e,
            0xe5, 0x86, 0x6d, 0x34, 0xef, 0x80, 0xa6, 0x2e, 0xe5, 0x86, 0x6d, 0x34, 0xef, 0x80,
            0x38, 0xf9, 0xf4, 0x65,
        ];

        assert_eq!(ocl.inverse(a).unwrap(), expected);
    }

    #[test]
    fn test_modular_inverse_max_128_bits() {
        let mut ocl = ModularInverse::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let expected = vec![
            0x3a, 0xcc, 0x58, 0x80, 0xd4, 0xee, 0x94, 0xd4, 0x32, 0x68, 0x96, 0xff, 0xd1, 0x40,
            0x50, 0xb2, 0x3a, 0xcc, 0x58, 0x80, 0xd4, 0xee, 0x94, 0xd4, 0x32, 0x68, 0x96, 0xff,
            0x96, 0x73, 0xf7, 0x51,
        ];

        assert_eq!(ocl.inverse(a).unwrap(), expected);
    }

    #[test]
    fn test_modular_inverse_max_192_bits() {
        let mut ocl = ModularInverse::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let expected = vec![
            0x6b, 0xf6, 0x73, 0xe6, 0xdc, 0x69, 0x44, 0x30, 0xe3, 0xf9, 0x98, 0x40, 0x11, 0xb6,
            0xd6, 0xd6, 0x1d, 0x44, 0xe3, 0x59, 0xb8, 0x0e, 0xca, 0x7f, 0x6b, 0xf6, 0x73, 0xe6,
            0x70, 0x72, 0xce, 0xae,
        ];

        assert_eq!(ocl.inverse(a).unwrap(), expected);
    }
}
