#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    const MESSAGE_SIZE: usize = 165;
    const HASH_SIZE: usize = 64;

    pub struct Sha512_165Bytes {
        message_buffer: Buffer<u8>,
        hash_buffer: Buffer<u8>,
        sha512_165_bytes_kernel: Kernel,
    }

    impl Sha512_165Bytes {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let message_buffer = Self::new_buffer(&queue, MESSAGE_SIZE)?;
            let hash_buffer = Self::new_buffer(&queue, HASH_SIZE)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let sha512_165_bytes_kernel = match Kernel::builder()
                .program(&program)
                .name("sha512_165_bytes_kernel")
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
                sha512_165_bytes_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/sha512_165_bytes_kernel"));

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

        fn write_to_buffer(
            &mut self,
            buffer: &Buffer<u8>,
            data: Vec<u8>,
        ) -> Result<(), String> {
            match buffer.write(&data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing to buffer: ".to_string() + &e.to_string()),
            };
            Ok(())
        }

        fn read_from_buffer(&mut self, buffer: &Buffer<u8>) -> Result<Vec<u8>, String> {
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
                match self.sha512_165_bytes_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let hash = self.read_from_buffer(&self.hash_buffer.clone())?;

            Ok(hash)
        }
    }

    #[test]
    fn test_sha512_165_bytes_executive_order_6102() {
        let mut ocl = Sha512_165Bytes::new().unwrap();

        let message = b"all GOLD COIN, GOLD BULLION, AND GOLD CERTIFICATES now owned by them to a Federal Reserve Bank, branch or agency, or to any member bank of the Federal Reserve System".to_vec();

        assert_eq!(message.len(), MESSAGE_SIZE, "Message must be exactly 165 bytes");

        let expected: Vec<u8> = vec![
            0x91, 0x8c, 0x5a, 0xd0, 0x0b, 0x72, 0xa9, 0xd9,
            0x36, 0x2a, 0x5c, 0xb3, 0x63, 0x46, 0x25, 0xb3,
            0x6d, 0x3c, 0xc3, 0xcf, 0x3e, 0x6a, 0x8c, 0x52,
            0x9b, 0xf4, 0x7a, 0x8f, 0x6a, 0xa8, 0x7b, 0xf4,
            0xfc, 0x1c, 0x3f, 0xf7, 0xca, 0x1b, 0xbf, 0x54,
            0x7d, 0x9c, 0x1a, 0xb6, 0x82, 0xfd, 0x41, 0x26,
            0x35, 0x19, 0x87, 0xf6, 0x67, 0xa2, 0x1f, 0x09,
            0xd1, 0xa2, 0x91, 0x22, 0xc0, 0x2c, 0xfb, 0xb4,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_sha512_165_bytes_satoshi_nakamoto() {
        let mut ocl = Sha512_165Bytes::new().unwrap();

        // Satoshi's famous quote from the Bitcoin whitepaper announcement (padded to 165 bytes with spaces)
        let message = b"I've been working on a new electronic cash system that's fully peer-to-peer, with no trusted third party.                                                            ".to_vec();

        assert_eq!(message.len(), MESSAGE_SIZE, "Message must be exactly 165 bytes");

        let expected: Vec<u8> = vec![
            0x76, 0xa5, 0x98, 0x8c, 0x49, 0x9b, 0xcd, 0xcb,
            0xe2, 0x5a, 0x35, 0x93, 0x49, 0x37, 0x77, 0xa6,
            0x1b, 0xc0, 0xb9, 0x3c, 0x6f, 0xa7, 0xa8, 0x7c,
            0x95, 0x33, 0x03, 0x60, 0x3e, 0xbd, 0xab, 0x20,
            0x7b, 0x6e, 0x8d, 0xab, 0xaf, 0x29, 0x31, 0x0d,
            0x5b, 0xa4, 0xfe, 0x23, 0x0f, 0x8d, 0xed, 0x0c,
            0x72, 0x82, 0xf1, 0x1e, 0x55, 0x3c, 0xbb, 0x83,
            0xd2, 0x4b, 0x41, 0x40, 0x4c, 0x63, 0x6b, 0x5c,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_sha512_165_bytes_satoshi_20_years() {
        let mut ocl = Sha512_165Bytes::new().unwrap();

        // Satoshi's prediction about Bitcoin's future (padded to 165 bytes with spaces)
        let message = b"I'm sure that in 20 years there will either be very large transaction volume or no volume.                                                                           ".to_vec();

        assert_eq!(message.len(), MESSAGE_SIZE, "Message must be exactly 165 bytes");

        let expected: Vec<u8> = vec![
            0xac, 0xc6, 0xfe, 0x4f, 0xb4, 0x31, 0x80, 0xa1,
            0x3e, 0xa5, 0x56, 0xd6, 0x70, 0xc3, 0x8b, 0x8e,
            0xa5, 0xfb, 0xa5, 0x6c, 0xce, 0x9a, 0x67, 0xea,
            0xb5, 0xc4, 0x53, 0x4b, 0x89, 0xa0, 0x9e, 0xcc,
            0x25, 0x41, 0x24, 0xdf, 0x0e, 0xd8, 0x67, 0xf3,
            0x2e, 0xbd, 0x9e, 0xf5, 0x26, 0xa0, 0xe0, 0x16,
            0x47, 0xf4, 0x85, 0xff, 0x47, 0xee, 0x8b, 0x87,
            0xd8, 0x1d, 0xca, 0x04, 0xad, 0x74, 0xe1, 0x09,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_sha512_165_bytes_satoshi_arms_race() {
        let mut ocl = Sha512_165Bytes::new().unwrap();

        // Satoshi on cryptography and freedom (padded to 165 bytes with spaces)
        let message = b"Yes, but we can win a major battle in the arms race and gain a new territory of freedom for several years.                                                           ".to_vec();

        assert_eq!(message.len(), MESSAGE_SIZE, "Message must be exactly 165 bytes");

        let expected: Vec<u8> = vec![
            0x26, 0x64, 0x5f, 0x91, 0x61, 0xc7, 0xa5, 0x1d,
            0x49, 0xb8, 0xaa, 0x34, 0x20, 0xbf, 0x90, 0xa2,
            0x7d, 0x1c, 0xea, 0xa2, 0x97, 0x6e, 0x6a, 0xbe,
            0xf2, 0x6b, 0xd3, 0x67, 0x15, 0x3c, 0x06, 0x5c,
            0x4e, 0x5f, 0x25, 0xdf, 0x3d, 0x0e, 0xfb, 0x67,
            0xa4, 0x85, 0x45, 0x88, 0x34, 0xac, 0xba, 0x6f,
            0xa4, 0x14, 0x69, 0xf4, 0xda, 0xfe, 0xb2, 0x86,
            0xac, 0xc9, 0xa4, 0x43, 0x73, 0x64, 0x11, 0xc6,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }
}
