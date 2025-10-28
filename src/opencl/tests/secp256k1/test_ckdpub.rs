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
        let chain_code =
            hex::decode("873dff81c02f525623fd1fe5167eac3a55a049de3d314bb42ee227ffed37d508")
                .unwrap();

        // Parent (m - master) public key
        let k_par_x =
            hex::decode("39a36013301597daef41fbe593a02cc513d0b55527ec2df1050e2e8ff49c85c2")
                .unwrap();
        let k_par_y =
            hex::decode("3cbe7ded0e7ce6a594896b8f62888fdbc5c8821305e2ea42bf01e37300116281")
                .unwrap();

        // Child index (non-hardened)
        let index = 3u32;

        // Expected child (m/3) compressed public key
        // Y ends in 0x4a (even), so prefix is 0x02
        let expected_compressed_key =
            hex::decode("02c85080e00080aa933f93a2718bba9f01fd6fdc8e4712a155849f5ba588666471")
                .unwrap();

        let compressed_key = ocl
            .derive_child(chain_code, k_par_x, k_par_y, index)
            .unwrap();

        assert_eq!(compressed_key, expected_compressed_key, "Compressed key mismatch");
    }

    #[test]
    fn test_ckdpub_vanity_1test() {
        let mut ocl = CKDpub::new().unwrap();

        let chain_code =
            hex::decode("52c12f3457a48962a019040f2e9f0c4d4a0c1bf0cc45ebd3622ce563d48a2667")
                .unwrap();
        let k_par_x =
            hex::decode("ef43c9f45b3078c836b995c425f9fe612bd53ffa7fc8f025cbc5e76352bce9a1")
                .unwrap();
        let k_par_y =
            hex::decode("b39ebe21b159b7525b063110338989d7ff27888e71300c548c032edf9254c546")
                .unwrap();

        let compressed_key = ocl.derive_child(chain_code, k_par_x, k_par_y, 436).unwrap();

        // Y ends in 0xad (odd), so prefix is 0x03
        let expected_compressed_key =
            hex::decode("03de89c8d6ebdb6b510209da8b4868311a42e62469ec4ad143ccec7958ee7f2fe8")
                .unwrap();

        assert_eq!(
            compressed_key, expected_compressed_key,
            "Vanity address compressed key mismatch"
        );
    }
}
