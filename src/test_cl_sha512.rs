#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct OpenCLSha512 {
        message: Buffer<u8>,
        sha512_result: Buffer<u8>,
        sha512_kernel: Kernel,
    }

    impl OpenCLSha512 {
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
                .len(111)
                .build()
            {
                Ok(output) => output,
                Err(e) => {
                    return Err(
                        "Error creating OpenCL message buffer: ".to_string() + &e.to_string()
                    )
                }
            };

            let sha512_result = match Buffer::<u8>::builder().queue(queue.clone()).len(64).build() {
                Ok(output) => output,
                Err(e) => {
                    return Err(
                        "Error creating OpenCL sha256 result buffer: ".to_string() + &e.to_string()
                    )
                }
            };

            let sha512_kernel = match Kernel::builder()
                .program(&program)
                .name("sha512_kernel")
                .queue(queue.clone())
                .arg(&message)
                .arg(55u32)
                .arg(&sha512_result)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error building OpenCL kernel: ".to_string() + &e.to_string()),
            };

            Ok(OpenCLSha512 {
                message,
                sha512_result,
                sha512_kernel,
            })
        }

        pub fn sha512(&mut self, message: Vec<u8>) -> Result<Vec<u8>, String> {
            let message_len = message.len();
            if message_len > 111 {
                return Err("Message is too long".to_string());
            }

            match self.message.write(&message).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing message: ".to_string() + &e.to_string()),
            };

            match self.sha512_kernel.set_arg(1, message_len as u32) {
                Ok(_) => (),
                Err(e) => {
                    return Err(
                        "Error setting message length argument: ".to_string() + &e.to_string()
                    )
                }
            };

            unsafe {
                match self.sha512_kernel.enq() {
                    Ok(_) => (),
                    Err(e) => return Err("Error enqueuing kernel: ".to_string() + &e.to_string()),
                };
            }

            let mut result = vec![0; 64];
            match self.sha512_result.read(&mut result).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error reading sha512 result: ".to_string() + &e.to_string()),
            };
            Ok(result)
        }
    }

    #[test]
    fn test_sha512_abc() {
        let mut ocl = OpenCLSha512::new().unwrap();
        let message = "abc".as_bytes().to_vec();
        let result = ocl.sha512(message).unwrap();
        println!("result length: {}", result.len());

        assert_eq!(
            result,
            vec![
                0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba, 0xcc, 0x41, 0x73, 0x49, 0xae, 0x20,
                0x41, 0x31, 0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9, 0x7e, 0xa2, 0x0a, 0x9e, 0xee, 0xe6,
                0x4b, 0x55, 0xd3, 0x9a, 0x21, 0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8, 0x36, 0xba,
                0x3c, 0x23, 0xa3, 0xfe, 0xeb, 0xbd, 0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8, 0x0e,
                0x2a, 0x9a, 0xc9, 0x4f, 0xa5, 0x4c, 0xa4, 0x9f,
            ]
        );
    }

    #[test]
    fn test_sha512_abcde() {
        let mut ocl = OpenCLSha512::new().unwrap();
        let message = "abcde".as_bytes().to_vec();
        let result = ocl.sha512(message).unwrap();

        assert_eq!(
            result,
            vec![
                0x87, 0x8a, 0xe6, 0x5a, 0x92, 0xe8, 0x6c, 0xac, 0x01, 0x1a, 0x57, 0x0d, 0x4c, 0x30,
                0xa7, 0xea, 0xec, 0x44, 0x2b, 0x85, 0xce, 0x8e, 0xca, 0x0c, 0x29, 0x52, 0xb5, 0xe3,
                0xcc, 0x06, 0x28, 0xc2, 0xe7, 0x9d, 0x88, 0x9a, 0xd4, 0xd5, 0xc7, 0xc6, 0x26, 0x98,
                0x6d, 0x45, 0x2d, 0xd8, 0x63, 0x74, 0xb6, 0xff, 0xaa, 0x7c, 0xd8, 0xb6, 0x76, 0x65,
                0xbe, 0xf2, 0x28, 0x9a, 0x5c, 0x70, 0xb0, 0xa1,
            ]
        );
    }

    #[test]
    fn test_sha512_there_is_no_spoon() {
        let mut ocl = OpenCLSha512::new().unwrap();
        let message = "there is no spoon".as_bytes().to_vec();
        let result = ocl.sha512(message).unwrap();

        assert_eq!(
            result,
            vec![
                0x70, 0x7f, 0x56, 0x79, 0x14, 0x3c, 0x66, 0x0f, 0x34, 0x60, 0xc2, 0x83, 0x12, 0x4a,
                0x62, 0xc0, 0x0b, 0x31, 0x3a, 0x42, 0xf6, 0xe6, 0xf5, 0x35, 0x00, 0xc0, 0x17, 0xd6,
                0xfa, 0xe5, 0x0b, 0xaa, 0x78, 0xbb, 0x05, 0x6f, 0x82, 0x99, 0x1c, 0x39, 0x81, 0xd3,
                0x64, 0xba, 0x12, 0xd2, 0x60, 0x3d, 0x5b, 0xf1, 0x31, 0xbb, 0x20, 0x9e, 0x6d, 0xb7,
                0x44, 0x81, 0xfe, 0xa8, 0x9e, 0xea, 0xea, 0x1c
            ]
        );
    }

    #[test]
    fn test_big_random_string() {
        let mut ocl = OpenCLSha512::new().unwrap();
        let message = "adgfhaosjdfniaysdhfjasdfgih3876rghasg67d5s7d8gfuhasdfhu736478984u3uisafasdfasdf723ruysdfasdfsnbdf7665sdf34372dg"
            .as_bytes()
            .to_vec();

        let result = ocl.sha512(message).unwrap();
        assert_eq!(
            result,
            vec![
            0x58, 0xaf, 0xec, 0x1d, 0x9a, 0xef, 0x66, 0x6f, 0xbd, 0xa8, 0x4e, 0x11, 0xa2, 0xb5,
                0x87, 0x5a, 0x37, 0xc3, 0x4a, 0xb3, 0x4c, 0x47, 0xab, 0xd3, 0x71, 0x3f, 0x0c, 0x37,
                0x86, 0x04, 0xc0, 0x33, 0xf8, 0x8f, 0x50, 0x2a, 0xa4, 0x90, 0x0c, 0xd5, 0x11, 0xdd,
                0x14, 0xc6, 0x23, 0x39, 0x93, 0xbb, 0x11, 0x76, 0xac, 0x4d, 0xf8, 0x38, 0x55, 0x46,
                0xd9, 0x20, 0x63, 0x85, 0xc4, 0x0c, 0x51, 0x03
            ]
        );
    }

    #[test]
    fn test_sha512_too_long() {
        let mut ocl = OpenCLSha512::new().unwrap();
        // 56 bytes
        let message = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .as_bytes()
            .to_vec();
        let result = ocl.sha512(message);
        assert!(result.is_err());
    }
}
