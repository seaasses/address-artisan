#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct CKDpub {
        chain_code_buffer: Buffer<u8>,
        k_par_x_buffer: Buffer<u8>,
        k_par_y_buffer: Buffer<u8>,
        index_buffer: Buffer<u32>,
        compressed_key_buffer: Buffer<u8>,
        ckdpub_kernel: Kernel,
    }

    impl CKDpub {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let chain_code_buffer = Self::new_u8_buffer(&queue, 32)?;
            let k_par_x_buffer = Self::new_u8_buffer(&queue, 32)?;
            let k_par_y_buffer = Self::new_u8_buffer(&queue, 32)?;
            let index_buffer = Self::new_u32_buffer(&queue, 1)?;
            let compressed_key_buffer = Self::new_u8_buffer(&queue, 33)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let ckdpub_kernel = match Kernel::builder()
                .program(&program)
                .name("ckdpub_kernel")
                .queue(queue.clone())
                .arg(&chain_code_buffer)
                .arg(&k_par_x_buffer)
                .arg(&k_par_y_buffer)
                .arg(&index_buffer)
                .arg(&compressed_key_buffer)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                chain_code_buffer,
                k_par_x_buffer,
                k_par_y_buffer,
                index_buffer,
                compressed_key_buffer,
                ckdpub_kernel,
            })
        }

        fn new_u8_buffer(queue: &Queue, len: usize) -> Result<Buffer<u8>, String> {
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

        fn new_u32_buffer(queue: &Queue, len: usize) -> Result<Buffer<u32>, String> {
            let buffer = match Buffer::<u32>::builder()
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/ckdpub_kernel"));

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

        fn write_u8_buffer(
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

        fn write_u32_buffer(
            self: &mut Self,
            buffer: &Buffer<u32>,
            data: Vec<u32>,
        ) -> Result<(), String> {
            match buffer.write(&data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing to buffer: ".to_string() + &e.to_string()),
            };
            Ok(())
        }

        fn read_u8_buffer(
            self: &mut Self,
            buffer: &Buffer<u8>,
            len: usize,
        ) -> Result<Vec<u8>, String> {
            let mut data = vec![0u8; len];
            match buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data)
        }

        pub fn derive_child(
            &mut self,
            chain_code: Vec<u8>,
            k_par_x: Vec<u8>,
            k_par_y: Vec<u8>,
            index: u32,
        ) -> Result<Vec<u8>, String> {
            if chain_code.len() != 32 {
                return Err(format!(
                    "Chain code must be exactly 32 bytes, got {}",
                    chain_code.len()
                ));
            }

            if k_par_x.len() != 32 {
                return Err(format!(
                    "Parent key x must be exactly 32 bytes, got {}",
                    k_par_x.len()
                ));
            }

            if k_par_y.len() != 32 {
                return Err(format!(
                    "Parent key y must be exactly 32 bytes, got {}",
                    k_par_y.len()
                ));
            }

            // Write input to buffers
            self.write_u8_buffer(&self.chain_code_buffer.clone(), chain_code)?;
            self.write_u8_buffer(&self.k_par_x_buffer.clone(), k_par_x)?;
            self.write_u8_buffer(&self.k_par_y_buffer.clone(), k_par_y)?;
            self.write_u32_buffer(&self.index_buffer.clone(), vec![index])?;

            // Execute kernel
            unsafe {
                match self.ckdpub_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            // Read result
            let compressed_key = self.read_u8_buffer(&self.compressed_key_buffer.clone(), 33)?;

            Ok(compressed_key)
        }
    }

    #[test]
    fn test_ckdpub() {
        let mut ocl = CKDpub::new().unwrap();

        // Test data from BIP32 derivation: m -> m/3
        // Generated using Python bip32 library with seed: 000102030405060708090a0b0c0d0e0f

        // Parent (m - master) chain code
        let chain_code = vec![
            0x87, 0x3d, 0xff, 0x81, 0xc0, 0x2f, 0x52, 0x56, 0x23, 0xfd, 0x1f, 0xe5, 0x16, 0x7e,
            0xac, 0x3a, 0x55, 0xa0, 0x49, 0xde, 0x3d, 0x31, 0x4b, 0xb4, 0x2e, 0xe2, 0x27, 0xff,
            0xed, 0x37, 0xd5, 0x08,
        ];

        // Parent (m - master) public key
        let k_par_x = vec![
            0x39, 0xa3, 0x60, 0x13, 0x30, 0x15, 0x97, 0xda, 0xef, 0x41, 0xfb, 0xe5, 0x93, 0xa0,
            0x2c, 0xc5, 0x13, 0xd0, 0xb5, 0x55, 0x27, 0xec, 0x2d, 0xf1, 0x05, 0x0e, 0x2e, 0x8f,
            0xf4, 0x9c, 0x85, 0xc2,
        ];
        let k_par_y = vec![
            0x3c, 0xbe, 0x7d, 0xed, 0x0e, 0x7c, 0xe6, 0xa5, 0x94, 0x89, 0x6b, 0x8f, 0x62, 0x88,
            0x8f, 0xdb, 0xc5, 0xc8, 0x82, 0x13, 0x05, 0xe2, 0xea, 0x42, 0xbf, 0x01, 0xe3, 0x73,
            0x00, 0x11, 0x62, 0x81,
        ];

        // Child index (non-hardened)
        let index = 3u32;

        // Expected child (m/3) compressed public key
        // Y ends in 0x4a (even), so prefix is 0x02
        let expected_compressed_key = vec![
            0x02, 0xc8, 0x50, 0x80, 0xe0, 0x00, 0x80, 0xaa, 0x93, 0x3f, 0x93, 0xa2, 0x71, 0x8b,
            0xba, 0x9f, 0x01, 0xfd, 0x6f, 0xdc, 0x8e, 0x47, 0x12, 0xa1, 0x55, 0x84, 0x9f, 0x5b,
            0xa5, 0x88, 0x66, 0x64, 0x71,
        ];

        let compressed_key = ocl
            .derive_child(chain_code, k_par_x, k_par_y, index)
            .unwrap();

        assert_eq!(compressed_key, expected_compressed_key, "Compressed key mismatch");
    }

    #[test]
    fn test_ckdpub_vanity_1test() {
        let mut ocl = CKDpub::new().unwrap();

        let chain_code = vec![
            0x52, 0xc1, 0x2f, 0x34, 0x57, 0xa4, 0x89, 0x62, 0xa0, 0x19, 0x04, 0x0f, 0x2e, 0x9f,
            0x0c, 0x4d, 0x4a, 0x0c, 0x1b, 0xf0, 0xcc, 0x45, 0xeb, 0xd3, 0x62, 0x2c, 0xe5, 0x63,
            0xd4, 0x8a, 0x26, 0x67,
        ];
        let k_par_x = vec![
            0xef, 0x43, 0xc9, 0xf4, 0x5b, 0x30, 0x78, 0xc8, 0x36, 0xb9, 0x95, 0xc4, 0x25, 0xf9,
            0xfe, 0x61, 0x2b, 0xd5, 0x3f, 0xfa, 0x7f, 0xc8, 0xf0, 0x25, 0xcb, 0xc5, 0xe7, 0x63,
            0x52, 0xbc, 0xe9, 0xa1,
        ];
        let k_par_y = vec![
            0xb3, 0x9e, 0xbe, 0x21, 0xb1, 0x59, 0xb7, 0x52, 0x5b, 0x06, 0x31, 0x10, 0x33, 0x89,
            0x89, 0xd7, 0xff, 0x27, 0x88, 0x8e, 0x71, 0x30, 0x0c, 0x54, 0x8c, 0x03, 0x2e, 0xdf,
            0x92, 0x54, 0xc5, 0x46,
        ];

        let compressed_key = ocl.derive_child(chain_code, k_par_x, k_par_y, 436).unwrap();

        // Y ends in 0xad (odd), so prefix is 0x03
        let expected_compressed_key = vec![
            0x03, 0xde, 0x89, 0xc8, 0xd6, 0xeb, 0xdb, 0x6b, 0x51, 0x02, 0x09, 0xda, 0x8b, 0x48,
            0x68, 0x31, 0x1a, 0x42, 0xe6, 0x24, 0x69, 0xec, 0x4a, 0xd1, 0x43, 0xcc, 0xec, 0x79,
            0x58, 0xee, 0x7f, 0x2f, 0xe8,
        ];

        assert_eq!(
            compressed_key, expected_compressed_key,
            "Vanity address compressed key mismatch"
        );
    }
}
