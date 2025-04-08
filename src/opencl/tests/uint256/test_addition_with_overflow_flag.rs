#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct Uint256AdditionWithOverflowFlag {
        a_buffer: Buffer<u8>,
        b_buffer: Buffer<u8>,
        overflow_flag_buffer: Buffer<u8>,
        result_buffer: Buffer<u8>,
        uint256_addition_with_overflow_flag_kernel: Kernel,
    }

    impl Uint256AdditionWithOverflowFlag {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
            let (device, context, queue) = Self::get_device_context_and_queue()?;

            // Create buffers
            let a_buffer = Self::new_buffer(&queue, 32)?;
            let b_buffer = Self::new_buffer(&queue, 32)?;
            let overflow_flag_buffer = Self::new_buffer(&queue, 1)?;

            let result_buffer = Self::new_buffer(&queue, 32)?;

            let program = Self::build_program(device, context)?;

            // Create kernel
            let uint256_addition_with_overflow_flag_kernel = match Kernel::builder()
                .program(&program)
                .name("uint256AdditionWithOverflowFlagKernel")
                .queue(queue.clone())
                .arg(&a_buffer)
                .arg(&b_buffer)
                .arg(&overflow_flag_buffer)
                .arg(&result_buffer)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
            };

            Ok(Self {
                a_buffer,
                b_buffer,
                overflow_flag_buffer,
                result_buffer,
                uint256_addition_with_overflow_flag_kernel,
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
                "/uint256AdditionWithOverflowFlagKernel"
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

        fn read_boolean_from_buffer(self: &mut Self, buffer: &Buffer<u8>) -> Result<bool, String> {
            let mut data = vec![0u8; 1];
            match buffer.read(&mut data[..]).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading from buffer: ".to_string() + &e.to_string()),
            };
            Ok(data[0] == 1)
        }

        fn addition_with_overflow_flag(
            &mut self,
            a: Vec<u8>,
            b: Vec<u8>,
        ) -> Result<(Vec<u8>, bool), String> {
            if a.len() != 32 || b.len() != 32 {
                return Err(format!(
                    "Input vectors must be 32 bytes long, got a: {}, b: {}",
                    a.len(),
                    b.len()
                ));
            }

            // Clone the buffer to avoid borrowing issues
            self.write_to_buffer(&self.a_buffer.clone(), a)?;
            self.write_to_buffer(&self.b_buffer.clone(), b)?;

            // Execute kernel
            unsafe {
                match self.uint256_addition_with_overflow_flag_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error executing kernel: ".to_string() + &e.to_string()),
                };
            }

            let result_array = self.read_from_buffer(&self.result_buffer.clone())?;
            let overflow_flag =
                self.read_boolean_from_buffer(&self.overflow_flag_buffer.clone())?;

            Ok((result_array, overflow_flag))
        }
    }

    #[test]
    fn test_addition_1_1() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02,
        ];
        let expected_overflow_flag = false;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }

    #[test]
    fn test_addition_carry_on_limb_3() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected_overflow_flag = false;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }

    #[test]
    fn test_addition_carry_on_limb_2() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected_overflow_flag = false;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }

    #[test]
    fn test_addition_carry_on_limb_1() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected_overflow_flag = false;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }

    #[test]
    fn test_addition_overflow() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let b = vec![
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected_overflow_flag = true;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }

    #[test]
    fn test_addition_all_bits_set_plus_1() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let b = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let expected_overflow_flag = true;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }

    #[test]
    fn test_addition_all_bits_set_plus_all_bits_set() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let b = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let expected = vec![
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFE,
        ];

        let expected_overflow_flag = true;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }

    #[test]
    fn test_addition_0_1_bits_pattern_plus_0_1_bits_pattern() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
        ];
        let b = vec![
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

        let expected_overflow_flag = false;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }

    #[test]
    fn test_addition_1_0_bits_pattern_plus_1_0_bits_pattern() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
        ];
        let b = vec![
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
        ];
        let expected = vec![
            0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
            0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
            0x55, 0x55, 0x55, 0x54,
        ];

        let expected_overflow_flag = true;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }

    #[test]
    fn test_addition_1_0_bits_pattern_plus_0_1_bits_pattern() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010, 0b10101010,
            0b10101010, 0b10101010, 0b10101010, 0b10101010,
        ];
        let b = vec![
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101, 0b01010101,
            0b01010101, 0b01010101, 0b01010101, 0b01010101,
        ];
        let expected = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff,
        ];

        let expected_overflow_flag = false;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }

    #[test]
    fn test_addition_big_numbers_that_overflow() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0xf1, 0x35, 0xb9, 0x59, 0x87, 0xfc, 0xb5, 0x40, 0x07, 0xe0, 0x3a, 0x04, 0xd6, 0x29,
            0x6d, 0x0f, 0xa3, 0x03, 0x00, 0xeb, 0xf3, 0x1a, 0x51, 0x64, 0x20, 0xfb, 0xbe, 0x18,
            0xb8, 0x85, 0x96, 0x8a,
        ];
        let b = vec![
            0x1a, 0x94, 0x8a, 0x63, 0xb8, 0x5b, 0x74, 0x4f, 0x40, 0x12, 0x6f, 0x00, 0x5c, 0xa4,
            0x80, 0x28, 0xba, 0xbc, 0x68, 0x4a, 0xf3, 0xd9, 0x74, 0x46, 0xb3, 0x95, 0x74, 0x3f,
            0x49, 0x8d, 0x46, 0x57,
        ];
        let expected = vec![
            0x0b, 0xca, 0x43, 0xbd, 0x40, 0x58, 0x29, 0x8f, 0x47, 0xf2, 0xa9, 0x05, 0x32, 0xcd,
            0xed, 0x38, 0x5d, 0xbf, 0x69, 0x36, 0xe6, 0xf3, 0xc5, 0xaa, 0xd4, 0x91, 0x32, 0x58,
            0x02, 0x12, 0xdc, 0xe1,
        ];

        let expected_overflow_flag = true;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }

    #[test]
    fn test_addition_big_numbers_that_do_not_overflow() {
        let mut ocl = Uint256AdditionWithOverflowFlag::new().unwrap();

        let a = vec![
            0x05, 0x38, 0x13, 0x53, 0x2e, 0xe4, 0x8e, 0x08, 0x00, 0xb6, 0x25, 0x4e, 0x5c, 0xfa,
            0xea, 0x21, 0x96, 0xec, 0x38, 0x55, 0x66, 0xde, 0x51, 0xf5, 0xe8, 0x68, 0x07, 0xd4,
            0xc9, 0x28, 0x07, 0xc1,
        ];
        let b = vec![
            0xed, 0xdc, 0x50, 0x71, 0xf0, 0x4c, 0x62, 0x4d, 0x99, 0x23, 0x3e, 0xa0, 0x30, 0xfb,
            0x6b, 0x97, 0xd3, 0x41, 0xc5, 0x86, 0x2c, 0xec, 0xcb, 0x47, 0xca, 0x0f, 0x67, 0xc7,
            0xec, 0x48, 0x6a, 0xae,
        ];
        let expected = vec![
            0xf3, 0x14, 0x63, 0xc5, 0x1f, 0x30, 0xf0, 0x55, 0x99, 0xd9, 0x63, 0xee, 0x8d, 0xf6,
            0x55, 0xb9, 0x6a, 0x2d, 0xfd, 0xdb, 0x93, 0xcb, 0x1d, 0x3d, 0xb2, 0x77, 0x6f, 0x9c,
            0xb5, 0x70, 0x72, 0x6f,
        ];

        let expected_overflow_flag = false;
        let (result, overflow_flag) = ocl.addition_with_overflow_flag(a, b).unwrap();

        assert_eq!(result, expected);
        assert_eq!(overflow_flag, expected_overflow_flag);
    }
}
