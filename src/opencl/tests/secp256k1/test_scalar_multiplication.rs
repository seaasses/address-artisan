#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct Secp256k1ScalarMultiplication {
        x_buffer: Buffer<u8>,
        y_buffer: Buffer<u8>,
        scalar_buffer: Buffer<u8>,
        x_result_buffer: Buffer<u8>,
        y_result_buffer: Buffer<u8>,
        secp256k1_scalar_multiplication_kernel: Kernel,
    }

    impl Secp256k1ScalarMultiplication {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let x_buffer = Self::new_buffer(&queue, 32)?;
            let y_buffer = Self::new_buffer(&queue, 32)?;
            let scalar_buffer = Self::new_buffer(&queue, 32)?;

            let x_result_buffer = Self::new_buffer(&queue, 32)?;
            let y_result_buffer = Self::new_buffer(&queue, 32)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let secp256k1_scalar_multiplication_kernel = match Kernel::builder()
                .program(&program)
                .name("secp256k1ScalarMultiplicationKernel")
                .queue(queue.clone())
                .arg(&x_buffer)
                .arg(&y_buffer)
                .arg(&scalar_buffer)
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
                scalar_buffer,
                x_result_buffer,
                y_result_buffer,
                secp256k1_scalar_multiplication_kernel,
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
            let src = include_str!(concat!(
                env!("OUT_DIR"),
                "/secp256k1ScalarMultiplicationKernel"
            ));

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

        fn scalar_multiplication(
            &mut self,
            point: (Vec<u8>, Vec<u8>),
            scalar: Vec<u8>,
        ) -> Result<(Vec<u8>, Vec<u8>), String> {
            if point.0.len() != 32 || point.1.len() != 32 || scalar.len() != 32 {
                return Err(format!(
                    "Input vectors must be 32 bytes long, got x: {}, y: {}, scalar: {}",
                    point.0.len(),
                    point.1.len(),
                    scalar.len()
                ));
            }

            // Clone the buffer to avoid borrowing issues
            self.write_to_buffer(&self.x_buffer.clone(), point.0)?;
            self.write_to_buffer(&self.y_buffer.clone(), point.1)?;
            self.write_to_buffer(&self.scalar_buffer.clone(), scalar)?;

            // Execute kernel
            unsafe {
                match self.secp256k1_scalar_multiplication_kernel.enq() {
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
        let mut ocl = Secp256k1ScalarMultiplication::new().unwrap();

        let point = (
            vec![
                0x0d, 0x36, 0x26, 0x6d, 0x1b, 0x04, 0x28, 0x3d, 0x71, 0xbe, 0x89, 0xe5, 0x57, 0xcf,
                0xc8, 0x4a, 0xca, 0x8a, 0xd8, 0x6f, 0xf0, 0x5d, 0x83, 0x18, 0x87, 0xb8, 0xd8, 0x06,
                0x74, 0xbe, 0x48, 0xb5,
            ],
            vec![
                0xf5, 0x2f, 0xfa, 0x5f, 0x6d, 0xf7, 0xc1, 0x8e, 0x80, 0x3e, 0xf7, 0x7c, 0xa5, 0x1f,
                0xb8, 0x97, 0x79, 0xb8, 0xb5, 0xf1, 0x61, 0x79, 0x52, 0xa6, 0xf2, 0x14, 0xe4, 0x82,
                0x42, 0x53, 0x57, 0x4c,
            ],
        );

        let scalar = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x0A,
        ];

        let expected = (
            vec![
                0x75, 0xd3, 0x6e, 0x58, 0x46, 0x93, 0x70, 0x90, 0xf1, 0xb9, 0xb7, 0x9f, 0xdd, 0xa8,
                0xb4, 0xeb, 0x78, 0xdd, 0x41, 0xa0, 0x5e, 0x12, 0x95, 0xa6, 0xcb, 0x0f, 0x8d, 0x98,
                0x6c, 0x1c, 0xb1, 0x50,
            ],
            vec![
                0x6a, 0x00, 0x95, 0xe5, 0xa0, 0xa2, 0x17, 0xda, 0x0e, 0xac, 0x4d, 0x18, 0xa0, 0xbe,
                0x30, 0xbe, 0x45, 0x7b, 0x6d, 0x36, 0x99, 0xd8, 0x20, 0x3a, 0x8f, 0xca, 0xf2, 0x9f,
                0x42, 0x2e, 0x62, 0x3b,
            ],
        );

        let result = ocl.scalar_multiplication(point, scalar).unwrap();

        assert_eq!(result, expected);
    }
}
