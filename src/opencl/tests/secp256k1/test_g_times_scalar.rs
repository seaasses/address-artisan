#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    const SECP256K1_P_MINUS_1: [u8; 32] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF,
        0xFC, 0x2E,
    ];

    pub struct GTimesScalar {
        scalar_buffer: Buffer<u8>,
        result_x_buffer: Buffer<u8>,
        result_y_buffer: Buffer<u8>,
        g_times_scalar_kernel: Kernel,
    }

    impl GTimesScalar {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let scalar_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let result_x_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let result_y_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes

            let program = Self::build_program(device, context)?;

            // Create kernel
            let g_times_scalar_kernel = match Kernel::builder()
                .program(&program)
                .name("g_times_scalar_kernel")
                .queue(queue.clone())
                .arg(&scalar_buffer)
                .arg(&result_x_buffer)
                .arg(&result_y_buffer)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                scalar_buffer,
                result_x_buffer,
                result_y_buffer,
                g_times_scalar_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/g_times_scalar_kernel"));

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
            let mut data = vec![0u8; 32]; // Uint256 = 32 bytes
            match buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data)
        }

        fn multiply(&mut self, scalar: Vec<u8>) -> Result<(Vec<u8>, Vec<u8>), String> {
            if scalar.len() != 32 {
                return Err(format!(
                    "Scalar must be 32 bytes long, got: {}",
                    scalar.len()
                ));
            }

            // Clone the buffers to avoid borrowing issues
            self.write_to_buffer(&self.scalar_buffer.clone(), scalar)?;

            // Execute kernel
            unsafe {
                match self.g_times_scalar_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let result_x = self.read_from_buffer(&self.result_x_buffer.clone())?;
            let result_y = self.read_from_buffer(&self.result_y_buffer.clone())?;

            Ok((result_x, result_y))
        }
    }

    #[test]
    fn test_g_times_scalar_simple() {
        let mut ocl = GTimesScalar::new().unwrap();

        // Test: G * 1 should give us the generator point G
        let scalar_1 = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        // Expected G point coordinates (secp256k1 generator)
        let expected_g_x = vec![
            0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87,
            0x0b, 0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b,
            0x16, 0xf8, 0x17, 0x98,
        ];
        let expected_g_y = vec![
            0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65, 0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11,
            0x08, 0xa8, 0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19, 0x9c, 0x47, 0xd0, 0x8f,
            0xfb, 0x10, 0xd4, 0xb8,
        ];

        let (result_x, result_y) = ocl.multiply(scalar_1).unwrap();
        assert_eq!(result_x, expected_g_x);
        assert_eq!(result_y, expected_g_y);
    }

    #[test]
    fn test_g_times_scalar_two() {
        let mut ocl = GTimesScalar::new().unwrap();

        // Test: G * 2
        let scalar_2 = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];

        // Expected 2G point coordinates
        let expected_2g_x = vec![
            0xc6, 0x04, 0x7f, 0x94, 0x41, 0xed, 0x7d, 0x6d, 0x30, 0x45, 0x40, 0x6e, 0x95, 0xc0,
            0x7c, 0xd8, 0x5c, 0x77, 0x8e, 0x4b, 0x8c, 0xef, 0x3c, 0xa7, 0xab, 0xac, 0x09, 0xb9,
            0x5c, 0x70, 0x9e, 0xe5,
        ];
        let expected_2g_y = vec![
            0x1a, 0xe1, 0x68, 0xfe, 0xa6, 0x3d, 0xc3, 0x39, 0xa3, 0xc5, 0x84, 0x19, 0x46, 0x6c,
            0xea, 0xee, 0xf7, 0xf6, 0x32, 0x65, 0x32, 0x66, 0xd0, 0xe1, 0x23, 0x64, 0x31, 0xa9,
            0x50, 0xcf, 0xe5, 0x2a,
        ];

        let (result_x, result_y) = ocl.multiply(scalar_2).unwrap();
        assert_eq!(result_x, expected_2g_x);
        assert_eq!(result_y, expected_2g_y);
    }

    #[test]
    fn test_generator_point_times_big_number_0() {
        let mut ocl = GTimesScalar::new().unwrap();

        let scalar = SECP256K1_P_MINUS_1.to_vec();

        // Expected point from C test: {0x02541d1403fc71a5, 0xd927923b20a673e7, 0x69284b16e7d1f597, 0xf9413dc64e82fc48}
        let expected_x = vec![
            0x02, 0x54, 0x1d, 0x14, 0x03, 0xfc, 0x71, 0xa5, 0xd9, 0x27, 0x92, 0x3b, 0x20, 0xa6,
            0x73, 0xe7, 0x69, 0x28, 0x4b, 0x16, 0xe7, 0xd1, 0xf5, 0x97, 0xf9, 0x41, 0x3d, 0xc6,
            0x4e, 0x82, 0xfc, 0x48,
        ];
        // Expected point from C test: {0x621239e38c2af6bc, 0x145db2424cf44bb8, 0xed12351858f67c7c, 0x6130a2f69fe4a28c}
        let expected_y = vec![
            0x62, 0x12, 0x39, 0xe3, 0x8c, 0x2a, 0xf6, 0xbc, 0x14, 0x5d, 0xb2, 0x42, 0x4c, 0xf4,
            0x4b, 0xb8, 0xed, 0x12, 0x35, 0x18, 0x58, 0xf6, 0x7c, 0x7c, 0x61, 0x30, 0xa2, 0xf6,
            0x9f, 0xe4, 0xa2, 0x8c,
        ];

        let (result_x, result_y) = ocl.multiply(scalar).unwrap();
        assert_eq!(result_x, expected_x);
        assert_eq!(result_y, expected_y);
    }

    #[test]
    fn test_generator_point_times_big_number_1() {
        let mut ocl = GTimesScalar::new().unwrap();

        // Scalar from C test: {0x6d914012b8952160, 0xa79d2f7573143c10, 0xfbf324b497bf9289, 0x2581e55b7f6aabf1}
        let scalar = vec![
            0x6d, 0x91, 0x40, 0x12, 0xb8, 0x95, 0x21, 0x60, 0xa7, 0x9d, 0x2f, 0x75, 0x73, 0x14,
            0x3c, 0x10, 0xfb, 0xf3, 0x24, 0xb4, 0x97, 0xbf, 0x92, 0x89, 0x25, 0x81, 0xe5, 0x5b,
            0x7f, 0x6a, 0xab, 0xf1,
        ];

        // Expected point from C test: {0xdf4a2e774212f80b, 0x688835a9eb6836bf, 0x6b7795372c3a1efb, 0x5e26e42bd28dc4ba}
        let expected_x = vec![
            0xdf, 0x4a, 0x2e, 0x77, 0x42, 0x12, 0xf8, 0x0b, 0x68, 0x88, 0x35, 0xa9, 0xeb, 0x68,
            0x36, 0xbf, 0x6b, 0x77, 0x95, 0x37, 0x2c, 0x3a, 0x1e, 0xfb, 0x5e, 0x26, 0xe4, 0x2b,
            0xd2, 0x8d, 0xc4, 0xba,
        ];
        // Expected point from C test: {0x2cb999fbf85dd51d, 0x4561e142a7b9bd5d, 0xf30b0bc986e37407, 0x48ba9374f6a51d1a}
        let expected_y = vec![
            0x2c, 0xb9, 0x99, 0xfb, 0xf8, 0x5d, 0xd5, 0x1d, 0x45, 0x61, 0xe1, 0x42, 0xa7, 0xb9,
            0xbd, 0x5d, 0xf3, 0x0b, 0x0b, 0xc9, 0x86, 0xe3, 0x74, 0x07, 0x48, 0xba, 0x93, 0x74,
            0xf6, 0xa5, 0x1d, 0x1a,
        ];

        let (result_x, result_y) = ocl.multiply(scalar).unwrap();
        assert_eq!(result_x, expected_x);
        assert_eq!(result_y, expected_y);
    }

    #[test]
    fn test_generator_point_times_big_number_2() {
        let mut ocl = GTimesScalar::new().unwrap();

        // Scalar from C test: {0xfde5e382aa2ef05d, 0xb142e7acfb9cd795, 0xd4756d96d3aeccb8, 0x4c8e74c94999ad0f}
        let scalar = vec![
            0xfd, 0xe5, 0xe3, 0x82, 0xaa, 0x2e, 0xf0, 0x5d, 0xb1, 0x42, 0xe7, 0xac, 0xfb, 0x9c,
            0xd7, 0x95, 0xd4, 0x75, 0x6d, 0x96, 0xd3, 0xae, 0xcc, 0xb8, 0x4c, 0x8e, 0x74, 0xc9,
            0x49, 0x99, 0xad, 0x0f,
        ];

        // Expected point from C test: {0x8b4ab5b746742718, 0x8eecdaa0f5e5e643, 0x10d08ee9ea07529d, 0x1638dae7c82176c3}
        let expected_x = vec![
            0x8b, 0x4a, 0xb5, 0xb7, 0x46, 0x74, 0x27, 0x18, 0x8e, 0xec, 0xda, 0xa0, 0xf5, 0xe5,
            0xe6, 0x43, 0x10, 0xd0, 0x8e, 0xe9, 0xea, 0x07, 0x52, 0x9d, 0x16, 0x38, 0xda, 0xe7,
            0xc8, 0x21, 0x76, 0xc3,
        ];
        // Expected point from C test: {0x6f4310e4f9218059, 0x0e195da5df3215e8, 0x64f245d87a784209, 0x29e440e9f2f61c98}
        let expected_y = vec![
            0x6f, 0x43, 0x10, 0xe4, 0xf9, 0x21, 0x80, 0x59, 0x0e, 0x19, 0x5d, 0xa5, 0xdf, 0x32,
            0x15, 0xe8, 0x64, 0xf2, 0x45, 0xd8, 0x7a, 0x78, 0x42, 0x09, 0x29, 0xe4, 0x40, 0xe9,
            0xf2, 0xf6, 0x1c, 0x98,
        ];

        let (result_x, result_y) = ocl.multiply(scalar).unwrap();
        assert_eq!(result_x, expected_x);
        assert_eq!(result_y, expected_y);
    }
}
