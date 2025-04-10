#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct Secp256k1DoublePoint {
        x_buffer: Buffer<u8>,
        y_buffer: Buffer<u8>,
        x_result_buffer: Buffer<u8>,
        y_result_buffer: Buffer<u8>,
        secp256k1_double_point_kernel: Kernel,
    }

    impl Secp256k1DoublePoint {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let x_buffer = Self::new_buffer(&queue, 32)?;
            let y_buffer = Self::new_buffer(&queue, 32)?;

            let x_result_buffer = Self::new_buffer(&queue, 32)?;
            let y_result_buffer = Self::new_buffer(&queue, 32)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let secp256k1_double_point_kernel = match Kernel::builder()
                .program(&program)
                .name("secp256k1DoublePointKernel")
                .queue(queue.clone())
                .arg(&x_buffer)
                .arg(&y_buffer)
                .arg(&x_result_buffer)
                .arg(&y_result_buffer)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                x_buffer,
                y_buffer,
                x_result_buffer,
                y_result_buffer,
                secp256k1_double_point_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/secp256k1DoublePointKernel"));

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
            let mut data = vec![0u8; 32];
            match buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data)
        }

        fn double_point(
            &mut self,
            point: (Vec<u8>, Vec<u8>),
        ) -> Result<(Vec<u8>, Vec<u8>), String> {
            if point.0.len() != 32 || point.1.len() != 32 {
                return Err(format!(
                    "Input vectors must be 32 bytes long, got xa: {}, ya: {}",
                    point.0.len(),
                    point.1.len()
                ));
            }

            // Clone the buffer to avoid borrowing issues
            self.write_to_buffer(&self.x_buffer.clone(), point.0)?;
            self.write_to_buffer(&self.y_buffer.clone(), point.1)?;

            // Execute kernel
            unsafe {
                match self.secp256k1_double_point_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let x_result_array = self.read_from_buffer(&self.x_result_buffer.clone())?;
            let y_result_array = self.read_from_buffer(&self.y_result_buffer.clone())?;

            Ok((x_result_array, y_result_array))
        }
    }

    #[test]
    fn test_double_random_point_0() {
        let mut ocl = Secp256k1DoublePoint::new().unwrap();

        let point = (
            vec![
                0xc7, 0x15, 0x50, 0x87, 0xb7, 0xfd, 0x30, 0x46, 0x65, 0x14, 0x0b, 0x0d, 0xbc, 0x5c,
                0x86, 0xde, 0x4e, 0xcb, 0x3d, 0x07, 0x50, 0xf6, 0xdf, 0x7a, 0x8b, 0xa0, 0x97, 0x04,
                0xd4, 0xa9, 0x99, 0x00,
            ],
            vec![
                0x2d, 0x9b, 0x0a, 0x39, 0xa0, 0xc2, 0xe6, 0x36, 0x74, 0x96, 0x02, 0x3f, 0x15, 0x89,
                0xd0, 0xbd, 0xa7, 0x79, 0x3b, 0x4b, 0x23, 0x12, 0xec, 0x94, 0x56, 0x76, 0x37, 0x22,
                0x62, 0xaf, 0x7f, 0xb8,
            ],
        );

        let expected = (
            vec![
                0x0d, 0xf3, 0xf8, 0x27, 0xc8, 0x18, 0x9b, 0xc2, 0xef, 0x5b, 0x9b, 0xf8, 0x94, 0xf9,
                0xae, 0xf8, 0xf9, 0x44, 0xc8, 0x74, 0x38, 0x00, 0x9e, 0xc9, 0x23, 0x55, 0x40, 0x8e,
                0xc9, 0xb9, 0x50, 0x4b,
            ],
            vec![
                0x96, 0x78, 0xd0, 0x40, 0x8f, 0x65, 0x84, 0x9c, 0x94, 0x5e, 0x53, 0x13, 0xde, 0xe7,
                0x65, 0x6e, 0x1a, 0xd1, 0xa0, 0x6e, 0xcd, 0xbc, 0x99, 0x3e, 0xbe, 0xb3, 0x15, 0xda,
                0x22, 0x61, 0xb4, 0xdb,
            ],
        );

        let result = ocl.double_point(point).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_double_random_point_1() {
        let mut ocl = Secp256k1DoublePoint::new().unwrap();

        let point = (
            vec![
                0xa9, 0x71, 0x35, 0xd7, 0xe6, 0xe0, 0x79, 0x8f, 0xc5, 0x95, 0xa4, 0x98, 0xb7, 0xcf,
                0x47, 0x4f, 0x6e, 0xd9, 0x99, 0xcc, 0x5e, 0xbe, 0xec, 0xd6, 0xfc, 0x76, 0x40, 0xb5,
                0xb9, 0xfc, 0x90, 0x8a,
            ],
            vec![
                0xb0, 0x6d, 0x34, 0x9e, 0x80, 0xd3, 0x18, 0x61, 0xd3, 0xd9, 0xed, 0x0b, 0xb0, 0x9f,
                0x14, 0x94, 0x73, 0x91, 0xa9, 0x83, 0x61, 0x26, 0xc9, 0xc7, 0x66, 0x40, 0xa7, 0xb3,
                0xc5, 0xe1, 0xd8, 0x69,
            ],
        );

        let expected = (
            vec![
                0xa7, 0x03, 0x98, 0x5e, 0x38, 0x44, 0x29, 0x1c, 0x3c, 0xb2, 0x3b, 0xf6, 0xca, 0xf8,
                0xa5, 0x7b, 0x18, 0x67, 0x0f, 0x79, 0x19, 0x77, 0xe0, 0x87, 0xf6, 0xff, 0x39, 0x97,
                0x0f, 0xd0, 0xce, 0xb3,
            ],
            vec![
                0x3f, 0xa8, 0xe1, 0x0a, 0x87, 0x22, 0xe1, 0xbe, 0x7e, 0xbe, 0xde, 0xbf, 0x18, 0x39,
                0x7c, 0xb0, 0x84, 0x6f, 0x85, 0x3c, 0x2f, 0x73, 0x7a, 0xb8, 0xd6, 0x62, 0x13, 0x7c,
                0x64, 0xd9, 0x76, 0x71,
            ],
        );

        let result = ocl.double_point(point).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_double_random_point_2() {
        let mut ocl = Secp256k1DoublePoint::new().unwrap();

        let point = (
            vec![
                0x32, 0xbc, 0x0f, 0xb8, 0x05, 0xb9, 0x47, 0x99, 0x4a, 0x9b, 0xdf, 0x80, 0x08, 0xb7,
                0x13, 0x59, 0x19, 0xcc, 0x18, 0x4a, 0x81, 0xe1, 0x7d, 0x8e, 0xde, 0x10, 0xec, 0x0e,
                0x04, 0x6f, 0x9c, 0xcf,
            ],
            vec![
                0xcb, 0x09, 0x62, 0x00, 0x56, 0xc6, 0xbc, 0x23, 0x8f, 0xf0, 0x5f, 0xb2, 0x25, 0xcc,
                0x49, 0x39, 0xc8, 0x34, 0x4b, 0x27, 0xbe, 0xc9, 0x7f, 0xfb, 0xa5, 0x9f, 0xcf, 0x44,
                0xbe, 0x5b, 0x00, 0x5d,
            ],
        );

        let expected = (
            vec![
                0x80, 0xe3, 0x27, 0x21, 0xce, 0xc4, 0x88, 0x1c, 0x33, 0x60, 0x14, 0x6e, 0x93, 0x0d,
                0x5a, 0xc1, 0x38, 0xc1, 0xb6, 0x56, 0x7b, 0x7e, 0xd2, 0xa0, 0xc3, 0x2f, 0x23, 0x78,
                0x79, 0xc2, 0x4a, 0x96,
            ],
            vec![
                0xb7, 0xd3, 0xd7, 0x38, 0xd8, 0x68, 0x14, 0xac, 0xf1, 0x9f, 0xf3, 0x14, 0xab, 0x3d,
                0x21, 0xdf, 0x39, 0xb2, 0xbf, 0xd8, 0xb2, 0x71, 0xd6, 0x79, 0x3c, 0xa0, 0xef, 0xc2,
                0xd2, 0xb3, 0x3f, 0xc4,
            ],
        );

        let result = ocl.double_point(point).unwrap();

        assert_eq!(result, expected);
    }
}
