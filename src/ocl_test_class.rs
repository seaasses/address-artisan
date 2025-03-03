use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue, SpatialDims};

pub struct OclTestClass {
    last_run_size: u32,
    offset: u64,
    count_kernel: Kernel,
    output: Buffer<u8>,
    found_flag: Buffer<u32>,
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

        let found_flag = match Buffer::<u32>::builder().queue(queue.clone()).len(1).build() {
            Ok(output) => output,
            Err(e) => {
                return Err("Error creating OpenCL found flag buffer: ".to_string() + &e.to_string())
            }
        };

        let output = match Buffer::<u8>::builder().queue(queue.clone()).len(32).build() {
            Ok(output) => output,
            Err(e) => {
                return Err("Error creating OpenCL output buffer: ".to_string() + &e.to_string())
            }
        };

        let count_kernel = match Kernel::builder()
            .program(&program)
            .name("count")
            .queue(queue.clone())
            .arg(0u32) // (workers_count) will be replaced
            .arg(0u64) // (offset) will be replaced - but start at 0
            .arg(&found_flag)
            .arg(&output)
            .build()
        {
            Ok(kernel) => kernel,
            Err(e) => return Err("Error building OpenCL kernel: ".to_string() + &e.to_string()),
        };

        Ok(OclTestClass {
            last_run_size: 0,
            offset: 0u64,
            count_kernel,
            output,
            found_flag,
        })
    }

    pub fn run(&mut self, quant: u32) -> Result<Option<FoundResult>, String> {
        if quant != self.last_run_size {
            self.count_kernel
                .set_default_global_work_size(SpatialDims::One(quant as usize));

            match self.count_kernel.set_arg(0, quant) {
                Ok(_) => (),
                Err(e) => return Err(format!("Error setting kernel arg: {:?}", e)),
            };
        }

        // SET ARGS
        // set workers_count

        // set offset
        match self.count_kernel.set_arg(1, self.offset) {
            Ok(_) => (),
            Err(e) => return Err(format!("Error setting kernel arg: {:?}", e)),
        };

        unsafe {
            match self.count_kernel.enq() {
                Ok(result) => result,
                Err(e) => return Err(format!("Error executing kernel: {:?}", e)),
            }
        }

        // get found flag
        let mut found_flag: Vec<u32> = vec![0];

        match self.found_flag.read(&mut found_flag).enq() {
            Ok(result) => result,
            Err(e) => return Err(format!("Error reading found flag: {:?}", e)),
        };

        self.offset += quant as u64;

        if found_flag[0] == 1 {
            // get output
            let mut output = vec![0; 32];

            match self.output.read(&mut output).enq() {
                Ok(result) => result,
                Err(e) => return Err(format!("Error reading output: {:?}", e)),
            };

            match self.found_flag.write(&vec![0u32]).enq() {
                Ok(result) => result,
                Err(e) => return Err(format!("Error writing found flag: {:?}", e)),
            };

            return Ok(Some(FoundResult {
                id: 3,
                hash: output,
            }));
        } else {
            return Ok(None);
        }
    }
}
