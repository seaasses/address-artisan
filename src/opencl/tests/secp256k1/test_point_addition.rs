#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct Secp256k1DoublePoint {
        x1_buffer: Buffer<u8>,
        y1_buffer: Buffer<u8>,
        x2_buffer: Buffer<u8>,
        y2_buffer: Buffer<u8>,
        x_result_buffer: Buffer<u8>,
        y_result_buffer: Buffer<u8>,
        secp256k1_point_addition_kernel: Kernel,
    }

    impl Secp256k1DoublePoint {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let x1_buffer = Self::new_buffer(&queue, 32)?;
            let y1_buffer = Self::new_buffer(&queue, 32)?;
            let x2_buffer = Self::new_buffer(&queue, 32)?;
            let y2_buffer = Self::new_buffer(&queue, 32)?;

            let x_result_buffer = Self::new_buffer(&queue, 32)?;
            let y_result_buffer = Self::new_buffer(&queue, 32)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let secp256k1_point_addition_kernel = match Kernel::builder()
                .program(&program)
                .name("secp256k1PointAdditionKernel")
                .queue(queue.clone())
                .arg(&x1_buffer)
                .arg(&y1_buffer)
                .arg(&x2_buffer)
                .arg(&y2_buffer)
                .arg(&x_result_buffer)
                .arg(&y_result_buffer)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                x1_buffer,
                y1_buffer,
                x2_buffer,
                y2_buffer,
                x_result_buffer,
                y_result_buffer,
                secp256k1_point_addition_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/secp256k1PointAdditionKernel"));

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

        fn point_addition(
            &mut self,
            point1: (Vec<u8>, Vec<u8>),
            point2: (Vec<u8>, Vec<u8>),
        ) -> Result<(Vec<u8>, Vec<u8>), String> {
            if point1.0.len() != 32
                || point1.1.len() != 32
                || point2.0.len() != 32
                || point2.1.len() != 32
            {
                return Err(format!(
                    "Input vectors must be 32 bytes long, got xa: {}, ya: {}, xb: {}, yb: {}",
                    point1.0.len(),
                    point1.1.len(),
                    point2.0.len(),
                    point2.1.len()
                ));
            }

            // Clone the buffer to avoid borrowing issues
            self.write_to_buffer(&self.x1_buffer.clone(), point1.0)?;
            self.write_to_buffer(&self.y1_buffer.clone(), point1.1)?;
            self.write_to_buffer(&self.x2_buffer.clone(), point2.0)?;
            self.write_to_buffer(&self.y2_buffer.clone(), point2.1)?;

            // Execute kernel
            unsafe {
                match self.secp256k1_point_addition_kernel.enq() {
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
    fn test_point_addition_0() {
        let mut ocl = Secp256k1DoublePoint::new().unwrap();

        let point1 = (
            vec![
                0x8c, 0x18, 0x3c, 0xc4, 0x04, 0xc9, 0xca, 0x65, 0x90, 0x46, 0x6e, 0xed, 0xd0, 0x95,
                0x89, 0x9b, 0xfd, 0x60, 0xdb, 0x5e, 0x8c, 0x59, 0x0c, 0xb4, 0x8c, 0x78, 0xd1, 0x4e,
                0xc9, 0x06, 0xbd, 0x24,
            ],
            vec![
                0x5a, 0x59, 0x0f, 0x53, 0x05, 0xd9, 0xb9, 0x66, 0xc8, 0x45, 0x2a, 0xab, 0x62, 0x13,
                0xbf, 0x06, 0xac, 0xb9, 0x2d, 0xa7, 0x53, 0xe3, 0xdb, 0x96, 0xbf, 0xc8, 0xe9, 0x70,
                0xd0, 0xbe, 0x07, 0xb5,
            ],
        );

        let point2 = (
            vec![
                0xdd, 0x79, 0x5f, 0xb3, 0xb8, 0x3f, 0x54, 0x5d, 0x04, 0x66, 0x52, 0xf4, 0x32, 0x6a,
                0x76, 0x4c, 0x3c, 0x22, 0xc6, 0x9f, 0xc5, 0xd9, 0x02, 0xc7, 0xbd, 0xb5, 0x42, 0xc0,
                0x84, 0xea, 0xcb, 0x10,
            ],
            vec![
                0x32, 0x7e, 0xc5, 0xbd, 0x2f, 0xcc, 0x86, 0x81, 0xe3, 0x1c, 0xc1, 0x51, 0xbe, 0xf9,
                0x9c, 0x65, 0xd0, 0xc7, 0xd6, 0xf1, 0x76, 0xbd, 0x5b, 0x60, 0x54, 0xc9, 0xbf, 0xd0,
                0x05, 0x84, 0xde, 0xaf,
            ],
        );

        let expected = (
            vec![
                0x43, 0x93, 0x1c, 0x95, 0x22, 0x87, 0x98, 0x7f, 0x6c, 0xdc, 0x9e, 0x73, 0x31, 0x47,
                0x11, 0x25, 0x97, 0x40, 0x98, 0xd2, 0x89, 0x12, 0xbe, 0xff, 0xbb, 0x4b, 0xec, 0xcf,
                0x0d, 0x0e, 0xee, 0xb3,
            ],
            vec![
                0x67, 0x4a, 0x1d, 0xbe, 0x3a, 0x4f, 0xaf, 0x2b, 0x28, 0x7d, 0x44, 0x7f, 0xbb, 0xf6,
                0xad, 0x29, 0x67, 0xc4, 0xca, 0x00, 0x8a, 0x59, 0xfc, 0xe4, 0x58, 0x65, 0x53, 0x15,
                0xcb, 0xd0, 0x59, 0x11,
            ],
        );

        let result = ocl.point_addition(point1, point2).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_point_addition_1() {
        let mut ocl = Secp256k1DoublePoint::new().unwrap();

        let point1 = (
            vec![
                0x73, 0x81, 0xf8, 0xbe, 0xfd, 0x32, 0x30, 0xf2, 0x21, 0xb9, 0x36, 0xd0, 0x2e, 0xf7,
                0x33, 0x8e, 0x57, 0x0c, 0x90, 0x1e, 0x68, 0x21, 0x1a, 0xbf, 0xb4, 0x47, 0x3a, 0x30,
                0x7e, 0x85, 0x49, 0x3a,
            ],
            vec![
                0x37, 0x28, 0x2e, 0x24, 0xa7, 0xd8, 0xc8, 0x1b, 0x6e, 0xe6, 0x6c, 0x98, 0xe3, 0x8f,
                0x89, 0xa9, 0x57, 0x2c, 0x6c, 0xe6, 0x88, 0xd3, 0x4c, 0x69, 0x95, 0x39, 0x1a, 0xff,
                0x0a, 0x7e, 0x01, 0xe9,
            ],
        );

        let point2 = (
            vec![
                0xfb, 0x6e, 0x54, 0x13, 0x49, 0x0a, 0x5f, 0xb0, 0x52, 0xe5, 0xa8, 0x18, 0xde, 0x02,
                0xea, 0xb5, 0xf3, 0x7d, 0xee, 0x27, 0xb0, 0x3a, 0x59, 0x1a, 0xc5, 0x81, 0x3e, 0xc7,
                0x7d, 0x8b, 0x63, 0x7a,
            ],
            vec![
                0x9e, 0xf5, 0xa4, 0x28, 0x1d, 0xa7, 0x2a, 0x63, 0x24, 0xfe, 0x87, 0xba, 0x8e, 0x94,
                0x18, 0xb2, 0xe5, 0xbe, 0x8f, 0x97, 0x3a, 0x5c, 0x97, 0xb7, 0x4e, 0x3e, 0x64, 0x34,
                0x2b, 0x88, 0x24, 0x42,
            ],
        );

        let expected = (
            vec![
                0xf7, 0xa3, 0xe7, 0x85, 0xc9, 0xe1, 0x2e, 0x18, 0xfd, 0xd4, 0x60, 0x7c, 0x40, 0xae,
                0x1b, 0x06, 0x5b, 0xee, 0xf8, 0x90, 0xd0, 0x37, 0x39, 0x27, 0xaa, 0xcb, 0x63, 0x82,
                0x22, 0x24, 0xb9, 0xc3,
            ],
            vec![
                0xfa, 0x92, 0x6a, 0x57, 0x20, 0x6d, 0x15, 0x21, 0xd9, 0xf2, 0xdb, 0x8a, 0xe1, 0xf7,
                0x53, 0xce, 0xed, 0x9e, 0xf0, 0x59, 0x5a, 0x19, 0xb6, 0xb1, 0x9d, 0xb1, 0x9d, 0xd1,
                0x14, 0x3e, 0x21, 0xee,
            ],
        );

        let result = ocl.point_addition(point1, point2).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_point_addition_2() {
        let mut ocl = Secp256k1DoublePoint::new().unwrap();

        let point1 = (
            vec![
                0xeb, 0x46, 0x4c, 0x22, 0xae, 0xf9, 0xe2, 0xd4, 0x5a, 0x51, 0x09, 0x8d, 0xf0, 0xd0,
                0xf1, 0xf7, 0xe4, 0x34, 0xa2, 0xf8, 0x10, 0x6f, 0xc6, 0x96, 0xaf, 0x80, 0xdd, 0x80,
                0xce, 0xec, 0xbc, 0x0e,
            ],
            vec![
                0xe3, 0x62, 0xc2, 0xe8, 0xb2, 0xfe, 0x55, 0xd7, 0xaa, 0xc8, 0xad, 0x67, 0xb3, 0xa1,
                0xe9, 0x1a, 0xd2, 0xc3, 0x62, 0x07, 0x92, 0x2f, 0xf6, 0xff, 0x15, 0x36, 0x28, 0x89,
                0xa0, 0x5d, 0xa2, 0x68,
            ],
        );

        let point2 = (
            vec![
                0x52, 0x4c, 0xb7, 0x55, 0xbf, 0x6c, 0xd4, 0x20, 0xf8, 0x49, 0x93, 0xc3, 0xea, 0xa2,
                0x93, 0x88, 0x52, 0xdf, 0x11, 0xba, 0x14, 0xf1, 0xf9, 0x2e, 0xd3, 0xbe, 0x3a, 0xdd,
                0x85, 0xd5, 0xbb, 0x1c,
            ],
            vec![
                0x50, 0xb1, 0x04, 0x7d, 0x2d, 0x59, 0xf6, 0xea, 0xf2, 0x12, 0xfe, 0x30, 0x23, 0xc6,
                0xc3, 0x6e, 0x55, 0x90, 0x6a, 0x53, 0x51, 0x9c, 0xab, 0xd6, 0x04, 0x10, 0xdb, 0xbf,
                0x29, 0xec, 0x25, 0xdc,
            ],
        );

        let expected = (
            vec![
                0x49, 0x3c, 0xdf, 0x31, 0x66, 0x9e, 0x6d, 0xf5, 0x51, 0x99, 0xf2, 0xf7, 0x0a, 0xe3,
                0xed, 0x6b, 0xb9, 0xc4, 0x57, 0x27, 0xa5, 0x32, 0x79, 0x80, 0x25, 0x7d, 0xb9, 0xac,
                0x9a, 0xb8, 0xb8, 0x0d,
            ],
            vec![
                0x26, 0x8b, 0x19, 0x6e, 0xb5, 0x3d, 0x3e, 0x4e, 0x15, 0x05, 0x87, 0xc5, 0xbf, 0x39,
                0x0a, 0xbf, 0xfb, 0x96, 0x73, 0xde, 0x13, 0xe6, 0x9d, 0xc7, 0xa7, 0x13, 0xeb, 0x62,
                0x74, 0x62, 0xfe, 0xc0,
            ],
        );

        let result = ocl.point_addition(point1, point2).unwrap();

        assert_eq!(result, expected);
    }
}
