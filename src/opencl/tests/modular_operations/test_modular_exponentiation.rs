#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct ModularExponentiation {
        a_buffer: Buffer<u8>,
        b_buffer: Buffer<u8>,
        result_buffer: Buffer<u8>,
        modular_exponentiation_kernel: Kernel,
    }

    impl ModularExponentiation {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let a_buffer = Self::new_buffer(&queue, 32)?;
            let b_buffer = Self::new_buffer(&queue, 32)?;

            let result_buffer = Self::new_buffer(&queue, 32)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let modular_exponentiation_kernel = match Kernel::builder()
                .program(&program)
                .name("modularExponentiationKernel")
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
                modular_exponentiation_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/modularExponentiationKernel"));

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
            let mut data = vec![0u8; 32];
            match buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data)
        }

        fn modular_exponentiation(&mut self, a: Vec<u8>, b: Vec<u8>) -> Result<Vec<u8>, String> {
            if a.len() != 32 || b.len() != 32 {
                return Err(format!(
                    "Input vectors must be 32 bytes long, got a: {}, b: {}",
                    a.len(),
                    b.len()
                ));
            }

            // Clone the buffer to avoid borrowing issues
            self.write_to_buffer(&self.a_buffer.clone(), a)?;
            self.write_to_buffer(&self.b_buffer.clone(), b)?;

            // Execute kernel
            unsafe {
                match self.modular_exponentiation_kernel.enq() {
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

    // This operation is a + b mod p, a and b are 32 bytes long and less than p - this is important
    #[test]
    fn test_modular_exponentiation_3_4() {
        let mut ocl = ModularExponentiation::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03,
        ];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x04,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x51,
        ];

        assert_eq!(ocl.modular_exponentiation(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_exponentiation_p_minus_1_power_0() {
        let mut ocl = ModularExponentiation::new().unwrap();

        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = vec![0x00; 32];

        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        assert_eq!(ocl.modular_exponentiation(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_exponentiation_p_minus_1_power_1() {
        let mut ocl = ModularExponentiation::new().unwrap();

        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let expected = SECP256K1_P_MINUS_1.to_vec();

        assert_eq!(ocl.modular_exponentiation(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_exponentiation_p_minus_1_2_power_2() {
        let mut ocl = ModularExponentiation::new().unwrap();

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

        assert_eq!(ocl.modular_exponentiation(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_exponentiation_p_minus_1_2_power_3() {
        let mut ocl = ModularExponentiation::new().unwrap();

        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03,
        ];
        let expected = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0xff, 0xff, 0xfc, 0x2e,
        ];

        assert_eq!(ocl.modular_exponentiation(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_exponentiation_p_minus_1_2_power_p_minus_1() {
        let mut ocl = ModularExponentiation::new().unwrap();

        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = SECP256K1_P_MINUS_1.to_vec();
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        assert_eq!(ocl.modular_exponentiation(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_exponentiation_two_big_numbers() {
        let mut ocl = ModularExponentiation::new().unwrap();
        let a = vec![
            0x56, 0x19, 0x93, 0x47, 0x27, 0x06, 0x9f, 0x9a, 0x1f, 0x15, 0xb3, 0x4b, 0xe4, 0xf1,
            0x0f, 0xc7, 0x90, 0xb8, 0x9c, 0x63, 0xb3, 0xb0, 0xdb, 0x06, 0xf6, 0x94, 0x05, 0x2d,
            0x4a, 0xf7, 0x39, 0x40,
        ];

        let b = vec![
            0x28, 0x22, 0xd9, 0xb2, 0x18, 0x0c, 0xd9, 0x16, 0x84, 0x46, 0x0c, 0x60, 0xb6, 0x19,
            0x9b, 0xc0, 0x74, 0xe0, 0x1e, 0x7d, 0x49, 0x95, 0xc8, 0xca, 0x31, 0x72, 0xed, 0xd6,
            0x89, 0x2e, 0x55, 0x05,
        ];
        let expected = vec![
            0xfc, 0x1d, 0xbd, 0xb3, 0xa7, 0xf2, 0x47, 0x6b, 0x27, 0xc1, 0xff, 0x10, 0x70, 0xe6,
            0x5b, 0xa1, 0x9e, 0x41, 0xf3, 0xd4, 0xae, 0xdf, 0x22, 0x5e, 0x55, 0x2e, 0xbd, 0xf3,
            0x4d, 0x0e, 0x8b, 0x6d,
        ];

        assert_eq!(ocl.modular_exponentiation(a, b).unwrap(), expected);
    }
}
