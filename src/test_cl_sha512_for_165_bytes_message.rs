#[cfg(test)]
mod tests {
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};

    pub struct OpenCLSha512For165BytesMessage {
        message: Buffer<u8>,
        sha512_result: Buffer<u8>,
        sha512_kernel: Kernel,
    }

    impl OpenCLSha512For165BytesMessage {
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

            let src = include_str!(concat!(env!("OUT_DIR"), "/sha512For165BytesMessage"));

            let program = match Program::builder().src(src).devices(device).build(&context) {
                Ok(program) => program,
                Err(e) => {
                    return Err("Error building OpenCL program: ".to_string() + &e.to_string())
                }
            };

            // REAL CLASS PART
            let message = match Buffer::<u8>::builder()
                .queue(queue.clone())
                .len(165)
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
                .name("sha512For165BytesMessageKernel")
                .queue(queue.clone())
                .arg(&message)
                .arg(&sha512_result)
                .global_work_size(1)
                .build()
            {
                Ok(kernel) => kernel,
                Err(e) => return Err("Error building OpenCL kernel: ".to_string() + &e.to_string()),
            };

            Ok(OpenCLSha512For165BytesMessage {
                message,
                sha512_result,
                sha512_kernel,
            })
        }

        pub fn sha512(&mut self, message: Vec<u8>) -> Result<Vec<u8>, String> {
            let message_len = message.len();
            if message_len != 165 {
                return Err(format!("Message length must be 165. Got: {}", message_len));
            }

            match self.message.write(&message).enq() {
                Ok(_) => (),
                Err(e) => return Err("Error writing message: ".to_string() + &e.to_string()),
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
    fn test_satoshi_quote() {
        let mut ocl = OpenCLSha512For165BytesMessage::new().unwrap();
        let message = "                                    privacy can still be maintained by breaking the flow of information in another place: by keeping public keys anonymous.          ".as_bytes().to_vec();
        let result = ocl.sha512(message).unwrap();

        let expected_result = vec![
            157, 219, 208, 29, 22, 226, 125, 4, 139, 208, 101, 186, 139, 20, 225, 230, 100, 46,
            255, 198, 143, 219, 220, 32, 108, 243, 11, 85, 16, 197, 167, 241, 143, 44, 15, 43, 93,
            178, 12, 107, 204, 133, 1, 193, 125, 49, 88, 106, 94, 152, 241, 245, 110, 88, 155, 238,
            210, 168, 11, 184, 87, 142, 140, 94,
        ];
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_genesis_block_message() {
        let mut ocl = OpenCLSha512For165BytesMessage::new().unwrap();
        let message = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f - EThe Times 03/Jan/2009 Chancellor on brink of second bailout for banks............................".as_bytes().to_vec();
        let result = ocl.sha512(message).unwrap();

        let expected_result = vec![
            0x6c, 0x51, 0x37, 0xeb, 0xa8, 0x8a, 0x11, 0x41, 0x76, 0x7c, 0x84, 0x4a, 0xfc, 0x8a,
            0x98, 0xbd, 0x2a, 0x84, 0xeb, 0x5d, 0x88, 0xb0, 0xa8, 0xf4, 0x68, 0x1e, 0xab, 0x03,
            0xe7, 0xcc, 0x09, 0x9c, 0x9f, 0x9b, 0x77, 0x30, 0x96, 0x43, 0x7b, 0xd2, 0x05, 0x21,
            0x54, 0xbb, 0xbb, 0x3e, 0xeb, 0xde, 0xec, 0xe6, 0x29, 0x4a, 0x5e, 0x07, 0xbf, 0xaf,
            0xb5, 0x2f, 0x4b, 0x1f, 0xa2, 0x3f, 0x8e, 0x03,
        ];
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_all_ascii_a_message() {
        let mut ocl = OpenCLSha512For165BytesMessage::new().unwrap();
        let message = vec![b'a'; 165];
        let result = ocl.sha512(message).unwrap();

        let expected_result = vec![
            0xaa, 0x02, 0x90, 0x95, 0xfe, 0xc7, 0xe3, 0x5c, 0x5e, 0x4b, 0x60, 0x41, 0x83, 0x4f,
            0xf3, 0xd2, 0xec, 0x47, 0xd3, 0x63, 0x3c, 0xc1, 0xe8, 0x09, 0x58, 0xb7, 0xda, 0x35,
            0xf8, 0xb8, 0xea, 0x2e, 0x9c, 0x46, 0xcc, 0xbd, 0xc9, 0x3e, 0xe9, 0x3e, 0x20, 0x25,
            0xaa, 0x9a, 0xb7, 0xbd, 0xde, 0x99, 0xdc, 0x2d, 0xcd, 0xba, 0xdb, 0x10, 0xa3, 0x77,
            0x0c, 0x55, 0x69, 0x9e, 0xa7, 0xf5, 0x13, 0x40,
        ];
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_full_random_message() {
        let mut ocl = OpenCLSha512For165BytesMessage::new().unwrap();
        let message = vec![
            0xf3, 0x45, 0x68, 0x90, 0xa6, 0x5b, 0xd3, 0x43, 0x27, 0x89, 0x86, 0x54, 0x53, 0x43,
            0xd3, 0xe3, 0x57, 0x38, 0x93, 0x75, 0x36, 0x52, 0x43, 0x62, 0x32, 0xdd, 0xe2, 0xed,
            0x32, 0x63, 0x26, 0x34, 0x72, 0x35, 0x23, 0x78, 0x27, 0x39, 0x84, 0x56, 0x7e, 0xe5,
            0x67, 0x89, 0x63, 0x45, 0xde, 0xde, 0xae, 0xdb, 0xcd, 0xef, 0x52, 0x34, 0x56, 0x78,
            0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34,
            0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0xa6, 0x6f, 0xa3, 0x4f, 0x76, 0xfa,
            0xaf, 0x76, 0x78, 0x63, 0x25, 0x23, 0x45, 0x73, 0x42, 0x88, 0x73, 0x24, 0x76, 0x32,
            0x45, 0x43, 0x24, 0x97, 0x83, 0x24, 0x98, 0x98, 0x03, 0x40, 0x99, 0x38, 0x93, 0x76,
            0x35, 0x64, 0x42, 0x42, 0x32, 0x34, 0x26, 0x37, 0x34, 0x53, 0x27, 0x62, 0x38, 0x87,
            0x25, 0x62, 0x35, 0x4a, 0x67, 0xb7, 0x7c, 0x7d, 0x54, 0x6e, 0x8f, 0x34, 0x56, 0x03,
            0x86, 0x35, 0x24, 0x63, 0x89, 0x39, 0x87, 0x76, 0xd6, 0x5e, 0x67, 0xe8, 0xd9, 0x8d,
            0x76, 0x67, 0x88, 0x76, 0x88, 0xf8, 0x75, 0x44, 0x78, 0xd5, 0x58,
        ];

        let result = ocl.sha512(message).unwrap();

        assert_eq!(
            result,
            vec![
                0x82, 0x13, 0xde, 0x9d, 0x42, 0x30, 0x5f, 0x8d, 0x12, 0xac, 0xbe, 0x01, 0x47, 0x35,
                0xf0, 0x0c, 0x92, 0x47, 0xfd, 0xfd, 0x49, 0xbd, 0xa5, 0xef, 0x0a, 0x3a, 0x0d, 0x1c,
                0x62, 0x08, 0x44, 0x26, 0x17, 0xa8, 0x52, 0xc4, 0x07, 0x70, 0x8d, 0x0c, 0xad, 0x18,
                0x04, 0x91, 0x09, 0x0e, 0x36, 0xb7, 0x9c, 0x9f, 0xfc, 0xf2, 0x72, 0x60, 0x34, 0x85,
                0x55, 0xbd, 0x90, 0x02, 0x1c, 0x1c, 0xa6, 0xda,
            ]
        );
    }
}
