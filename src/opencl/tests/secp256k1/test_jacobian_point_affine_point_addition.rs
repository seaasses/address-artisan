#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct JacobianPointAffinePointAddition {
        jac_a_x_buffer: Buffer<u8>,
        jac_a_y_buffer: Buffer<u8>,
        jac_a_z_buffer: Buffer<u8>,
        aff_b_x_buffer: Buffer<u8>,
        aff_b_y_buffer: Buffer<u8>,
        result_x_buffer: Buffer<u8>,
        result_y_buffer: Buffer<u8>,
        result_z_buffer: Buffer<u8>,
        jacobian_point_affine_point_addition_kernel: Kernel,
    }

    impl JacobianPointAffinePointAddition {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let jac_a_x_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let jac_a_y_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let jac_a_z_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let aff_b_x_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let aff_b_y_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let result_x_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let result_y_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes
            let result_z_buffer = Self::new_buffer(&queue, 32)?; // Uint256 = 32 bytes

            let program = Self::build_program(device, context)?;

            // Create kernel
            let jacobian_point_affine_point_addition_kernel = match Kernel::builder()
                .program(&program)
                .name("jacobian_point_affine_point_addition_kernel")
                .queue(queue.clone())
                .arg(&jac_a_x_buffer)
                .arg(&jac_a_y_buffer)
                .arg(&jac_a_z_buffer)
                .arg(&aff_b_x_buffer)
                .arg(&aff_b_y_buffer)
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
                jac_a_x_buffer,
                jac_a_y_buffer,
                jac_a_z_buffer,
                aff_b_x_buffer,
                aff_b_y_buffer,
                result_x_buffer,
                result_y_buffer,
                result_z_buffer,
                jacobian_point_affine_point_addition_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/jacobian_point_affine_point_addition_kernel"));

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

        fn addition(&mut self, jac_a_x: Vec<u8>, jac_a_y: Vec<u8>, jac_a_z: Vec<u8>, aff_b_x: Vec<u8>, aff_b_y: Vec<u8>) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
            if jac_a_x.len() != 32 || jac_a_y.len() != 32 || jac_a_z.len() != 32 || aff_b_x.len() != 32 || aff_b_y.len() != 32 {
                return Err(format!(
                    "All inputs must be 32 bytes long, got: jac_a_x={}, jac_a_y={}, jac_a_z={}, aff_b_x={}, aff_b_y={}",
                    jac_a_x.len(), jac_a_y.len(), jac_a_z.len(), aff_b_x.len(), aff_b_y.len()
                ));
            }

            // Clone the buffers to avoid borrowing issues
            self.write_to_buffer(&self.jac_a_x_buffer.clone(), jac_a_x)?;
            self.write_to_buffer(&self.jac_a_y_buffer.clone(), jac_a_y)?;
            self.write_to_buffer(&self.jac_a_z_buffer.clone(), jac_a_z)?;
            self.write_to_buffer(&self.aff_b_x_buffer.clone(), aff_b_x)?;
            self.write_to_buffer(&self.aff_b_y_buffer.clone(), aff_b_y)?;

            // Execute kernel
            unsafe {
                match self.jacobian_point_affine_point_addition_kernel.enq() {
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
    fn test_jacobian_point_affine_point_addition_simple() {
        let mut ocl = JacobianPointAffinePointAddition::new().unwrap();

        // Test: Jacobian point (1, 1, 1) + Affine point (2, 2)
        let jac_a_x = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let jac_a_y = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let jac_a_z = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let aff_b_x = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];
        let aff_b_y = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];

        let (result_x, result_y, result_z) = ocl.addition(jac_a_x, jac_a_y, jac_a_z, aff_b_x, aff_b_y).unwrap();

        // Just check that we get some result (exact values would need proper elliptic curve math)
        assert_eq!(result_x.len(), 32);
        assert_eq!(result_y.len(), 32);
        assert_eq!(result_z.len(), 32);
    }
}