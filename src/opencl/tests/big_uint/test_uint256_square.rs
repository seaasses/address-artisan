#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct Uint256Square {
        a_buffer: Buffer<u8>,
        result_buffer: Buffer<u8>,
        uint256_square_kernel: Kernel,
    }

    impl Uint256Square {
        pub fn new() -> Result<Self, String> {
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            let a_buffer = Self::new_buffer(&queue, 32)?;
            let result_buffer = Self::new_buffer(&queue, 64)?;

            let program = Self::build_program(device, context)?;

            let uint256_square_kernel = match Kernel::builder()
                .program(&program)
                .name("uint256_square_kernel")
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
                uint256_square_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/uint256_square_kernel"));

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
                match self.uint256_square_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let mut data = vec![0u8; 64];
            match self.result_buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data)
        }
    }

    #[test]
    fn test_uint256_square_zero() {
        let mut ocl = Uint256Square::new().unwrap();
        assert_eq!(ocl.square(vec![0u8; 32]).unwrap(), vec![0u8; 64]);
    }

    #[test]
    fn test_uint256_square_three() {
        let mut ocl = Uint256Square::new().unwrap();
        let mut a = vec![0u8; 32];
        a[31] = 0x03;
        let mut expected = vec![0u8; 64];
        expected[63] = 0x09;
        assert_eq!(ocl.square(a).unwrap(), expected);
    }

    #[test]
    fn test_uint256_square_max_u64_limb() {
        let mut ocl = Uint256Square::new().unwrap();
        // a = 2^64 - 1: exercises the diagonal product and the carry into
        // the next limb: (2^64-1)^2 = 0xFFFFFFFFFFFFFFFE_0000000000000001
        let mut a = vec![0u8; 32];
        for i in 24..32 {
            a[i] = 0xff;
        }
        let mut expected = vec![0u8; 64];
        for i in 48..55 {
            expected[i] = 0xff;
        }
        expected[55] = 0xfe;
        expected[63] = 0x01;
        assert_eq!(ocl.square(a).unwrap(), expected);
    }

    #[test]
    fn test_uint256_square_all_ones() {
        let mut ocl = Uint256Square::new().unwrap();
        // a = 2^256 - 1: full carry-chain stress.
        // (2^256-1)^2 = 2^512 - 2^257 + 1
        let a = vec![0xffu8; 32];
        let mut expected = vec![0u8; 64];
        for i in 0..31 {
            expected[i] = 0xff;
        }
        expected[31] = 0xfe;
        expected[63] = 0x01;
        assert_eq!(ocl.square(a).unwrap(), expected);
    }

    #[test]
    fn test_uint256_square_p_minus_1() {
        let mut ocl = Uint256Square::new().unwrap();
        let a = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0xff, 0xff, 0xfc, 0x2e,
        ];
        let expected = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfd,
            0xff, 0xff, 0xf8, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x07, 0xa4, 0x00, 0x0e, 0x98, 0x44,
        ];
        assert_eq!(expected.len(), 64);
        assert_eq!(ocl.square(a).unwrap(), expected);
    }

    #[test]
    fn test_uint256_square_random_big() {
        let mut ocl = Uint256Square::new().unwrap();
        let a = vec![
            0xd3, 0x58, 0x14, 0x94, 0xb0, 0xf9, 0x22, 0xf3, 0x39, 0x3a, 0x25, 0xc9, 0x1a, 0xd6,
            0xa4, 0x90, 0x57, 0x6b, 0x61, 0x1e, 0xde, 0x5b, 0x2a, 0xbc, 0x86, 0x2c, 0xa0, 0x4e,
            0x3b, 0x09, 0x4e, 0x23,
        ];
        let expected = vec![
            0xae, 0x7a, 0x50, 0x3b, 0x43, 0x9b, 0xec, 0xe8, 0xff, 0x6b, 0x1e, 0xf2, 0x2e, 0x3e,
            0x2c, 0xea, 0xce, 0x91, 0x61, 0x3e, 0xc5, 0xf0, 0x39, 0x83, 0x66, 0x17, 0x08, 0xe2,
            0xe9, 0xf7, 0x17, 0x62, 0xa4, 0x3e, 0x14, 0x69, 0xc2, 0x34, 0x22, 0x06, 0x3b, 0x1d,
            0x9a, 0xc0, 0xe8, 0x6e, 0x77, 0xf3, 0xa1, 0x20, 0xd0, 0x35, 0x69, 0x44, 0x2a, 0x1b,
            0x28, 0xfc, 0xa7, 0xae, 0xb8, 0x4f, 0x58, 0xc9,
        ];
        assert_eq!(ocl.square(a).unwrap(), expected);
    }
}
