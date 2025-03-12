#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct OpenCLSha256 {
        message: Buffer<u8>,
        sha256_result: Buffer<u8>,
        sha256_kernel: Kernel,
    }

    impl OpenCLSha256 {
        pub fn new() -> Result<Self, String> {
            // CREATE OPENCL CONTEXT
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

            let queue = Queue::new(&context, device, None)?;

            let src = include_str!(concat!(env!("OUT_DIR"), "/combined_kernels.cl"));

            let program = match Program::builder().src(src).devices(device).build(&context) {
                Ok(program) => program,
                Err(e) => {
                    return Err("Error building OpenCL program: ".to_string() + &e.to_string())
                }
            };

            // REAL CLASS PART
            let message = match Buffer::<u8>::builder()
                .queue(queue.clone())
                .len(55)
                .build()
            {
                Ok(output) => output,
                Err(e) => {
                    return Err(
                        "Error creating OpenCL message buffer: ".to_string() + &e.to_string()
                    )
                }
            };

            let sha256_result = match Buffer::<u8>::builder().queue(queue.clone()).len(32).build() {
                Ok(output) => output,
                Err(e) => {
                    return Err(
                        "Error creating OpenCL sha256 result buffer: ".to_string() + &e.to_string()
                    )
                }
            };

            let sha256_kernel = match Kernel::builder()
                .program(&program)
                .name("sha256_kernel")
                .queue(queue.clone())
                .arg(&message)
                .arg(55u32)
                .arg(&sha256_result)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error building OpenCL kernel: ".to_string() + &e.to_string()),
            };

            Ok(OpenCLSha256 {
                message,
                sha256_result,
                sha256_kernel,
            })
        }

        pub fn sha256(&mut self, message: Vec<u8>) -> Result<Vec<u8>, String> {
            let message_len = message.len();
            if message_len > 55 {
                return Err("Message is too long".to_string());
            }

            match self.message.write(&message).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing message: ".to_string() + &e.to_string()),
            };

            match self.sha256_kernel.set_arg(1, message_len as u32) {
                Ok(_) => (),
                Err(e) => {
                    return Err(
                        "Error setting message length argument: ".to_string() + &e.to_string()
                    )
                }
            };

            unsafe {
                match self.sha256_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error enqueuing kernel: ".to_string() + &e.to_string()),
                };
            }

            let mut result = vec![0; 32];
            match self.sha256_result.read(&mut result).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading sha256 result: ".to_string() + &e.to_string()),
            };
            Ok(result)
        }
    }

    #[test]
    fn test_sha256_abc() {
        let mut ocl = OpenCLSha256::new().unwrap();
        let message = "abc".as_bytes().to_vec();
        let result = ocl.sha256(message).unwrap();
        assert_eq!(
            result,
            vec![
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
                0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
                0xf2, 0x00, 0x15, 0xad,
            ]
        );
    }

    #[test]
    fn test_sha256_abcde() {
        let mut ocl = OpenCLSha256::new().unwrap();
        let message = "abcde".as_bytes().to_vec();
        let result = ocl.sha256(message).unwrap();

        assert_eq!(
            result,
            vec![
                0x36, 0xbb, 0xe5, 0x0e, 0xd9, 0x68, 0x41, 0xd1, 0x04, 0x43, 0xbc, 0xb6, 0x70, 0xd6,
                0x55, 0x4f, 0x0a, 0x34, 0xb7, 0x61, 0xbe, 0x67, 0xec, 0x9c, 0x4a, 0x8a, 0xd2, 0xc0,
                0xc4, 0x4c, 0xa4, 0x2c,
            ]
        );
    }

    #[test]
    fn test_sha256_there_is_no_spoon() {
        let mut ocl = OpenCLSha256::new().unwrap();
        let message = "there is no spoon".as_bytes().to_vec();
        let result = ocl.sha256(message).unwrap();

        assert_eq!(
            result,
            vec![
                0xc6, 0x04, 0x97, 0x43, 0x67, 0xca, 0xb8, 0xcb, 0x1a, 0xe0, 0xe8, 0x1e, 0xac, 0xc7,
                0xe9, 0xf8, 0xfa, 0xe1, 0xb1, 0x49, 0xf5, 0x4a, 0x52, 0xd8, 0x5e, 0xd3, 0x24, 0xf4,
                0x2c, 0x2c, 0x83, 0x35,
            ]
        );
    }

    #[test]
    fn test_big_random_string() {
        let mut ocl = OpenCLSha256::new().unwrap();
        let message = "bapkjasddflkjaskakjsdfkjhhjsdjasdfddfihasdiasdfdfsdfasd"
            .as_bytes()
            .to_vec();
        println!("message length: {}", message.len());
        let result = ocl.sha256(message).unwrap();
        for i in result.iter() {
            print!("{:02x}", i);
        }
        assert_eq!(
            result,
            vec![
                0x7f, 0x1c, 0x64, 0x6a, 0xeb, 0xae, 0xce, 0xdb, 0xe4, 0x08, 0x87, 0x3c, 0xbd, 0xc0,
                0xab, 0x50, 0x44, 0x63, 0x3c, 0x02, 0xf4, 0xd6, 0xd7, 0xb9, 0x96, 0x9d, 0xfe, 0x83,
                0xad, 0xfc, 0x1d, 0xed
            ]
        );
    }
}
