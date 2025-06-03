#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct Uint320Uint256Addition {
        a_buffer: Buffer<u8>,
        b_buffer: Buffer<u8>,
        result_buffer: Buffer<u8>,
        uint256_addition_kernel: Kernel,
    }

    impl Uint320Uint256Addition {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let a_buffer = Self::new_buffer(&queue, 40)?;
            let b_buffer = Self::new_buffer(&queue, 32)?;

            let result_buffer = Self::new_buffer(&queue, 40)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let uint256_addition_kernel = match Kernel::builder()
                .program(&program)
                .name("uint320_uint256_addition_kernel")
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
                uint256_addition_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/uint320_uint256_addition_kernel"));

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
            let mut data = vec![0u8; buffer.len()];
            match buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data)
        }

        fn addition(&mut self, a: Vec<u8>, b: Vec<u8>) -> Result<Vec<u8>, String> {
            if a.len() != 40 || b.len() != 32 {
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
                match self.uint256_addition_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let result_array = self.read_from_buffer(&self.result_buffer.clone())?;

            Ok(result_array)
        }
    }

    #[test]
    fn test_uint320_uint256_addition_1_plus_1() {
        let mut ocl = Uint320Uint256Addition::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
        ];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }

    #[test]
    fn test_uint320_uint256_addition_1_plus_max_256_bits() {
        let mut ocl = Uint320Uint256Addition::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let b = vec![0xFF; 32];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }

    #[test]
    fn test_uint320_uint256_addition_max_320_bits_plus_1() {
        let mut ocl = Uint320Uint256Addition::new().unwrap();

        let a = vec![0xFF; 40];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let expected = vec![0x00; 40];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }

    #[test]
    fn test_uint320_uint256_addition_big_320_bits_plus_big_256_bits() {
        let mut ocl = Uint320Uint256Addition::new().unwrap();

        let a = vec![
            0x11, 0x6f, 0x4f, 0xb0, 0x9f, 0x8f, 0x89, 0x99, 0xe2, 0x19, 0xe8, 0x54, 0x1a, 0xc0,
            0x86, 0xb1, 0x09, 0xaa, 0xef, 0x79, 0x6c, 0xb6, 0x0f, 0x6b, 0xbb, 0xdd, 0xb2, 0x29,
            0xd7, 0x24, 0xb4, 0x09, 0x94, 0xba, 0x26, 0x6e, 0x9f, 0x1d, 0x06, 0x39,
        ];
        let b = vec![
            0xec, 0x06, 0xb5, 0x06, 0x05, 0x9a, 0x56, 0x50, 0x22, 0xb1, 0xf6, 0x90, 0x23, 0x28,
            0x54, 0xef, 0xd6, 0xc2, 0x12, 0x68, 0xf3, 0xdc, 0xbb, 0x22, 0x8d, 0x1f, 0x03, 0x33,
            0xf7, 0x8f, 0x19, 0x83,
        ];
        let expected = vec![
            0x11, 0x6f, 0x4f, 0xb0, 0x9f, 0x8f, 0x89, 0x9a, 0xce, 0x20, 0x9d, 0x5a, 0x20, 0x5a,
            0xdd, 0x01, 0x2c, 0x5c, 0xe6, 0x09, 0x8f, 0xde, 0x64, 0x5b, 0x92, 0x9f, 0xc4, 0x92,
            0xcb, 0x01, 0x6f, 0x2c, 0x21, 0xd9, 0x29, 0xa2, 0x96, 0xac, 0x1f, 0xbc,
        ];

        assert_eq!(ocl.addition(a, b).unwrap(), expected);
    }
}
