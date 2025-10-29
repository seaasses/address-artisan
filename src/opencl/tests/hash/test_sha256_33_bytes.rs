#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    const MESSAGE_SIZE: usize = 33;
    const HASH_SIZE: usize = 32;

    pub struct Sha256_33Bytes {
        message_buffer: Buffer<u8>,
        hash_buffer: Buffer<u8>,
        sha256_33_bytes_kernel: Kernel,
    }

    impl Sha256_33Bytes {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let message_buffer = Self::new_buffer(&queue, MESSAGE_SIZE)?;
            let hash_buffer = Self::new_buffer(&queue, HASH_SIZE)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let sha256_33_bytes_kernel = match Kernel::builder()
                .program(&program)
                .name("sha256_33_bytes_kernel")
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
                sha256_33_bytes_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/sha256_33_bytes_kernel"));

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
                match self.sha256_33_bytes_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let hash = self.read_from_buffer(&self.hash_buffer.clone())?;

            Ok(hash)
        }
    }

    #[test]
    fn test_sha256_33_bytes_bitcoin_genesis_pubkey() {
        let mut ocl = Sha256_33Bytes::new().unwrap();

        // Compressed public key from Bitcoin genesis block
        let message = vec![
            0x02, 0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce,
            0x87, 0x0b, 0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81,
            0x5b, 0x16, 0xf8, 0x17, 0x98,
        ];

        assert_eq!(message.len(), MESSAGE_SIZE, "Message must be exactly 33 bytes");

        // Expected SHA256 hash
        let expected = vec![
            0x0f, 0x71, 0x5b, 0xaf, 0x5d, 0x4c, 0x2e, 0xd3, 0x29, 0x78, 0x5c, 0xef, 0x29, 0xe5,
            0x62, 0xf7, 0x34, 0x88, 0xc8, 0xa2, 0xbb, 0x9d, 0xbc, 0x57, 0x00, 0xb3, 0x61, 0xd5,
            0x4b, 0x9b, 0x05, 0x54,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_sha256_33_bytes_example_1() {
        let mut ocl = Sha256_33Bytes::new().unwrap();

        // Another compressed public key example
        let message = vec![
            0x02, 0xc6, 0x04, 0x7f, 0x94, 0x41, 0xed, 0x7d, 0x6d, 0x30, 0x45, 0x40, 0x6e, 0x95,
            0xc0, 0x7c, 0xd8, 0x5c, 0x77, 0x8e, 0x4b, 0x8c, 0xef, 0x3c, 0xa7, 0xab, 0xac, 0x09,
            0xb9, 0x5c, 0x70, 0x9e, 0xe5,
        ];

        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected = vec![
            0xb1, 0xc9, 0x93, 0x8f, 0x01, 0x12, 0x1e, 0x15, 0x98, 0x87, 0xac, 0x2c, 0x8d, 0x39,
            0x3a, 0x22, 0xe4, 0x47, 0x6f, 0xf8, 0x21, 0x2d, 0xe1, 0x3f, 0xe1, 0x93, 0x9d, 0xe2,
            0xa2, 0x36, 0xf0, 0xa7,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_sha256_33_bytes_example_2() {
        let mut ocl = Sha256_33Bytes::new().unwrap();

        // Example with 0x03 prefix (odd y)
        let message = vec![
            0x03, 0xff, 0xf9, 0x7b, 0xd5, 0x75, 0x5e, 0xee, 0xa4, 0x20, 0x45, 0x3a, 0x14, 0x35,
            0x52, 0x35, 0xd3, 0x82, 0xf6, 0x47, 0x2f, 0x85, 0x68, 0xa1, 0x8b, 0x2f, 0x05, 0x7a,
            0x14, 0x60, 0x29, 0x75, 0x56,
        ];

        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected = vec![
            0xc7, 0xd9, 0xba, 0x2f, 0xa1, 0x49, 0x6c, 0x81, 0xbe, 0x20, 0x03, 0x8e, 0x5c, 0x60,
            0x8f, 0x2f, 0xd5, 0xd0, 0x24, 0x6d, 0x86, 0x43, 0x78, 0x37, 0x30, 0xdf, 0x6c, 0x2b,
            0xbb, 0x85, 0x5c, 0xb2,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }
}
