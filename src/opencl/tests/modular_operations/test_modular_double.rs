#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct ModularDouble {
        x_buffer: Buffer<u8>,
        result_buffer: Buffer<u8>,
        modular_double_kernel: Kernel,
    }

    impl ModularDouble {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let x_buffer = Self::new_buffer(&queue, 32)?;

            let result_buffer = Self::new_buffer(&queue, 32)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let modular_double_kernel = match Kernel::builder()
                .program(&program)
                .name("modularDoubleKernel")
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
                modular_double_kernel,
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
            let src = include_str!(concat!(env!("OUT_DIR"), "/modularDoubleKernel"));

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

        fn modular_shift_left(&mut self, x: Vec<u8>) -> Result<Vec<u8>, String> {
            if x.len() != 32 {
                return Err(format!(
                    "Input vectors must be 32 bytes long, got a: {}",
                    x.len()
                ));
            }

            // Clone the buffer to avoid borrowing issues
            self.write_to_buffer(&self.x_buffer.clone(), x)?;

            // Execute kernel
            unsafe {
                match self.modular_double_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            // Clone the buffer to avoid borrowing issues
            let result_array = self.read_from_buffer(&self.result_buffer.clone())?;

            Ok(result_array)
        }
    }

    const SECP256K1_P_MINUS_1: [u8; 32] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF,
        0xFC, 0x2E,
    ];

    #[test]
    fn test_shift_left_0() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![0; 32];
        let expected = vec![0; 32];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }
    #[test]
    fn test_shift_left_1() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_between_limbs_3_2() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_between_limbs_2_1() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_between_limbs_1_0() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_just_most_significant_bit_set() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x03, 0xd1,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_p_minus_1() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = SECP256K1_P_MINUS_1.to_vec();

        let expected = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0xff, 0xff, 0xfc, 0x2d,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_less_than_max_256_bits_after_shift_left() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0x7f, 0xff, 0xfe, 0x49,
        ];

        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x63,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_0_1_bits_pattern() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
        ];

        let expected = vec![
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
            0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa,
            0xaa, 0xaa, 0xaa, 0xaa,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_1_0_bits_pattern() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
        ];

        let expected = vec![
            0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
            0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x56,
            0x55, 0x55, 0x59, 0x25,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_0_ff_bytes_pattern() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
            0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
            0x00, 0xFF, 0x00, 0xFF,
        ];

        let expected = vec![
            0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE,
            0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE, 0x01, 0xFE,
            0x01, 0xFE, 0x01, 0xFE,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_ff_00_bytes_pattern() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00,
            0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00,
            0xFF, 0x00, 0xFF, 0x00,
        ];

        let expected = vec![
            0xfe, 0x01, 0xfe, 0x01, 0xfe, 0x01, 0xfe, 0x01, 0xfe, 0x01, 0xfe, 0x01, 0xfe, 0x01,
            0xfe, 0x01, 0xfe, 0x01, 0xfe, 0x01, 0xfe, 0x01, 0xfe, 0x01, 0xfe, 0x01, 0xfe, 0x02,
            0xfe, 0x02, 0x01, 0xd1,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_ffffffffffffffff_0000000000000000_limbs_pattern() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x03, 0xd1,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_0000000000000000_ffffffffffffffff_limbs_pattern() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFE,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }
    #[test]
    fn test_shift_left_bit_inside_byte() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0b00100000, 0b00010000, 0b01000000, 0b00001000, 0b00001000, 0b00010000, 0b01010000,
            0b00001000, 0b00000010, 0b00000000, 0b01000000, 0b01000100, 0b00010000, 0b00010000,
            0b01000000, 0b00000010, 0b00000001, 0b00101000, 0b00011000, 0b01000000, 0b00000100,
            0b00000000, 0b00100000, 0b00010000, 0b00000010, 0b00100000, 0b01001000, 0b01000000,
            0b00000001, 0b00000000, 0b01000100, 0b00001000,
        ];

        let expected = vec![
            0b01000000, 0b00100000, 0b10000000, 0b00010000, 0b00010000, 0b00100000, 0b10100000,
            0b00010000, 0b00000100, 0b00000000, 0b10000000, 0b10001000, 0b00100000, 0b00100000,
            0b10000000, 0b00000100, 0b00000010, 0b01010000, 0b00110000, 0b10000000, 0b00001000,
            0b00000000, 0b01000000, 0b00100000, 0b00000100, 0b01000000, 0b10010000, 0b10000000,
            0b00000010, 0b00000000, 0b10001000, 0b00010000,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }

    #[test]
    fn test_shift_left_big_number() {
        let mut ocl = ModularDouble::new().unwrap();

        let x = vec![
            0xd5, 0xa4, 0x9b, 0xa6, 0x04, 0x46, 0x90, 0x2d, 0x3b, 0x97, 0xe1, 0x75, 0x5e, 0xe5,
            0x7b, 0x3b, 0x80, 0xbc, 0xb6, 0x0b, 0xeb, 0x05, 0xa7, 0xf8, 0x9f, 0xdf, 0x96, 0xdd,
            0xcf, 0xa9, 0x85, 0x2d,
        ];

        let expected = vec![
            0xab, 0x49, 0x37, 0x4c, 0x08, 0x8d, 0x20, 0x5a, 0x77, 0x2f, 0xc2, 0xea, 0xbd, 0xca,
            0xf6, 0x77, 0x01, 0x79, 0x6c, 0x17, 0xd6, 0x0b, 0x4f, 0xf1, 0x3f, 0xbf, 0x2d, 0xbc,
            0x9f, 0x53, 0x0e, 0x2b,
        ];

        assert_eq!(ocl.modular_shift_left(x).unwrap(), expected);
    }
}
