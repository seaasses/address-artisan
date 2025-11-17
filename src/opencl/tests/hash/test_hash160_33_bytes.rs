#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    const MESSAGE_SIZE: usize = 33; // Compressed public key
    const HASH_SIZE: usize = 20;    // Hash160 output

    pub struct Hash160_33Bytes {
        message_buffer: Buffer<u8>,
        hash_buffer: Buffer<u8>,
        hash160_33_bytes_kernel: Kernel,
    }

    impl Hash160_33Bytes {
        pub fn new() -> Result<Self, String> {
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            let message_buffer = Self::new_buffer(&queue, MESSAGE_SIZE)?;
            let hash_buffer = Self::new_buffer(&queue, HASH_SIZE)?;

            let program = Self::build_program(device, context)?;

            let hash160_33_bytes_kernel = match Kernel::builder()
                .program(&program)
                .name("hash160_33_bytes_kernel")
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
                hash160_33_bytes_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/hash160_33_bytes_kernel"));

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

            self.write_to_buffer(&self.message_buffer.clone(), message)?;

            unsafe {
                match self.hash160_33_bytes_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let hash = self.read_from_buffer(&self.hash_buffer.clone())?;

            Ok(hash)
        }
    }

    #[test]
    fn test_hash160_genesis_pubkey_compressed() {
        let mut ocl = Hash160_33Bytes::new().unwrap();

        let pubkey = vec![
            0x04, 0x67, 0x8a, 0xfd, 0xb0, 0xfe, 0x55, 0x48, 0x27, 0x19, 0x67, 0xf1, 0xa6, 0x71,
            0x30, 0xb7, 0x10, 0x5c, 0xd6, 0xa8, 0x28, 0xe0, 0x39, 0x09, 0xa6, 0x79, 0x62, 0xe0,
            0xea, 0x1f, 0x61, 0xde, 0xb6,
        ];
        assert_eq!(pubkey.len(), MESSAGE_SIZE);

        let expected = vec![
            0x84, 0x6c, 0x2c, 0xcd, 0xfe, 0xfd, 0x29, 0xbd, 0x78, 0x05, 0x77, 0xac, 0x4d, 0xf8,
            0x1a, 0xc9, 0xd1, 0x0f, 0x0a, 0x72,
        ];
        assert_eq!(ocl.hash(pubkey).unwrap(), expected);
    }

    #[test]
    fn test_hash160_compressed_pubkey_02() {
        let mut ocl = Hash160_33Bytes::new().unwrap();

        let pubkey = vec![
            0x02, 0x50, 0x86, 0x3a, 0xd6, 0x4a, 0x87, 0xae, 0x8a, 0x2f, 0xe8, 0x3c, 0x1a, 0xf1,
            0xa8, 0x40, 0x3c, 0xb5, 0x3f, 0x53, 0xe4, 0x86, 0xd8, 0x51, 0x1d, 0xad, 0x8a, 0x04,
            0x88, 0x7e, 0x5b, 0x23, 0x52,
        ];
        assert_eq!(pubkey.len(), MESSAGE_SIZE);

        let expected = vec![
            0xf5, 0x4a, 0x58, 0x51, 0xe9, 0x37, 0x2b, 0x87, 0x81, 0x0a, 0x8e, 0x60, 0xcd, 0xd2,
            0xe7, 0xcf, 0xd8, 0x0b, 0x6e, 0x31,
        ];
        assert_eq!(ocl.hash(pubkey).unwrap(), expected);
    }

    #[test]
    fn test_hash160_compressed_pubkey_03() {
        let mut ocl = Hash160_33Bytes::new().unwrap();

        let pubkey = vec![
            0x03, 0x89, 0x44, 0xff, 0xa9, 0x54, 0x36, 0x97, 0x37, 0xa3, 0x23, 0x77, 0xf6, 0x75,
            0xb9, 0xf3, 0xe1, 0xdb, 0x85, 0xc0, 0xec, 0x5d, 0x4d, 0xdf, 0x9d, 0x92, 0xce, 0xab,
            0xa5, 0xc8, 0x23, 0xda, 0xd8,
        ];
        assert_eq!(pubkey.len(), MESSAGE_SIZE);

        let expected = vec![
            0x71, 0xd2, 0x75, 0x42, 0x2c, 0x80, 0x68, 0xed, 0xeb, 0xe6, 0xe8, 0x8f, 0xf8, 0x9c,
            0xf6, 0xd0, 0xf5, 0x9e, 0x98, 0xd5,
        ];
        assert_eq!(ocl.hash(pubkey).unwrap(), expected);
    }

    #[test]
    fn test_hash160_all_zeros() {
        let mut ocl = Hash160_33Bytes::new().unwrap();
        let pubkey = vec![0x00; 33];
        assert_eq!(pubkey.len(), MESSAGE_SIZE);

        let expected = vec![
            0x29, 0xcf, 0xc6, 0x37, 0x62, 0x55, 0xa7, 0x84, 0x51, 0xee, 0xb4, 0xb1, 0x29, 0xed,
            0x8e, 0xac, 0xff, 0xa2, 0xfe, 0xef,
        ];
        assert_eq!(ocl.hash(pubkey).unwrap(), expected);
    }

    #[test]
    fn test_hash160_all_ff() {
        let mut ocl = Hash160_33Bytes::new().unwrap();
        let pubkey = vec![0xFF; 33];
        assert_eq!(pubkey.len(), MESSAGE_SIZE);

        let expected = vec![
            0xf7, 0x27, 0x93, 0xa5, 0xd7, 0x08, 0x21, 0x2c, 0x14, 0x13, 0x78, 0xdc, 0xb2, 0x27,
            0xc5, 0xe3, 0x95, 0x8c, 0xba, 0xec,
        ];
        assert_eq!(ocl.hash(pubkey).unwrap(), expected);
    }

    #[test]
    fn test_hash160_sequential() {
        let mut ocl = Hash160_33Bytes::new().unwrap();
        let pubkey: Vec<u8> = (0..33).collect();
        assert_eq!(pubkey.len(), MESSAGE_SIZE);

        let expected = vec![
            0xc3, 0x1b, 0x1d, 0x87, 0xd3, 0x52, 0xc7, 0xf1, 0x7b, 0xc1, 0xe2, 0x49, 0x42, 0xb0,
            0x5b, 0xdd, 0x4c, 0x33, 0x87, 0xea,
        ];
        assert_eq!(ocl.hash(pubkey).unwrap(), expected);
    }

    #[test]
    fn test_hash160_satoshi_block1_pubkey() {
        let mut ocl = Hash160_33Bytes::new().unwrap();

        let pubkey = vec![
            0x04, 0x11, 0xdb, 0x93, 0xe1, 0xdc, 0xdb, 0x8a, 0x01, 0x6b, 0x49, 0x84, 0x0f, 0x8c,
            0x53, 0xbc, 0x1e, 0xb6, 0x8a, 0x38, 0x2e, 0x97, 0xb1, 0x48, 0x2e, 0xca, 0xd7, 0xb1,
            0x48, 0xa6, 0x90, 0x9a, 0x5c,
        ];
        assert_eq!(pubkey.len(), MESSAGE_SIZE);

        let expected = vec![
            0x19, 0x3c, 0x12, 0x36, 0x3e, 0x5e, 0xdc, 0xe3, 0x5f, 0x0a, 0x64, 0x57, 0x98, 0xd0,
            0x6e, 0xa5, 0x53, 0x83, 0x9c, 0xf4,
        ];
        assert_eq!(ocl.hash(pubkey).unwrap(), expected);
    }

    #[test]
    fn test_hash160_random_data_1() {
        let mut ocl = Hash160_33Bytes::new().unwrap();

        let pubkey = vec![
            0x02, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22,
            0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00,
            0x11, 0x22, 0x33, 0x44, 0x55,
        ];
        assert_eq!(pubkey.len(), MESSAGE_SIZE);

        let expected = vec![
            0x86, 0x50, 0x2c, 0x1e, 0x7c, 0x9f, 0x29, 0x47, 0x1d, 0x12, 0x31, 0x7a, 0xfd, 0x40,
            0x75, 0x53, 0x84, 0xa5, 0x48, 0x7f,
        ];
        assert_eq!(ocl.hash(pubkey).unwrap(), expected);
    }

    #[test]
    fn test_hash160_even_y_coordinate() {
        let mut ocl = Hash160_33Bytes::new().unwrap();

        let pubkey = vec![
            0x02, 0xc6, 0x04, 0x7f, 0x94, 0x41, 0xed, 0x7d, 0x6d, 0x30, 0x45, 0x40, 0x6e, 0x95,
            0xc0, 0x7c, 0xd8, 0x5c, 0x77, 0x8e, 0x4b, 0x8c, 0xef, 0x3c, 0xa7, 0xab, 0xac, 0x09,
            0xb9, 0x5c, 0x70, 0x9e, 0xe5,
        ];
        assert_eq!(pubkey.len(), MESSAGE_SIZE);

        let expected = vec![
            0x06, 0xaf, 0xd4, 0x6b, 0xcd, 0xfd, 0x22, 0xef, 0x94, 0xac, 0x12, 0x2a, 0xa1, 0x1f,
            0x24, 0x12, 0x44, 0xa3, 0x7e, 0xcc,
        ];
        assert_eq!(ocl.hash(pubkey).unwrap(), expected);
    }

    #[test]
    fn test_hash160_odd_y_coordinate() {
        let mut ocl = Hash160_33Bytes::new().unwrap();

        let pubkey = vec![
            0x03, 0xc6, 0x04, 0x7f, 0x94, 0x41, 0xed, 0x7d, 0x6d, 0x30, 0x45, 0x40, 0x6e, 0x95,
            0xc0, 0x7c, 0xd8, 0x5c, 0x77, 0x8e, 0x4b, 0x8c, 0xef, 0x3c, 0xa7, 0xab, 0xac, 0x09,
            0xb9, 0x5c, 0x70, 0x9e, 0xe5,
        ];
        assert_eq!(pubkey.len(), MESSAGE_SIZE);

        let expected = vec![
            0xee, 0x61, 0x20, 0x57, 0x5d, 0xb9, 0x4b, 0x9e, 0xfd, 0xd9, 0xc7, 0xf5, 0x62, 0x12,
            0xb2, 0x3e, 0x7f, 0xf6, 0xe5, 0x6f,
        ];
        assert_eq!(ocl.hash(pubkey).unwrap(), expected);
    }
}
