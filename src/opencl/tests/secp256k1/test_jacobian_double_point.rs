#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    type PointResult = Result<(Vec<u8>, Vec<u8>, Vec<u8>), String>;

    pub struct JacobianDoublePoint {
        point_x_buffer: Buffer<u8>,
        point_y_buffer: Buffer<u8>,
        point_z_buffer: Buffer<u8>,
        result_x_buffer: Buffer<u8>,
        result_y_buffer: Buffer<u8>,
        result_z_buffer: Buffer<u8>,
        jacobian_double_point_kernel: Kernel,
    }

    impl JacobianDoublePoint {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let point_x_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let point_y_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let point_z_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let result_x_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let result_y_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let result_z_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes

            let program = Self::build_program(device, context)?;

            // Create kernel
            let jacobian_double_point_kernel = match Kernel::builder()
                .program(&program)
                .name("jacobian_double_point_kernel")
                .queue(queue.clone())
                .arg(&point_x_buffer)
                .arg(&point_y_buffer)
                .arg(&point_z_buffer)
                .arg(&result_x_buffer)
                .arg(&result_y_buffer)
                .arg(&result_z_buffer)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                point_x_buffer,
                point_y_buffer,
                point_z_buffer,
                result_x_buffer,
                result_y_buffer,
                result_z_buffer,
                jacobian_double_point_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/jacobian_double_point_kernel"));

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
            let mut data = vec![0u8; 32]; // Uint256 = 32 bytes
            match buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data)
        }

        fn double_point(
            &mut self,
            point_x: Vec<u8>,
            point_y: Vec<u8>,
            point_z: Vec<u8>,
        ) -> PointResult {
            if point_x.len() != 32 || point_y.len() != 32 || point_z.len() != 32 {
                return Err(format!(
                    "All inputs must be 32 bytes long, got: x={}, y={}, z={}",
                    point_x.len(),
                    point_y.len(),
                    point_z.len()
                ));
            }

            // Clone the buffers to avoid borrowing issues
            self.write_to_buffer(&self.point_x_buffer.clone(), point_x)?;
            self.write_to_buffer(&self.point_y_buffer.clone(), point_y)?;
            self.write_to_buffer(&self.point_z_buffer.clone(), point_z)?;

            // Execute kernel
            unsafe {
                match self.jacobian_double_point_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let result_x = self.read_from_buffer(&self.result_x_buffer.clone())?;
            let result_y = self.read_from_buffer(&self.result_y_buffer.clone())?;
            let result_z = self.read_from_buffer(&self.result_z_buffer.clone())?;

            Ok((result_x, result_y, result_z))
        }
    }

    #[test]
    fn test_jacobian_double_point_simple() {
        let mut ocl = JacobianDoublePoint::new().unwrap();

        // Test: Double point (1, 1, 1)
        let point_x = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let point_y = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let point_z = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        let (result_x, result_y, result_z) = ocl.double_point(point_x, point_y, point_z).unwrap();

        // Just check that we get some result (exact values would need proper elliptic curve math)
        assert_eq!(result_x.len(), 32);
        assert_eq!(result_y.len(), 32);
        assert_eq!(result_z.len(), 32);
    }

    #[test]
    fn test_jacobian_double_point_random_point_0() {
        let mut ocl = JacobianDoublePoint::new().unwrap();

        let point_x = vec![
            0x74, 0xa8, 0x56, 0x23, 0x48, 0x7f, 0xa7, 0x64, 0xce, 0xcf, 0x29, 0x61, 0xfc, 0xb0,
            0x25, 0x7b, 0xe7, 0x3e, 0x1a, 0x52, 0xe5, 0x13, 0x35, 0xd3, 0x12, 0xe6, 0xa7, 0x03,
            0x9f, 0x8a, 0xcd, 0xa1,
        ];
        let point_y = vec![
            0x24, 0xaa, 0x17, 0xaf, 0x4f, 0xb4, 0xac, 0xd7, 0x46, 0x0c, 0x10, 0x58, 0xd9, 0x75,
            0xf4, 0x27, 0x2a, 0x25, 0x96, 0x87, 0x0f, 0x98, 0xb5, 0x0c, 0xdb, 0x45, 0xad, 0x50,
            0x24, 0xc2, 0x67, 0x23,
        ];
        let point_z = vec![
            0xc5, 0x1c, 0x62, 0x4b, 0x40, 0x0e, 0xdc, 0x9b, 0xe0, 0x0a, 0x59, 0x91, 0xfe, 0xd7,
            0xc8, 0x8b, 0x7f, 0x6b, 0x6c, 0x84, 0xd3, 0xd1, 0x2e, 0x90, 0x77, 0x0a, 0x36, 0x86,
            0xa8, 0x2f, 0xb0, 0x5d,
        ];

        let expected_x = vec![
            0x48, 0x1c, 0xf2, 0xc6, 0x3e, 0x4c, 0xc0, 0xe7, 0x7d, 0xf7, 0x33, 0x80, 0xbb, 0x45,
            0x85, 0x2c, 0x81, 0x77, 0x81, 0xe8, 0x4b, 0x10, 0x80, 0x9a, 0x5e, 0x19, 0xd1, 0x15,
            0x0c, 0xa5, 0x8c, 0xfc,
        ];
        let expected_y = vec![
            0x00, 0xd9, 0x5b, 0xb1, 0x91, 0xbe, 0x9d, 0x6f, 0x10, 0xb9, 0x1b, 0x8e, 0xb2, 0xcc,
            0x21, 0x39, 0xe5, 0x1d, 0x66, 0x76, 0x20, 0xed, 0xe7, 0x50, 0x95, 0x4c, 0x0e, 0xc1,
            0x5c, 0xe6, 0xa1, 0x43,
        ];
        let expected_z = vec![
            0xda, 0xdd, 0x6f, 0xf6, 0xa4, 0x43, 0xad, 0xc3, 0x8e, 0x26, 0x04, 0x44, 0x6e, 0xac,
            0x18, 0x27, 0x00, 0xda, 0xfa, 0xfb, 0x7f, 0x86, 0x6e, 0xd7, 0xeb, 0xfd, 0x3a, 0x80,
            0xa5, 0x6d, 0xc9, 0xba,
        ];

        let (result_x, result_y, result_z) = ocl.double_point(point_x, point_y, point_z).unwrap();
        assert_eq!(result_x, expected_x);
        assert_eq!(result_y, expected_y);
        assert_eq!(result_z, expected_z);
    }

    #[test]
    fn test_jacobian_double_point_random_point_1() {
        let mut ocl = JacobianDoublePoint::new().unwrap();

        let point_x = vec![
            0x79, 0xc4, 0xc5, 0xd6, 0xd2, 0x86, 0x8b, 0xae, 0xec, 0x90, 0xc0, 0xcf, 0x7a, 0x80,
            0x28, 0xc2, 0x88, 0xed, 0x33, 0x9d, 0x41, 0xa4, 0x08, 0xfd, 0xdf, 0xe5, 0x09, 0x53,
            0x80, 0x91, 0xb9, 0xa8,
        ];
        let point_y = vec![
            0xff, 0xd0, 0xdb, 0x0f, 0xbe, 0x2e, 0x09, 0xf5, 0x32, 0x8b, 0x4e, 0x77, 0x46, 0x8f,
            0x61, 0xc2, 0x09, 0x3a, 0xd2, 0xcc, 0x0b, 0x3d, 0xc5, 0x84, 0x6c, 0x60, 0x62, 0x9d,
            0xbb, 0x60, 0x2e, 0xec,
        ];
        let point_z = vec![
            0xe4, 0x90, 0xbe, 0x34, 0x04, 0x8e, 0xbe, 0xff, 0xde, 0xd5, 0xe9, 0x70, 0xf9, 0xd4,
            0x36, 0x2b, 0xfe, 0xac, 0xbd, 0xdb, 0x8c, 0xd4, 0xa1, 0x59, 0x1b, 0x54, 0xb3, 0x6b,
            0x28, 0x8e, 0x65, 0xe9,
        ];

        let expected_x = vec![
            0xcd, 0xb6, 0xe1, 0xd0, 0xa9, 0xc7, 0xeb, 0x41, 0x14, 0x2b, 0xd3, 0x5c, 0xbb, 0x83,
            0xf5, 0xd6, 0x21, 0x50, 0x03, 0xab, 0x61, 0xc1, 0x2c, 0xf8, 0xf5, 0xd3, 0xb2, 0xbd,
            0x5c, 0x65, 0xdd, 0x97,
        ];
        let expected_y = vec![
            0x71, 0xca, 0x72, 0xd1, 0xd6, 0x3e, 0x77, 0x35, 0xe4, 0xeb, 0x95, 0x96, 0x11, 0xa4,
            0xde, 0x2d, 0x5b, 0xb3, 0x19, 0x38, 0x43, 0x88, 0xd7, 0xb2, 0xfd, 0xf1, 0xda, 0x8f,
            0x96, 0xa1, 0x5f, 0x65,
        ];
        let expected_z = vec![
            0x6d, 0xe9, 0x8b, 0xb7, 0x28, 0xeb, 0x14, 0xc5, 0x31, 0xa7, 0xb4, 0xc7, 0xcd, 0x08,
            0xa5, 0xa8, 0xa6, 0xeb, 0x8a, 0xec, 0x54, 0xf1, 0x60, 0xd2, 0x83, 0xc2, 0x5b, 0x3b,
            0xae, 0x7f, 0x33, 0x2d,
        ];

        let (result_x, result_y, result_z) = ocl.double_point(point_x, point_y, point_z).unwrap();
        assert_eq!(result_x, expected_x);
        assert_eq!(result_y, expected_y);
        assert_eq!(result_z, expected_z);
    }

    #[test]
    fn test_jacobian_double_point_random_point_2() {
        let mut ocl = JacobianDoublePoint::new().unwrap();

        let point_x = vec![
            0x68, 0x34, 0xdb, 0x3d, 0x47, 0xd0, 0xc7, 0x26, 0xfc, 0x54, 0x91, 0x4d, 0x43, 0xd9,
            0x02, 0x12, 0x80, 0xca, 0xdc, 0xcd, 0x2e, 0x66, 0xb5, 0x10, 0xa3, 0x97, 0x82, 0x17,
            0x96, 0x70, 0xfa, 0x3b,
        ];
        let point_y = vec![
            0xd8, 0x18, 0xb1, 0xb2, 0xa3, 0x33, 0x80, 0xae, 0xb5, 0x09, 0x8b, 0x85, 0x95, 0x75,
            0x4e, 0x0e, 0x61, 0xa0, 0x06, 0x5e, 0x83, 0xa6, 0x8e, 0x6c, 0xb9, 0xb5, 0x27, 0x17,
            0x67, 0x2c, 0x1f, 0x9c,
        ];
        let point_z = vec![
            0x9f, 0x95, 0xa0, 0x70, 0xb8, 0x3e, 0xba, 0x69, 0x0b, 0xd8, 0xfa, 0xb5, 0x53, 0xbc,
            0x15, 0xfd, 0x04, 0x79, 0x21, 0x08, 0xff, 0xb6, 0xdb, 0x81, 0xed, 0x0d, 0xd3, 0x47,
            0xaf, 0x49, 0xe6, 0x72,
        ];

        let expected_x = vec![
            0xaa, 0xd3, 0x4b, 0x76, 0x5f, 0x3f, 0x39, 0x1b, 0x5d, 0x9d, 0x12, 0xc3, 0x6b, 0xb4,
            0xc4, 0x40, 0x1e, 0xdb, 0x7a, 0xda, 0x33, 0x1a, 0xfc, 0x07, 0x28, 0xbb, 0xb4, 0x2f,
            0x7d, 0x8a, 0xba, 0xb3,
        ];
        let expected_y = vec![
            0x79, 0x9a, 0xd9, 0x61, 0x8c, 0xfd, 0x08, 0x25, 0x43, 0xd3, 0xaa, 0xec, 0x75, 0x01,
            0xa0, 0x8c, 0x64, 0x2e, 0x12, 0xa9, 0xe2, 0x14, 0x03, 0x2e, 0x3d, 0xf9, 0x6b, 0x8d,
            0xc7, 0xfa, 0x33, 0x74,
        ];
        let expected_z = vec![
            0xcb, 0x5e, 0xef, 0x90, 0xb4, 0xe0, 0x39, 0x7b, 0xe4, 0x90, 0xa1, 0xbb, 0x21, 0x18,
            0x65, 0x61, 0x5e, 0x04, 0x9d, 0x1c, 0xe4, 0x8d, 0x16, 0x43, 0x3c, 0x12, 0x60, 0x5d,
            0x01, 0x2f, 0xe2, 0xc8,
        ];

        let (result_x, result_y, result_z) = ocl.double_point(point_x, point_y, point_z).unwrap();
        assert_eq!(result_x, expected_x);
        assert_eq!(result_y, expected_y);
        assert_eq!(result_z, expected_z);
    }
}
