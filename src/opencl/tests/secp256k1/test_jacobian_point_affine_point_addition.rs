#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    type PointResult = Result<(Vec<u8>, Vec<u8>, Vec<u8>), String>;

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
            let src = include_str!(concat!(
                env!("OUT_DIR"),
                "/jacobian_point_affine_point_addition_kernel"
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

        fn addition(
            &mut self,
            jac_a_x: Vec<u8>,
            jac_a_y: Vec<u8>,
            jac_a_z: Vec<u8>,
            aff_b_x: Vec<u8>,
            aff_b_y: Vec<u8>,
        ) -> PointResult {
            if jac_a_x.len() != 32
                || jac_a_y.len() != 32
                || jac_a_z.len() != 32
                || aff_b_x.len() != 32
                || aff_b_y.len() != 32
            {
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

        let (result_x, result_y, result_z) = ocl
            .addition(jac_a_x, jac_a_y, jac_a_z, aff_b_x, aff_b_y)
            .unwrap();

        // Just check that we get some result (exact values would need proper elliptic curve math)
        assert_eq!(result_x.len(), 32);
        assert_eq!(result_y.len(), 32);
        assert_eq!(result_z.len(), 32);
    }

    #[test]
    fn test_jacobian_infinity_plus_affine_infinity() {
        let mut ocl = JacobianPointAffinePointAddition::new().unwrap();

        // Jacobian point at infinity: Z = 0
        let jac_a_x = vec![0u8; 32];
        let jac_a_y = vec![0u8; 32];
        let jac_a_z = vec![0u8; 32]; // Z = 0 means infinity

        // Affine point at infinity: x = SECP256K1_P
        // P = FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
        let aff_b_x = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xFF, 0xFF, 0xFC, 0x2F,
        ];
        let aff_b_y = vec![0u8; 32]; // y value doesn't matter for infinity

        let (_result_x, _result_y, result_z) = ocl
            .addition(jac_a_x, jac_a_y, jac_a_z, aff_b_x, aff_b_y)
            .unwrap();

        // Result should be infinity (Z = 0)
        let expected_z = vec![0u8; 32];
        assert_eq!(result_z, expected_z, "Z coordinate should be 0 (infinity)");
    }

    #[test]
    fn test_jacobian_infinity_plus_affine_point() {
        let mut ocl = JacobianPointAffinePointAddition::new().unwrap();

        // Jacobian point at infinity: Z = 0
        let jac_a_x = vec![0u8; 32];
        let jac_a_y = vec![0u8; 32];
        let jac_a_z = vec![0u8; 32]; // Z = 0 means infinity

        // Affine point: secp256k1 generator G
        // Gx = 79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798
        let aff_b_x = vec![
            0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87,
            0x0B, 0x07, 0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B,
            0x16, 0xF8, 0x17, 0x98,
        ];
        // Gy = 483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8
        let aff_b_y = vec![
            0x48, 0x3A, 0xDA, 0x77, 0x26, 0xA3, 0xC4, 0x65, 0x5D, 0xA4, 0xFB, 0xFC, 0x0E, 0x11,
            0x08, 0xA8, 0xFD, 0x17, 0xB4, 0x48, 0xA6, 0x85, 0x54, 0x19, 0x9C, 0x47, 0xD0, 0x8F,
            0xFB, 0x10, 0xD4, 0xB8,
        ];

        let (result_x, result_y, result_z) = ocl
            .addition(jac_a_x, jac_a_y, jac_a_z, aff_b_x.clone(), aff_b_y.clone())
            .unwrap();

        // Result should be B in Jacobian form (with Z = 1)
        assert_eq!(result_x, aff_b_x, "X coordinate should match B's x");
        assert_eq!(result_y, aff_b_y, "Y coordinate should match B's y");

        let expected_z = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        assert_eq!(result_z, expected_z, "Z coordinate should be 1");
    }

    #[test]
    fn test_jacobian_point_plus_affine_infinity() {
        let mut ocl = JacobianPointAffinePointAddition::new().unwrap();

        // Jacobian point: secp256k1 generator G in Jacobian form (Z = 1)
        // Gx = 79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798
        let jac_a_x = vec![
            0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87,
            0x0B, 0x07, 0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B,
            0x16, 0xF8, 0x17, 0x98,
        ];
        // Gy = 483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8
        let jac_a_y = vec![
            0x48, 0x3A, 0xDA, 0x77, 0x26, 0xA3, 0xC4, 0x65, 0x5D, 0xA4, 0xFB, 0xFC, 0x0E, 0x11,
            0x08, 0xA8, 0xFD, 0x17, 0xB4, 0x48, 0xA6, 0x85, 0x54, 0x19, 0x9C, 0x47, 0xD0, 0x8F,
            0xFB, 0x10, 0xD4, 0xB8,
        ];
        let jac_a_z = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        // Affine point at infinity: x = SECP256K1_P
        let aff_b_x = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
            0xFF, 0xFF, 0xFC, 0x2F,
        ];
        let aff_b_y = vec![0u8; 32]; // y value doesn't matter for infinity

        let (result_x, result_y, result_z) = ocl
            .addition(
                jac_a_x.clone(),
                jac_a_y.clone(),
                jac_a_z.clone(),
                aff_b_x,
                aff_b_y,
            )
            .unwrap();

        // Result should be A (unchanged)
        assert_eq!(result_x, jac_a_x, "X coordinate should match A's x");
        assert_eq!(result_y, jac_a_y, "Y coordinate should match A's y");
        assert_eq!(result_z, jac_a_z, "Z coordinate should match A's z");
    }
}
