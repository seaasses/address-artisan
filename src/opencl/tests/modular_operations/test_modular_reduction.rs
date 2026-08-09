#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct ModularReduction {
        x_buffer: Buffer<u8>,
        result_buffer: Buffer<u8>,
        modular_reduction_kernel: Kernel,
    }

    impl ModularReduction {
        pub fn new() -> Result<Self, String> {
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            let x_buffer = Self::new_buffer(&queue, 64)?; // Uint512 = 64 bytes
            let result_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes

            let program = Self::build_program(device, context)?;

            let modular_reduction_kernel = match Kernel::builder()
                .program(&program)
                .name("modular_reduction_kernel")
                .queue(queue.clone())
                .arg(&x_buffer)
                .arg(&result_buffer)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                x_buffer,
                result_buffer,
                modular_reduction_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/modular_reduction_kernel"));

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

        fn reduction(&mut self, x: Vec<u8>) -> Result<Vec<u8>, String> {
            if x.len() != 64 {
                return Err(format!("Input 'x' must be 64 bytes long, got: {}", x.len()));
            }

            match self.x_buffer.write(&x[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing to buffer: ".to_string() + &e.to_string()),
            };

            unsafe {
                match self.modular_reduction_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let mut data = vec![0u8; 32];
            match self.result_buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data)
        }
    }

    #[test]
    fn test_modular_reduction_zero() {
        let mut ocl = ModularReduction::new().unwrap();
        let x = vec![0u8; 64];
        let expected = vec![0u8; 32];
        assert_eq!(ocl.reduction(x).unwrap(), expected);
    }

    #[test]
    fn test_modular_reduction_p_reduces_to_zero() {
        let mut ocl = ModularReduction::new().unwrap();
        // x = p (secp256k1 prime) in the low 256 bits
        let x = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xfe, 0xff, 0xff, 0xfc, 0x2f,
        ];
        let expected = vec![0u8; 32];
        assert_eq!(ocl.reduction(x).unwrap(), expected);
    }

    #[test]
    fn test_modular_reduction_p_plus_5() {
        let mut ocl = ModularReduction::new().unwrap();
        let x = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xfe, 0xff, 0xff, 0xfc, 0x34,
        ];
        let mut expected = vec![0u8; 32];
        expected[31] = 0x05;
        assert_eq!(ocl.reduction(x).unwrap(), expected);
    }

    #[test]
    fn test_modular_reduction_max_512() {
        let mut ocl = ModularReduction::new().unwrap();
        // x = 2^512 - 1  =>  x mod p = 0x1000007a2000e90a0
        let x = vec![0xffu8; 64];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x07, 0xa2,
            0x00, 0x0e, 0x90, 0xa0,
        ];
        assert_eq!(ocl.reduction(x).unwrap(), expected);
    }

    #[test]
    fn test_modular_reduction_p_minus_1_squared() {
        let mut ocl = ModularReduction::new().unwrap();
        // x = (p-1)^2 (full 512-bit)  =>  x mod p = 1
        let x = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfd,
            0xff, 0xff, 0xf8, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x07, 0xa4, 0x00, 0x0e, 0x98, 0x44,
        ];
        assert_eq!(x.len(), 64);
        let mut expected = vec![0u8; 32];
        expected[31] = 0x01;
        assert_eq!(ocl.reduction(x).unwrap(), expected);
    }

    #[test]
    fn test_modular_reduction_2pow255_shifted() {
        // x = 2^511 (top bit of the 512-bit input). Stresses the second fold
        // where the remainder spans a 32-bit limb boundary.
        // x mod p = 0x8000...0000800003d080074668
        let mut ocl = ModularReduction::new().unwrap();
        let mut x = vec![0u8; 64];
        x[0] = 0x80;
        let expected = vec![
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x03, 0xd0,
            0x80, 0x07, 0x46, 0x68,
        ];
        assert_eq!(ocl.reduction(x).unwrap(), expected);
    }

    #[test]
    fn test_modular_reduction_known_product() {
        let mut ocl = ModularReduction::new().unwrap();
        // x = 0xd358...4e23 * 0x76ba...e83a (the two big numbers from the
        // modular multiplication test); x mod p matches that test's expected.
        let x = vec![
            0x62, 0x04, 0x43, 0x6c, 0x48, 0x93, 0x54, 0x92, 0xcf, 0x2b, 0xca, 0x81, 0xab, 0x35,
            0xe7, 0x04, 0x91, 0x53, 0x24, 0x37, 0xee, 0x4f, 0xfe, 0x5d, 0x73, 0xe2, 0x94, 0x71,
            0x92, 0x7c, 0x5e, 0xcb, 0xf0, 0x22, 0xce, 0x48, 0xdf, 0x0c, 0xf1, 0x92, 0x75, 0x3c,
            0xa5, 0xca, 0x2c, 0xa2, 0x20, 0x79, 0x8a, 0x82, 0x83, 0xfa, 0xcc, 0xf6, 0x80, 0x78,
            0xdf, 0xad, 0xe8, 0x74, 0xbc, 0x66, 0x6b, 0xee,
        ];
        let expected = vec![
            0x4a, 0xfb, 0x73, 0x1c, 0xa8, 0x7e, 0x80, 0x5c, 0xc6, 0x92, 0x65, 0xad, 0x26, 0xab,
            0xed, 0x20, 0x17, 0x1f, 0xbb, 0xcc, 0xc0, 0x22, 0xd7, 0x92, 0x17, 0xe7, 0x13, 0x08,
            0xdb, 0x57, 0x16, 0xfc,
        ];
        assert_eq!(ocl.reduction(x).unwrap(), expected);
    }
}
