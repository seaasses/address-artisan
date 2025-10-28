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

        let message = hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
            .unwrap();

        assert_eq!(message.len(), MESSAGE_SIZE, "Message must be exactly 32 bytes");

        let expected = hex::decode("b472a266d0bd89c13706a4132ccfb16f7c3b9fcb")
            .unwrap();

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_ripemd160_32_bytes_sha256_genesis_pubkey() {
        let mut ocl = Ripemd160_32Bytes::new().unwrap();

        let message = hex::decode("0f715baf5d4c2ed329785cef29e562f73488c8a2bb9dbc5700b361d54b9b0554")
            .unwrap();

        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected = hex::decode("751e76e8199196d454941c45d1b3a323f1433bd6")
            .unwrap();

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_ripemd160_32_bytes_example_1() {
        let mut ocl = Ripemd160_32Bytes::new().unwrap();

        let message = hex::decode("b1c9938f01121e159887ac2c8d393a22e4476ff8212de13fe1939de2a236f0a7")
            .unwrap();

        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected = hex::decode("06afd46bcdfd22ef94ac122aa11f241244a37ecc")
            .unwrap();

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_ripemd160_32_bytes_satoshi_quote() {
        let mut ocl = Ripemd160_32Bytes::new().unwrap();

        // SHA256("I've been working on a new electronic cash system that's fully peer-to-peer, with no trusted third party.")
        let message = hex::decode("3eb721aeb7677a34a48b243bc5731a7217bf8ce2a0c0ca1729f91768bd907442")
            .unwrap();

        assert_eq!(message.len(), MESSAGE_SIZE);

        // RIPEMD160 of above
        let expected = hex::decode("c3a5ce3660cbca4bacf5a8a9b632b197818f8d5d")
            .unwrap();

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_ripemd160_32_bytes_freedom_quote() {
        let mut ocl = Ripemd160_32Bytes::new().unwrap();

        // SHA256("Freedom is the mother, not the daughter, of order.")
        let message = hex::decode("67cb23be92ebd05744b7922ced6611bd54629293c0ab29f378ec1b84fe409a04")
            .unwrap();

        assert_eq!(message.len(), MESSAGE_SIZE);

        // RIPEMD160 of above
        let expected = hex::decode("f6dac9b50455db5ac58a8b0ea3c60d9b5d68f03c")
            .unwrap();

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_ripemd160_32_bytes_sound_money() {
        let mut ocl = Ripemd160_32Bytes::new().unwrap();

        // SHA256("Sound money is a prerequisite for limited government.")
        let message = hex::decode("61b0b7b526428548d73e4e0de2a0cea5f4a3756d4d0aef4075be6f60e9aab745")
            .unwrap();

        assert_eq!(message.len(), MESSAGE_SIZE);

        // RIPEMD160 of above
        let expected = hex::decode("b258c78e0399da11437af64abe68ba3cc794146c")
            .unwrap();

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }
}
