#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    const MESSAGE_SIZE: usize = 192;
    const HASH_SIZE: usize = 64;

    pub struct Sha512_192Bytes {
        message_buffer: Buffer<u8>,
        hash_buffer: Buffer<u8>,
        sha512_192_bytes_kernel: Kernel,
    }

    impl Sha512_192Bytes {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let message_buffer = Self::new_buffer(&queue, MESSAGE_SIZE)?;
            let hash_buffer = Self::new_buffer(&queue, HASH_SIZE)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let sha512_192_bytes_kernel = match Kernel::builder()
                .program(&program)
                .name("sha512_192_bytes_kernel")
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
                sha512_192_bytes_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/sha512_192_bytes_kernel"));

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

        fn write_to_buffer(&mut self, buffer: &Buffer<u8>, data: Vec<u8>) -> Result<(), String> {
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
                match self.sha512_192_bytes_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let hash = self.read_from_buffer(&self.hash_buffer.clone())?;

            Ok(hash)
        }
    }

    #[test]
    fn test_sha512_192_bytes_wealth_of_nations() {
        let mut ocl = Sha512_192Bytes::new().unwrap();

        // Adam Smith's famous quote from The Wealth of Nations (1776) - padded to 192 bytes with spaces
        let message = b"\"It is not from the benevolence of the butcher, the brewer, or the baker that we expect our dinner, but from their regard to their own interest.\" - The Wealth of Nations (1776)                ".to_vec();

        assert_eq!(
            message.len(),
            MESSAGE_SIZE,
            "Message must be exactly 192 bytes"
        );

        let expected: Vec<u8> = vec![
            0xbf, 0x0d, 0x19, 0xb1, 0xf3, 0x6b, 0x57, 0x21, 0x0b, 0x17, 0xab, 0x2f, 0xe3, 0x36,
            0x61, 0x6b, 0x48, 0xf5, 0x40, 0x00, 0xe7, 0xa7, 0x6b, 0x5a, 0x31, 0xad, 0xf4, 0x66,
            0x1e, 0xae, 0x36, 0x4f, 0x1f, 0xe9, 0xd8, 0xf5, 0x52, 0xa1, 0x6d, 0xaf, 0x8a, 0x40,
            0x6c, 0x27, 0x34, 0x68, 0xcb, 0xa0, 0xb3, 0x39, 0x82, 0x62, 0xfa, 0x35, 0x86, 0x21,
            0xbf, 0x9c, 0x35, 0xd8, 0x9f, 0x79, 0x61, 0xf4,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_sha512_192_bytes_hayek() {
        let mut ocl = Sha512_192Bytes::new().unwrap();

        // Friedrich Hayek on government and money - padded to 192 bytes with spaces
        let message = b"\"I don't believe we shall ever have a good money again before we take the thing out of the hands of government.\" - Friedrich Hayek                                                              ".to_vec();

        assert_eq!(
            message.len(),
            MESSAGE_SIZE,
            "Message must be exactly 192 bytes"
        );

        let expected: Vec<u8> = vec![
            0x84, 0xfd, 0x93, 0x2b, 0xb7, 0x5a, 0xf3, 0x39, 0x1d, 0xa3, 0x78, 0xcb, 0xc8, 0xb3,
            0x92, 0x41, 0xa0, 0x8e, 0x47, 0x3e, 0x92, 0xcd, 0x22, 0xaf, 0x23, 0x6a, 0x8b, 0x3c,
            0xab, 0xa9, 0xd0, 0xed, 0x6d, 0xe2, 0x67, 0xb9, 0x5b, 0xbf, 0xd8, 0xd1, 0x31, 0xe9,
            0x56, 0x20, 0x44, 0xfc, 0x90, 0xb6, 0x73, 0xc3, 0xa1, 0x7b, 0x3e, 0x1d, 0x5f, 0x50,
            0x21, 0xea, 0x2f, 0x20, 0x84, 0xc7, 0x3d, 0xa2,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_sha512_192_bytes_rothbard() {
        let mut ocl = Sha512_192Bytes::new().unwrap();

        let message = b"\"Inflation is not caused by the actions of private citizens, but by the government: by an artificial expansion of the money supply.\" - Murray Rothbard                                          ".to_vec();

        assert_eq!(
            message.len(),
            MESSAGE_SIZE,
            "Message must be exactly 192 bytes"
        );

        let expected: Vec<u8> = vec![
            0x32, 0xaa, 0xd1, 0xd1, 0x21, 0x51, 0x32, 0xa4, 0xb8, 0xe7, 0xf4, 0x62, 0x53, 0xb3,
            0x95, 0x9d, 0x44, 0x5b, 0xce, 0x2d, 0xac, 0x81, 0xe1, 0xc6, 0x45, 0x31, 0xdc, 0x7e,
            0x9b, 0x87, 0xaa, 0x51, 0xf6, 0x43, 0x11, 0xa3, 0xca, 0xf0, 0x2f, 0x95, 0x17, 0x71,
            0x2b, 0xa7, 0x25, 0xdf, 0xe7, 0x70, 0x6d, 0xea, 0xd4, 0x26, 0x2f, 0xe5, 0x27, 0xaa,
            0xba, 0x22, 0x53, 0xcb, 0x73, 0x51, 0x04, 0x66,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_sha512_192_bytes_greenspan() {
        let mut ocl = Sha512_192Bytes::new().unwrap();

        // Alan Greenspan on gold standard (1966) - padded to 192 bytes with spaces
        let message = b"\"In the absence of the gold standard, there is no way to protect savings from confiscation through inflation.\" - Alan Greenspan                                                                 ".to_vec();

        assert_eq!(
            message.len(),
            MESSAGE_SIZE,
            "Message must be exactly 192 bytes"
        );

        let expected: Vec<u8> = vec![
            0x96, 0xf9, 0x34, 0xc9, 0x6c, 0xfd, 0xe6, 0xed, 0x50, 0x09, 0xce, 0x2d, 0xb1, 0x6b,
            0xca, 0x52, 0x58, 0x75, 0xfb, 0x0e, 0x09, 0xdb, 0x44, 0x13, 0xdb, 0x07, 0xe0, 0x3e,
            0x3a, 0x87, 0xdb, 0xb7, 0x56, 0x29, 0xb3, 0x3c, 0x5e, 0xd3, 0xda, 0xd1, 0x2e, 0x9b,
            0x0b, 0xef, 0xec, 0x6f, 0xd3, 0x5f, 0x59, 0x3d, 0xf8, 0xf4, 0x04, 0x39, 0x26, 0x8e,
            0x30, 0xe2, 0x71, 0x7c, 0x7d, 0xe4, 0x89, 0xec,
        ];

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }
}
