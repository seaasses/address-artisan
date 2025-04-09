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
            let a_buffer = Self::new_buffer(&queue, 32)?;
            let b_buffer = Self::new_buffer(&queue, 32)?;

            let result_buffer = Self::new_buffer(&queue, 32)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let modular_multiplication_kernel = match Kernel::builder()
                .program(&program)
                .name("modularMultiplicationKernel")
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/modularMultiplicationKernel"));

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

        fn modular_multiplication(&mut self, a: Vec<u8>, b: Vec<u8>) -> Result<Vec<u8>, String> {
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

    // This operation is a * b mod p, a and b are 32 bytes long and less than p - this is important
    #[test]
    fn test_modular_multiplication_3_2() {
        let mut ocl = ModularMultiplication::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03,
        ];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x06,
        ];

        assert_eq!(ocl.modular_multiplication(a, b).unwrap(), expected);
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
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0xff, 0xff, 0xfc, 0x2d,
        ];

        assert_eq!(ocl.modular_multiplication(a, b).unwrap(), expected);
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

        assert_eq!(ocl.modular_multiplication(a, b).unwrap(), expected);
    }

    #[test]
    fn test_modular_multiplication_p_minus_1_times_big_number() {
        let mut ocl = ModularMultiplication::new().unwrap();

        let a = SECP256K1_P_MINUS_1.to_vec();
        let b = vec![
            0x8c, 0xed, 0x87, 0x13, 0x6a, 0xc2, 0x33, 0x48, 0xe5, 0x1e, 0x6d, 0x02, 0xbe, 0x37,
            0x25, 0xd7, 0x1e, 0xd6, 0x50, 0xff, 0x4e, 0x01, 0xad, 0xf8, 0x47, 0x42, 0x44, 0x08,
            0xed, 0x8c, 0xb6, 0x98,
        ];

        let expected = vec![
            0x73, 0x12, 0x78, 0xec, 0x95, 0x3d, 0xcc, 0xb7, 0x1a, 0xe1, 0x92, 0xfd, 0x41, 0xc8,
            0xda, 0x28, 0xe1, 0x29, 0xaf, 0x00, 0xb1, 0xfe, 0x52, 0x07, 0xb8, 0xbd, 0xbb, 0xf6,
            0x12, 0x73, 0x45, 0x97,
        ];

        assert_eq!(ocl.modular_multiplication(a, b).unwrap(), expected);
    }
}
