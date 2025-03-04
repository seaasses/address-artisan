use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue, SpatialDims};

pub struct OclTestClass {
    last_run_size: u32,
    offset_sha256: u64,
    offset_sha512: u64,
    sha256_kernel: Kernel,
    sha512_kernel: Kernel,
    output_sha256: Buffer<u8>,
    output_id_sha256: Buffer<u64>,
    output_sha512: Buffer<u8>,
    output_id_sha512: Buffer<u64>,
    found_flag_sha256: Buffer<u32>,
    found_flag_sha512: Buffer<u32>,
}

pub struct FoundResult {
    pub id: u64,
    pub hash: Vec<u8>,
}

impl OclTestClass {
    pub fn new() -> Result<Self, String> {
        let platform = match Platform::first() {
            Ok(platform) => platform,
            Err(e) => return Err("Error getting OpenCL platform: ".to_string() + &e.to_string()),
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
            Err(e) => return Err("Error building OpenCL context: ".to_string() + &e.to_string()),
        };

        let queue = Queue::new(&context, device, None)?;

        let src = include_str!(concat!(env!("OUT_DIR"), "/combined_kernels.cl"));

        let program = match Program::builder().src(src).devices(device).build(&context) {
            Ok(program) => program,
            Err(e) => return Err("Error building OpenCL program: ".to_string() + &e.to_string()),
        };

        let found_flag_sha256 = match Buffer::<u32>::builder().queue(queue.clone()).len(1).build() {
            Ok(output) => output,
            Err(e) => {
                return Err("Error creating OpenCL found flag buffer: ".to_string() + &e.to_string())
            }
        };

        let found_flag_sha512 = match Buffer::<u32>::builder().queue(queue.clone()).len(1).build() {
            Ok(output) => output,
            Err(e) => {
                return Err("Error creating OpenCL found flag buffer: ".to_string() + &e.to_string())
            }
        };

        let output_sha256 = match Buffer::<u8>::builder().queue(queue.clone()).len(32).build() {
            Ok(output) => output,
            Err(e) => {
                return Err("Error creating OpenCL output buffer: ".to_string() + &e.to_string())
            }
        };

        let output_sha512 = match Buffer::<u8>::builder().queue(queue.clone()).len(64).build() {
            Ok(output) => output,
            Err(e) => {
                return Err("Error creating OpenCL output buffer: ".to_string() + &e.to_string())
            }
        };

        let output_id_sha256 = match Buffer::<u64>::builder().queue(queue.clone()).len(1).build() {
            Ok(output_id) => output_id,
            Err(e) => {
                return Err("Error creating OpenCL output id buffer: ".to_string() + &e.to_string())
            }
        };

        let output_id_sha512 = match Buffer::<u64>::builder().queue(queue.clone()).len(1).build() {
            Ok(output_id) => output_id,
            Err(e) => {
                return Err("Error creating OpenCL output id buffer: ".to_string() + &e.to_string())
            }
        };

        let sha256_kernel = match Kernel::builder()
            .program(&program)
            .name("run_sha256")
            .queue(queue.clone())
            .arg(0u32) // (workers_count) will be replaced
            .arg(0u64) // (offset) will be replaced - but start at 0
            .arg(&found_flag_sha256)
            .arg(&output_sha256)
            .arg(&output_id_sha256)
            .build()
        {
            Ok(kernel) => kernel,
            Err(e) => return Err("Error building OpenCL kernel: ".to_string() + &e.to_string()),
        };

        let sha512_kernel = match Kernel::builder()
            .program(&program)
            .name("run_sha512")
            .queue(queue.clone())
            .arg(0u32)
            .arg(0u64)
            .arg(&found_flag_sha512)
            .arg(&output_sha512)
            .arg(&output_id_sha512)
            .build()
        {
            Ok(kernel) => kernel,
            Err(e) => return Err("Error building OpenCL kernel: ".to_string() + &e.to_string()),
        };

        Ok(OclTestClass {
            found_flag_sha256,
            found_flag_sha512,
            last_run_size: 0,
            offset_sha256: 0,
            offset_sha512: 0,
            sha256_kernel,
            sha512_kernel,
            output_sha256,
            output_id_sha256,
            output_sha512,
            output_id_sha512,
        })
    }
    
    pub fn run_sha512(&mut self, quant: u32) -> Result<Option<FoundResult>, String> {
        if quant != self.last_run_size {
            self.sha512_kernel
                .set_default_global_work_size(SpatialDims::One(quant as usize));

            match self.sha512_kernel.set_arg(0, quant) {
                Ok(_) => (),
                Err(e) => return Err(format!("Error setting kernel arg: {:?}", e)),
            };
        }

        // SET ARGS
        // set workers_count

        // set offset
        match self.sha512_kernel.set_arg(1, self.offset_sha512) {
            Ok(_) => (),
            Err(e) => return Err(format!("Error setting kernel arg: {:?}", e)),
        };

        unsafe {
            match self.sha512_kernel.enq() {
                Ok(result) => result,
                Err(e) => return Err(format!("Error executing kernel: {:?}", e)),
            }
        }

        // get found flag
        let mut found_flag: Vec<u32> = vec![0];

        match self.found_flag_sha512.read(&mut found_flag).enq() {
            Ok(result) => result,
            Err(e) => return Err(format!("Error reading found flag: {:?}", e)),
        };

        self.offset_sha512 += quant as u64;

        if found_flag[0] == 1 {
            // get output
            let mut output = vec![0; 64];
            let mut output_id = vec![0u64];

            match self.output_sha512.read(&mut output).enq() {
                Ok(result) => result,
                Err(e) => return Err(format!("Error reading output: {:?}", e)),
            };

            match self.output_id_sha512.read(&mut output_id).enq() {
                Ok(result) => result,
                Err(e) => return Err(format!("Error reading output id: {:?}", e)),
            };

            match self.found_flag_sha512.write(&vec![0u32]).enq() {
                Ok(result) => result,
                Err(e) => return Err(format!("Error writing found flag: {:?}", e)),
            };

            return Ok(Some(FoundResult {
                id: output_id[0],
                hash: output,
            }));
        } else {
            return Ok(None);
        }
    }


    pub fn run_sha256(&mut self, quant: u32) -> Result<Option<FoundResult>, String> {
        if quant != self.last_run_size {
            self.sha256_kernel
                .set_default_global_work_size(SpatialDims::One(quant as usize));

            match self.sha256_kernel.set_arg(0, quant) {
                Ok(_) => (),
                Err(e) => return Err(format!("Error setting kernel arg: {:?}", e)),
            };
        }

        // SET ARGS
        // set workers_count

        // set offset
        match self.sha256_kernel.set_arg(1, self.offset_sha256) {
            Ok(_) => (),
            Err(e) => return Err(format!("Error setting kernel arg: {:?}", e)),
        };

        unsafe {
            match self.sha256_kernel.enq() {
                Ok(result) => result,
                Err(e) => return Err(format!("Error executing kernel: {:?}", e)),
            }
        }

        // get found flag
        let mut found_flag: Vec<u32> = vec![0];

        match self.found_flag_sha256.read(&mut found_flag).enq() {
            Ok(result) => result,
            Err(e) => return Err(format!("Error reading found flag: {:?}", e)),
        };

        self.offset_sha256 += quant as u64;

        if found_flag[0] == 1 {
            // get output
            let mut output = vec![0; 32];
            let mut output_id = vec![0u64];

            match self.output_sha256.read(&mut output).enq() {
                Ok(result) => result,
                Err(e) => return Err(format!("Error reading output: {:?}", e)),
            };

            match self.output_id_sha256.read(&mut output_id).enq() {
                Ok(result) => result,
                Err(e) => return Err(format!("Error reading output id: {:?}", e)),
            };

            match self.found_flag_sha256.write(&vec![0u32]).enq() {
                Ok(result) => result,
                Err(e) => return Err(format!("Error writing found flag: {:?}", e)),
            };

            return Ok(Some(FoundResult {
                id: output_id[0],
                hash: output,
            }));
        } else {
            return Ok(None);
        }
    }
}
