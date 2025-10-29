#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    const MESSAGE_SIZE: usize = 32;
    const HASH_SIZE: usize = 20;

    pub struct Ripemd160_32Bytes {
        message_buffer: Buffer<u8>,
        hash_buffer: Buffer<u8>,
        ripemd160_32_bytes_kernel: Kernel,
    }

    impl Ripemd160_32Bytes {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let message_buffer = Self::new_buffer(&queue, MESSAGE_SIZE)?;
            let hash_buffer = Self::new_buffer(&queue, HASH_SIZE)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let ripemd160_32_bytes_kernel = match Kernel::builder()
                .program(&program)
                .name("ripemd160_32_bytes_kernel")
                .queue(queue.clone())
                .arg(&message_buffer)
                .arg(&hash_buffer)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                message_buffer,
                hash_buffer,
                ripemd160_32_bytes_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/ripemd160_32_bytes_kernel"));

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
            let mut data = vec![0u8; HASH_SIZE];
            match buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data)
        }

        fn hash(&mut self, message: Vec<u8>) -> Result<Vec<u8>, String> {
            if message.len() != MESSAGE_SIZE {
                return Err(format!(
                    "Message must be exactly {} bytes, got {}",
                    MESSAGE_SIZE,
                    message.len()
                ));
            }

            // Clone the buffer to avoid borrowing issues
            self.write_to_buffer(&self.message_buffer.clone(), message)?;

            // Execute kernel
            unsafe {
                match self.ripemd160_32_bytes_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let hash = self.read_from_buffer(&self.hash_buffer.clone())?;

            Ok(hash)
        }
    }

    #[test]
    fn test_ripemd160_32_bytes_sha256_empty_string() {
        let mut ocl = Ripemd160_32Bytes::new().unwrap();

        let message = vec![
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
            0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
            0x78, 0x52, 0xb8, 0x55,
        ];

        assert_eq!(message.len(), MESSAGE_SIZE, "Message must be exactly 32 bytes");

        let expected = vec![
            0xb4, 0x72, 0xa2, 0x66, 0xd0, 0xbd, 0x89, 0xc1, 0x37, 0x06, 0xa4, 0x13, 0x2c, 0xcf,
            0xb1, 0x6f, 0x7c, 0x3b, 0x9f, 0xcb,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_ripemd160_32_bytes_sha256_genesis_pubkey() {
        let mut ocl = Ripemd160_32Bytes::new().unwrap();

        let message = vec![
            0x0f, 0x71, 0x5b, 0xaf, 0x5d, 0x4c, 0x2e, 0xd3, 0x29, 0x78, 0x5c, 0xef, 0x29, 0xe5,
            0x62, 0xf7, 0x34, 0x88, 0xc8, 0xa2, 0xbb, 0x9d, 0xbc, 0x57, 0x00, 0xb3, 0x61, 0xd5,
            0x4b, 0x9b, 0x05, 0x54,
        ];

        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected = vec![
            0x75, 0x1e, 0x76, 0xe8, 0x19, 0x91, 0x96, 0xd4, 0x54, 0x94, 0x1c, 0x45, 0xd1, 0xb3,
            0xa3, 0x23, 0xf1, 0x43, 0x3b, 0xd6,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_ripemd160_32_bytes_example_1() {
        let mut ocl = Ripemd160_32Bytes::new().unwrap();

        let message = vec![
            0xb1, 0xc9, 0x93, 0x8f, 0x01, 0x12, 0x1e, 0x15, 0x98, 0x87, 0xac, 0x2c, 0x8d, 0x39,
            0x3a, 0x22, 0xe4, 0x47, 0x6f, 0xf8, 0x21, 0x2d, 0xe1, 0x3f, 0xe1, 0x93, 0x9d, 0xe2,
            0xa2, 0x36, 0xf0, 0xa7,
        ];

        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected = vec![
            0x06, 0xaf, 0xd4, 0x6b, 0xcd, 0xfd, 0x22, 0xef, 0x94, 0xac, 0x12, 0x2a, 0xa1, 0x1f,
            0x24, 0x12, 0x44, 0xa3, 0x7e, 0xcc,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_ripemd160_32_bytes_satoshi_quote() {
        let mut ocl = Ripemd160_32Bytes::new().unwrap();

        // SHA256("I've been working on a new electronic cash system that's fully peer-to-peer, with no trusted third party.")
        let message = vec![
            0x3e, 0xb7, 0x21, 0xae, 0xb7, 0x67, 0x7a, 0x34, 0xa4, 0x8b, 0x24, 0x3b, 0xc5, 0x73,
            0x1a, 0x72, 0x17, 0xbf, 0x8c, 0xe2, 0xa0, 0xc0, 0xca, 0x17, 0x29, 0xf9, 0x17, 0x68,
            0xbd, 0x90, 0x74, 0x42,
        ];

        assert_eq!(message.len(), MESSAGE_SIZE);

        // RIPEMD160 of above
        let expected = vec![
            0xc3, 0xa5, 0xce, 0x36, 0x60, 0xcb, 0xca, 0x4b, 0xac, 0xf5, 0xa8, 0xa9, 0xb6, 0x32,
            0xb1, 0x97, 0x81, 0x8f, 0x8d, 0x5d,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_ripemd160_32_bytes_freedom_quote() {
        let mut ocl = Ripemd160_32Bytes::new().unwrap();

        // SHA256("Freedom is the mother, not the daughter, of order.")
        let message = vec![
            0x67, 0xcb, 0x23, 0xbe, 0x92, 0xeb, 0xd0, 0x57, 0x44, 0xb7, 0x92, 0x2c, 0xed, 0x66,
            0x11, 0xbd, 0x54, 0x62, 0x92, 0x93, 0xc0, 0xab, 0x29, 0xf3, 0x78, 0xec, 0x1b, 0x84,
            0xfe, 0x40, 0x9a, 0x04,
        ];

        assert_eq!(message.len(), MESSAGE_SIZE);

        // RIPEMD160 of above
        let expected = vec![
            0xf6, 0xda, 0xc9, 0xb5, 0x04, 0x55, 0xdb, 0x5a, 0xc5, 0x8a, 0x8b, 0x0e, 0xa3, 0xc6,
            0x0d, 0x9b, 0x5d, 0x68, 0xf0, 0x3c,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_ripemd160_32_bytes_sound_money() {
        let mut ocl = Ripemd160_32Bytes::new().unwrap();

        // SHA256("Sound money is a prerequisite for limited government.")
        let message = vec![
            0x61, 0xb0, 0xb7, 0xb5, 0x26, 0x42, 0x85, 0x48, 0xd7, 0x3e, 0x4e, 0x0d, 0xe2, 0xa0,
            0xce, 0xa5, 0xf4, 0xa3, 0x75, 0x6d, 0x4d, 0x0a, 0xef, 0x40, 0x75, 0xbe, 0x6f, 0x60,
            0xe9, 0xaa, 0xb7, 0x45,
        ];

        assert_eq!(message.len(), MESSAGE_SIZE);

        // RIPEMD160 of above
        let expected = vec![
            0xb2, 0x58, 0xc7, 0x8e, 0x03, 0x99, 0xda, 0x11, 0x43, 0x7a, 0xf6, 0x4a, 0xbe, 0x68,
            0xba, 0x3c, 0xc7, 0x94, 0x14, 0x6c,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }
}
