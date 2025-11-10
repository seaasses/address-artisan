#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    const KEY_SIZE: usize = 32;
    const MESSAGE_SIZE: usize = 37;
    const HASH_SIZE: usize = 64;

    pub struct HmacSha512Key32Msg37 {
        key_buffer: Buffer<u8>,
        message_buffer: Buffer<u8>,
        hash_buffer: Buffer<u8>,
        hmac_sha512_key32_msg37_kernel: Kernel,
    }

    impl HmacSha512Key32Msg37 {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let key_buffer = Self::new_buffer(&queue, KEY_SIZE)?;
            let message_buffer = Self::new_buffer(&queue, MESSAGE_SIZE)?;
            let hash_buffer = Self::new_buffer(&queue, HASH_SIZE)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let hmac_sha512_key32_msg37_kernel = match Kernel::builder()
                .program(&program)
                .name("hmac_sha512_key32_msg37_kernel")
                .queue(queue.clone())
                .arg(&key_buffer)
                .arg(&message_buffer)
                .arg(&hash_buffer)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                key_buffer,
                message_buffer,
                hash_buffer,
                hmac_sha512_key32_msg37_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/hmac_sha512_key32_msg37_kernel"));

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

        fn hmac(&mut self, key: Vec<u8>, message: Vec<u8>) -> Result<Vec<u8>, String> {
            if key.len() != KEY_SIZE {
                return Err(format!(
                    "Key must be exactly {} bytes, got {}",
                    KEY_SIZE,
                    key.len()
                ));
            }

            if message.len() != MESSAGE_SIZE {
                return Err(format!(
                    "Message must be exactly {} bytes, got {}",
                    MESSAGE_SIZE,
                    message.len()
                ));
            }

            // Clone the buffers to avoid borrowing issues
            self.write_to_buffer(&self.key_buffer.clone(), key)?;
            self.write_to_buffer(&self.message_buffer.clone(), message)?;

            // Execute kernel
            unsafe {
                match self.hmac_sha512_key32_msg37_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let hash = self.read_from_buffer(&self.hash_buffer.clone())?;

            Ok(hash)
        }
    }

    #[test]
    fn test_hmac_sha512_key32_msg37_freedom() {
        let mut ocl = HmacSha512Key32Msg37::new().unwrap();

        let mut key = b"Freedom is the mother, not the ".to_vec();
        let message = b"daughter, of order.                  ".to_vec();

        while key.len() < KEY_SIZE {
            key.push(b' ');
        }

        assert_eq!(key.len(), KEY_SIZE);
        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected: Vec<u8> = vec![
            0x42, 0x8e, 0x90, 0x10, 0x52, 0x40, 0x11, 0x74, 0xc4, 0xbb, 0x5a, 0x70, 0x4d, 0x6b,
            0x3a, 0xe6, 0x43, 0xb0, 0x3a, 0xa9, 0xb5, 0x37, 0xcd, 0xee, 0x9a, 0xca, 0x45, 0xe9,
            0x07, 0x2c, 0xfa, 0x24, 0xdd, 0xf8, 0xf9, 0xbd, 0xf6, 0x8c, 0xc7, 0xf9, 0x71, 0x35,
            0x2c, 0x49, 0x37, 0x49, 0x8e, 0x34, 0x34, 0x89, 0x4c, 0xde, 0xf5, 0xcb, 0xec, 0x1b,
            0x2a, 0xd1, 0xa3, 0x55, 0xad, 0xe2, 0x74, 0x8f,
        ];

        let result = ocl.hmac(key, message).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_hmac_sha512_key32_msg37_minimal_state() {
        let mut ocl = HmacSha512Key32Msg37::new().unwrap();

        let mut key = b"The minimal state is the most ".to_vec();
        let message = b"extensive state that can be justified".to_vec();

        while key.len() < KEY_SIZE {
            key.push(b' ');
        }

        assert_eq!(key.len(), KEY_SIZE);
        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected: Vec<u8> = vec![
            0x73, 0x8b, 0xa5, 0xf7, 0x49, 0xae, 0xbb, 0x39, 0x94, 0xba, 0x43, 0x86, 0x11, 0xf7,
            0x47, 0xe9, 0xc6, 0x5c, 0x15, 0xc8, 0x24, 0x46, 0xfd, 0x62, 0x6e, 0x04, 0xae, 0xaa,
            0x5b, 0xb1, 0x69, 0x6c, 0x69, 0xf1, 0x8f, 0xd0, 0x85, 0x5b, 0xf3, 0x57, 0xba, 0xa3,
            0xdd, 0xad, 0x53, 0x35, 0x4e, 0x7c, 0x2d, 0x1f, 0xaf, 0xcc, 0x06, 0xcb, 0xfa, 0x42,
            0xff, 0x91, 0x04, 0x1c, 0x7a, 0xaa, 0x3d, 0x2e,
        ];

        let result = ocl.hmac(key, message).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_hmac_sha512_key32_msg37_state() {
        let mut ocl = HmacSha512Key32Msg37::new().unwrap();

        let key = b"The State is not, and never has ".to_vec();
        let message = b"been, a protector of society.        ".to_vec();

        assert_eq!(key.len(), KEY_SIZE);
        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected: Vec<u8> = vec![
            0x87, 0xbc, 0xd9, 0x3d, 0x93, 0x56, 0x81, 0x34, 0x45, 0x74, 0xc8, 0x2d, 0x36, 0x60,
            0x25, 0x30, 0xbc, 0x86, 0x5c, 0x5d, 0x0c, 0xd3, 0x8e, 0x71, 0x35, 0x41, 0x6f, 0x85,
            0xa0, 0xf1, 0x07, 0x82, 0xc2, 0x0f, 0x61, 0xe8, 0xc2, 0x46, 0x60, 0x7c, 0x69, 0x2c,
            0xa2, 0x77, 0x21, 0x5c, 0xcf, 0x9b, 0xcc, 0x99, 0x1d, 0xa1, 0xf7, 0x72, 0xa0, 0x1f,
            0x66, 0x73, 0xd4, 0x1c, 0xe5, 0x95, 0xe7, 0x3e,
        ];

        let result = ocl.hmac(key, message).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_hmac_sha512_key32_msg37_fix_the_money() {
        let mut ocl = HmacSha512Key32Msg37::new().unwrap();

        let mut key = b"Fix the money, fix the world. ".to_vec();
        let message = b"Fix the money, fix the world. Fix the".to_vec();

        while key.len() < KEY_SIZE {
            key.push(b' ');
        }

        assert_eq!(key.len(), KEY_SIZE);
        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected: Vec<u8> = vec![
            0x7d, 0x82, 0xbd, 0x75, 0x03, 0x51, 0xab, 0xb5, 0xeb, 0xda, 0x56, 0xec, 0x39, 0x6d,
            0x3e, 0xbe, 0x89, 0xd5, 0x97, 0xe5, 0xae, 0xb0, 0x64, 0x4e, 0x85, 0x04, 0xcb, 0x99,
            0xdf, 0x7c, 0xca, 0xfa, 0x74, 0xe0, 0xda, 0xd5, 0xe5, 0x72, 0x3b, 0xc1, 0x29, 0xf5,
            0x66, 0x4d, 0xf6, 0xf4, 0x03, 0x6d, 0xe5, 0xcc, 0x81, 0x10, 0x7e, 0xd2, 0x36, 0x61,
            0x89, 0xe2, 0x18, 0xe8, 0xdc, 0x19, 0xca, 0xee,
        ];

        let result = ocl.hmac(key, message).unwrap();

        assert_eq!(result, expected);
    }
}
