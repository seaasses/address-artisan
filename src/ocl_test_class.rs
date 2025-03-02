use ocl::{core, Buffer, Context, Device, Kernel, Platform, Program, Queue, SpatialDims};
pub struct OclTestClass {
    output_size: u32,
    count_kernel: Kernel,
    output: Buffer<u64>,
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

        let src = r#"
        __kernel void add(uint quant, __global ulong* output) {
        
            uint work_item_id = get_global_id(0);

            if (work_item_id >= quant) {
                return;
            } 

            output[work_item_id] = 1;
        }
        "#;

        let program = match Program::builder().src(src).devices(device).build(&context) {
            Ok(program) => program,
            Err(e) => return Err("Error building OpenCL program: ".to_string() + &e.to_string()),
        };

        let output = match Buffer::<u64>::builder().queue(queue.clone()).len(1).build() {
            Ok(output) => output,
            Err(e) => {
                return Err("Error creating OpenCL output buffer: ".to_string() + &e.to_string())
            }
        };

        let max_work_item_sizes = core::get_device_info(device, core::DeviceInfo::MaxWorkItemSizes);

        let count_kernel = match Kernel::builder()
            .program(&program)
            .name("add")
            .queue(queue.clone())
            .arg(0u32) // will be replaced
            .arg(&output)
            .build()
        {
            Ok(kernel) => kernel,
            Err(e) => return Err("Error building OpenCL kernel: ".to_string() + &e.to_string()),
        };

        Ok(OclTestClass {
            output_size: 0,
            count_kernel,
            output,
        })
    }

    fn resize_output(&mut self, new_size: u32) {
        if new_size <= self.output_size {
            self.output_size = new_size;
            return;
        }

        println!("Resizing output to {}", new_size);

        let queue = match self.count_kernel.default_queue() {
            Some(q) => q.clone(),
            None => panic!("No queue found"),
        };

        let new_output = match Buffer::<u64>::builder()
            .queue(queue)
            .len(new_size as usize)
            .fill_val(0u64)
            .build()
        {
            Ok(buffer) => buffer,
            Err(_) => panic!("Error creating new output buffer"),
        };

        let _ = self.count_kernel.set_arg(1, &new_output);
        self.output_size = new_size;

        self.output = new_output;
    }

    pub fn run(&mut self, quant: u32) {
        // run the kernel with the given quant
        self.resize_output(quant);
        let work_size = SpatialDims::One(quant as usize);

        self.count_kernel.set_default_global_work_size(work_size);
        let _ = self.count_kernel.set_arg(0, quant);

        unsafe {
            match self.count_kernel.enq() {
                Ok(result) => result,
                Err(e) => println!("Error executing kernel: {:?}", e),
            }
        }

        let mut output = vec![0; self.output_size as usize];
        let a = self.output.read(&mut output).enq().unwrap();
        println!("{:?}", output);
    }
}
