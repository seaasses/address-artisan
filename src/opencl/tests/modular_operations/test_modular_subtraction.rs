#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct ModularSubtraction {
        a_buffer: Buffer<u8>,
        b_buffer: Buffer<u8>,
        result_buffer: Buffer<u8>,
        modular_subtraction_kernel: Kernel,
    }

    impl ModularSubtraction {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let a_buffer = Self::new_buffer(&queue, 32)?;
            let b_buffer = Self::new_buffer(&queue, 32)?;

            let result_buffer = Self::new_buffer(&queue, 32)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let modular_subtraction_kernel = match Kernel::builder()
                .program(&program)
                .name("modularSubtractionKernel")
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
                modular_subtraction_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/modularSubtractionKernel"));

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

        fn modular_subtraction(&mut self, a: Vec<u8>, b: Vec<u8>) -> Result<Vec<u8>, String> {
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
                match self.modular_subtraction_kernel.enq() {
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
    fn test_modular_subtraction_1_1() {
        let mut ocl = ModularSubtraction::new().unwrap();

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
        let expected = vec![0x00; 32];

        assert_eq!(ocl.modular_subtraction(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_subtraction_p_minus_1_minus_p_minus_1() {
        let mut ocl = ModularSubtraction::new().unwrap();

        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = SECP256K1_P_MINUS_1.to_vec();
        let expected = vec![0x00; 32];

        assert_eq!(ocl.modular_subtraction(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_subtraction_two_big_numbers_that_not_underflow() {
        let mut ocl = ModularSubtraction::new().unwrap();

        let a = vec![
            0x37, 0x38, 0x7c, 0xbd, 0xcd, 0x21, 0x1b, 0x05, 0xfd, 0x9a, 0xa3, 0xba, 0x03, 0x4d,
            0x82, 0xa9, 0x33, 0xbb, 0x4e, 0xc1, 0x09, 0xa9, 0xc3, 0x41, 0x07, 0xc8, 0xcf, 0x87,
            0x43, 0x5c, 0x3b, 0x59,
        ];
        let b = vec![
            0x17, 0xda, 0xcc, 0x23, 0x7e, 0x0b, 0x7c, 0xac, 0x0f, 0x5d, 0xf0, 0x4f, 0x06, 0xf4,
            0x33, 0xad, 0xdd, 0x36, 0xd7, 0xa3, 0xb2, 0xa5, 0x08, 0x79, 0xff, 0xc0, 0x7d, 0x24,
            0x6a, 0x7c, 0x74, 0xb0,
        ];
        let expected = vec![
            0x1f, 0x5d, 0xb0, 0x9a, 0x4f, 0x15, 0x9e, 0x59, 0xee, 0x3c, 0xb3, 0x6a, 0xfc, 0x59,
            0x4e, 0xfb, 0x56, 0x84, 0x77, 0x1d, 0x57, 0x04, 0xba, 0xc7, 0x08, 0x08, 0x52, 0x62,
            0xd8, 0xdf, 0xc6, 0xa9,
        ];

        assert_eq!(ocl.modular_subtraction(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_subtraction_two_big_numbers_that_underflow_and_before_modulus_is_more_than_p() {
        let mut ocl = ModularSubtraction::new().unwrap();

        let a = vec![
            0x4b, 0xf1, 0x0b, 0xd5, 0x57, 0xc9, 0xe5, 0x77, 0xaa, 0xb3, 0x0d, 0x47, 0xee, 0xe0,
            0xef, 0x93, 0xed, 0xe4, 0xb1, 0x9a, 0x69, 0x8c, 0x09, 0xd8, 0xd5, 0xf3, 0x77, 0x0a,
            0x8e, 0xbf, 0x6d, 0xf5,
        ];
        let b = vec![
            0x4b, 0xf1, 0x0b, 0xd5, 0x57, 0xc9, 0xe5, 0x77, 0xaa, 0xb3, 0x0d, 0x47, 0xee, 0xe0,
            0xef, 0x93, 0xed, 0xe4, 0xb1, 0x9a, 0x69, 0x8c, 0x09, 0xd8, 0xd5, 0xf3, 0x77, 0x0a,
            0xab, 0xe1, 0x8e, 0xd3,
        ];
        let expected = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0xe2, 0xdd, 0xdb, 0x51,
        ];

        assert_eq!(ocl.modular_subtraction(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_subtraction_two_big_numbers_that_underflow_and_before_modulus_is_less_than_p() {
        let mut ocl = ModularSubtraction::new().unwrap();

        let a = vec![
            0x6b, 0xea, 0xd6, 0x97, 0xfb, 0x9c, 0xe5, 0xd5, 0x4f, 0x44, 0xa0, 0xe1, 0x33, 0x39,
            0x42, 0x0f, 0x1f, 0xcf, 0x90, 0x85, 0xd7, 0x5c, 0x92, 0x5b, 0x1f, 0xc9, 0x1a, 0xa5,
            0x3e, 0x0d, 0xc6, 0x11,
        ];
        let b = vec![
            0x9f, 0x53, 0x75, 0xd0, 0x56, 0x46, 0xba, 0x4b, 0xc5, 0xd6, 0x66, 0x62, 0x75, 0x1d,
            0x12, 0xf0, 0xea, 0xbd, 0x0c, 0x4a, 0xf7, 0x49, 0x1b, 0xba, 0x3d, 0xb1, 0xba, 0x97,
            0xb9, 0xfa, 0x8c, 0xde,
        ];
        let expected = vec![
            0xcc, 0x97, 0x60, 0xc7, 0xa5, 0x56, 0x2b, 0x89, 0x89, 0x6e, 0x3a, 0x7e, 0xbe, 0x1c,
            0x2f, 0x1e, 0x35, 0x12, 0x84, 0x3a, 0xe0, 0x13, 0x76, 0xa0, 0xe2, 0x17, 0x60, 0x0c,
            0x84, 0x13, 0x35, 0x62,
        ];

        assert_eq!(ocl.modular_subtraction(a, b).unwrap(), expected);
    }
}
