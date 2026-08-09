#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct ModularSquare {
        a_buffer: Buffer<u8>,
        result_buffer: Buffer<u8>,
        modular_square_kernel: Kernel,
    }

    impl ModularSquare {
        pub fn new() -> Result<Self, String> {
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            let a_buffer = Self::new_buffer(&queue, 32)?;
            let result_buffer = Self::new_buffer(&queue, 32)?;

            let program = Self::build_program(device, context)?;

            let modular_square_kernel = match Kernel::builder()
                .program(&program)
                .name("modular_square_kernel")
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
                modular_square_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/modular_square_kernel"));

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
                .devices(device)
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

        fn square(&mut self, a: Vec<u8>) -> Result<Vec<u8>, String> {
            if a.len() != 32 {
                return Err(format!("Input 'a' must be 32 bytes long, got: {}", a.len()));
            }

            match self.a_buffer.write(&a[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing to buffer: ".to_string() + &e.to_string()),
            };

            unsafe {
                match self.modular_square_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let mut data = vec![0u8; 32];
            match self.result_buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data)
        }
    }

    #[test]
    fn test_modular_square_three() {
        let mut ocl = ModularSquare::new().unwrap();
        let mut a = vec![0u8; 32];
        a[31] = 0x03;
        let mut expected = vec![0u8; 32];
        expected[31] = 0x09;
        assert_eq!(ocl.square(a).unwrap(), expected);
    }

    #[test]
    fn test_modular_square_p_minus_1() {
        let mut ocl = ModularSquare::new().unwrap();
        // (p-1)^2 mod p = 1
        let a = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0xff, 0xff, 0xfc, 0x2e,
        ];
        let mut expected = vec![0u8; 32];
        expected[31] = 0x01;
        assert_eq!(ocl.square(a).unwrap(), expected);
    }

    #[test]
    fn test_modular_square_random_a() {
        let mut ocl = ModularSquare::new().unwrap();
        let a = vec![
            0x2e, 0x91, 0xa4, 0xf9, 0x33, 0xe5, 0x54, 0x1b, 0xfb, 0x13, 0xb2, 0x82, 0xb7, 0x44,
            0x67, 0x66, 0xdd, 0xed, 0x2e, 0xdd, 0x82, 0x5d, 0x3a, 0x88, 0xce, 0x88, 0x2f, 0x31,
            0x93, 0xa2, 0xcf, 0x1a,
        ];
        let expected = vec![
            0x1b, 0x0d, 0x61, 0x7d, 0x07, 0x6f, 0x63, 0x91, 0x1f, 0x3f, 0x60, 0xca, 0x98, 0x0f,
            0xbe, 0x2c, 0x48, 0x3f, 0xf2, 0x4b, 0xd0, 0x6b, 0xc4, 0x2b, 0x73, 0xff, 0x53, 0x7d,
            0xa3, 0x71, 0x6a, 0x04,
        ];
        assert_eq!(ocl.square(a).unwrap(), expected);
    }

    #[test]
    fn test_modular_square_random_b() {
        let mut ocl = ModularSquare::new().unwrap();
        let a = vec![
            0x76, 0xba, 0x21, 0xd8, 0x24, 0x55, 0xfe, 0x6b, 0x7b, 0x64, 0xec, 0xe6, 0x41, 0x5b,
            0xcd, 0x77, 0xd4, 0xda, 0xc0, 0x60, 0x1a, 0xc6, 0xc3, 0x15, 0x6a, 0xfa, 0xb7, 0x48,
            0x5c, 0xc9, 0xe8, 0x3a,
        ];
        let expected = vec![
            0x39, 0x9e, 0x63, 0xe1, 0x54, 0xfe, 0xa4, 0xd5, 0x62, 0xc9, 0xac, 0x3f, 0x05, 0x67,
            0x07, 0xf5, 0x43, 0xab, 0xe7, 0xf2, 0x70, 0x1c, 0x1a, 0x42, 0xb2, 0x88, 0x11, 0xe7,
            0x09, 0x16, 0x10, 0x5e,
        ];
        assert_eq!(ocl.square(a).unwrap(), expected);
    }
}
