// TODO: Add more tests for edge cases and larger numbers

#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct ModularMultiplication {
        a_buffer: Buffer<u8>,
        b_buffer: Buffer<u8>,
        result_buffer: Buffer<u8>,
        modular_multiplication_kernel: Kernel,
    }

    impl ModularMultiplication {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let a_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let b_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let result_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes

            let program = Self::build_program(device, context)?;

            // Create kernel
            let modular_multiplication_kernel = match Kernel::builder()
                .program(&program)
                .name("modular_multiplication_kernel")
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
                modular_multiplication_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/modular_multiplication_kernel"));

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

        fn multiplication(&mut self, a: Vec<u8>, b: Vec<u8>) -> Result<Vec<u8>, String> {
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
                match self.modular_multiplication_kernel.enq() {
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
    fn test_modular_multiplication_simple() {
        let mut ocl = ModularMultiplication::new().unwrap();

        // Test: 3 * 5 = 15 (mod secp256k1_p)
        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03,
        ];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x05,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x0f,
        ];

        assert_eq!(ocl.multiplication(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_multiplication_2_3() {
        let mut ocl = ModularMultiplication::new().unwrap();

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

        assert_eq!(ocl.multiplication(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_multiplication_p_minus_1_times_2() {
        let mut ocl = ModularMultiplication::new().unwrap();

        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];
        let expected = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xFF, 0xFF, 0xFC, 0x2D,
        ];

        assert_eq!(ocl.multiplication(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_multiplication_p_minus_1_times_p_minus_1() {
        let mut ocl = ModularMultiplication::new().unwrap();

        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = SECP256K1_P_MINUS_1.to_vec();
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        assert_eq!(ocl.multiplication(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_multiplication_p_minus_1_times_big_number() {
        let mut ocl = ModularMultiplication::new().unwrap();

        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = vec![
            0x2e, 0x91, 0xa4, 0xf9, 0x33, 0xe5, 0x54, 0x1b, 0xfb, 0x13, 0xb2, 0x82, 0xb7, 0x44,
            0x67, 0x66, 0xdd, 0xed, 0x2e, 0xdd, 0x82, 0x5d, 0x3a, 0x88, 0xce, 0x88, 0x2f, 0x31,
            0x93, 0xa2, 0xcf, 0x1a,
        ];
        let expected = vec![
            0xd1, 0x6e, 0x5b, 0x06, 0xcc, 0x1a, 0xab, 0xe4, 0x04, 0xec, 0x4d, 0x7d, 0x48, 0xbb,
            0x98, 0x99, 0x22, 0x12, 0xd1, 0x22, 0x7d, 0xa2, 0xc5, 0x77, 0x31, 0x77, 0xd0, 0xcd,
            0x6c, 0x5d, 0x2d, 0x15,
        ];

        assert_eq!(ocl.multiplication(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_multiplication_two_big_numbers() {
        let mut ocl = ModularMultiplication::new().unwrap();

        let a = vec![
            0xd3, 0x58, 0x14, 0x94, 0xb0, 0xf9, 0x22, 0xf3, 0x39, 0x3a, 0x25, 0xc9, 0x1a, 0xd6,
            0xa4, 0x90, 0x57, 0x6b, 0x61, 0x1e, 0xde, 0x5b, 0x2a, 0xbc, 0x86, 0x2c, 0xa0, 0x4e,
            0x3b, 0x09, 0x4e, 0x23,
        ];
        let b = vec![
            0x76, 0xba, 0x21, 0xd8, 0x24, 0x55, 0xfe, 0x6b, 0x7b, 0x64, 0xec, 0xe6, 0x41, 0x5b,
            0xcd, 0x77, 0xd4, 0xda, 0xc0, 0x60, 0x1a, 0xc6, 0xc3, 0x15, 0x6a, 0xfa, 0xb7, 0x48,
            0x5c, 0xc9, 0xe8, 0x3a,
        ];
        let expected = vec![
            0x4a, 0xfb, 0x73, 0x1c, 0xa8, 0x7e, 0x80, 0x5c, 0xc6, 0x92, 0x65, 0xad, 0x26, 0xab,
            0xed, 0x20, 0x17, 0x1f, 0xbb, 0xcc, 0xc0, 0x22, 0xd7, 0x92, 0x17, 0xe7, 0x13, 0x08,
            0xdb, 0x57, 0x16, 0xfc,
        ];

        assert_eq!(ocl.multiplication(a, b).unwrap(), expected);
    }
}
