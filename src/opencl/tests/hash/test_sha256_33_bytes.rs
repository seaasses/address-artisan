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
        let message = hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
            .unwrap();

        assert_eq!(message.len(), MESSAGE_SIZE, "Message must be exactly 33 bytes");

        // Expected SHA256 hash
        let expected = hex::decode("0f715baf5d4c2ed329785cef29e562f73488c8a2bb9dbc5700b361d54b9b0554")
            .unwrap();

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_sha256_33_bytes_example_1() {
        let mut ocl = Sha256_33Bytes::new().unwrap();

        // Another compressed public key example
        let message = hex::decode("02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5")
            .unwrap();

        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected = hex::decode("b1c9938f01121e159887ac2c8d393a22e4476ff8212de13fe1939de2a236f0a7")
            .unwrap();

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }

    #[test]
    fn test_sha256_33_bytes_example_2() {
        let mut ocl = Sha256_33Bytes::new().unwrap();

        // Example with 0x03 prefix (odd y)
        let message = hex::decode("03fff97bd5755eeea420453a14355235d382f6472f8568a18b2f057a1460297556")
            .unwrap();

        assert_eq!(message.len(), MESSAGE_SIZE);

        let expected = hex::decode("c7d9ba2fa1496c81be20038e5c608f2fd5d0246d8643783730df6c2bbb855cb2")
            .unwrap();

        assert_eq!(ocl.hash(message).unwrap(), expected);
    }
}
