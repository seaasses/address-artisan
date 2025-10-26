use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};
use std::time::Instant;

pub struct DeviceInfo {
    pub platform: Platform,
    pub device: Device,
}

impl DeviceInfo {
    pub fn print_info_and_get_concurrent(&self) -> Result<(i32, i32), String> {
        let compute_units = self.device.info(ocl::enums::DeviceInfo::MaxComputeUnits)
            .map_err(|e| e.to_string())?
            .to_string()
            .parse::<u32>()
            .unwrap_or(128);

        let max_work_group_size = self.device.info(ocl::enums::DeviceInfo::MaxWorkGroupSize)
            .map_err(|e| e.to_string())?
            .to_string()
            .parse::<usize>()
            .unwrap_or(256);

        let global_mem_size = self.device.info(ocl::enums::DeviceInfo::GlobalMemSize)
            .map_err(|e| e.to_string())?
            .to_string()
            .parse::<u64>()
            .unwrap_or(8_589_934_592);

        let max_work_item_dimensions = self.device.info(ocl::enums::DeviceInfo::MaxWorkItemDimensions)
            .map_err(|e| e.to_string())?
            .to_string()
            .parse::<u32>()
            .unwrap_or(3);

        // Get max work item sizes for each dimension
        let max_work_item_sizes = self.device.info(ocl::enums::DeviceInfo::MaxWorkItemSizes)
            .map_err(|e| e.to_string())?
            .to_string();

        let max_mem_alloc_size = self.device.info(ocl::enums::DeviceInfo::MaxMemAllocSize)
            .map_err(|e| e.to_string())?
            .to_string()
            .parse::<u64>()
            .unwrap_or(2_147_483_648);

        // Calculate theoretical occupancy
        // Each compute unit can run multiple work groups simultaneously
        // Typical values: 7-8 work groups per CU on Intel GPUs
        let estimated_wgs_per_cu = 7; // Conservative estimate
        let concurrent_work_groups = compute_units as usize * estimated_wgs_per_cu;
        let concurrent_threads = concurrent_work_groups * 512; // Using our work group size of 512

        println!("╔════════════════════════════════════════════════════════╗");
        println!("║          Informações do Dispositivo OpenCL             ║");
        println!("╠════════════════════════════════════════════════════════╣");
        println!("  Compute Units (CUs): {}", compute_units);
        println!("  Max Work Group Size: {} threads", max_work_group_size);
        println!("  Max Work Item Dimensions: {}", max_work_item_dimensions);
        println!("  Max Work Item Sizes: {}", max_work_item_sizes);
        println!("  Global Memory: {} MB", global_mem_size / (1024 * 1024));
        println!("  Max Memory Alloc Size: {} MB", max_mem_alloc_size / (1024 * 1024));
        println!("╠════════════════════════════════════════════════════════╣");
        println!("║  Configuração Atual do Kernel                          ║");
        println!("╠════════════════════════════════════════════════════════╣");
        println!("  • Usando 1 DIMENSÃO (1D kernel)");
        println!("  • Work group size: 512 threads");
        println!("  • Work groups por kernel: global_work_size / 512");
        println!("╠════════════════════════════════════════════════════════╣");
        println!("║  🔥 PARALELISMO REAL (estimado)                        ║");
        println!("╠════════════════════════════════════════════════════════╣");
        println!("  • Work groups simultâneos: ~{} WGs", concurrent_work_groups);
        println!("    ({} CUs × ~{} WGs/CU)", compute_units, estimated_wgs_per_cu);
        println!("  • Threads REALMENTE em paralelo: ~{} threads", concurrent_threads);
        println!("╠════════════════════════════════════════════════════════╣");
        println!("  💡 Threads além de {} são enfileiradas e executadas", concurrent_threads);
        println!("     depois que as primeiras terminam. Isso NÃO torna");
        println!("     a GPU mais rápida, apenas mantém ela ocupada!");
        println!("╚════════════════════════════════════════════════════════╝");
        println!();

        // Calculate practical maximum threads
        // Conservative estimate: compute_units * max_work_group_size * 64 (waves)
        let suggested_max = (compute_units as usize * max_work_group_size * 64) as i32;

        // Round to multiple of 512
        let suggested_max = ((suggested_max + 511) / 512) * 512;

        // But also consider memory limits (each thread uses some memory)
        // Assuming ~1KB per thread for safety
        let mem_limited_max = (max_mem_alloc_size / 1024) as i32;

        let practical_max = std::cmp::min(suggested_max, mem_limited_max);
        let practical_max = std::cmp::min(practical_max, 16_777_216); // OpenCL limit: 2^24

        println!("💡 Recomendações de valores:");
        println!("   📊 Mínimo para saturar GPU: {} threads", concurrent_threads);
        println!("      (Garante que todos os {} CUs estejam ocupados)", compute_units);
        println!();
        println!("   🎯 Recomendado (2-4x o mínimo): {} - {} threads",
                 concurrent_threads * 2, concurrent_threads * 4);
        println!("      (Compensa variações de tempo e mantém GPU 100% ocupada)");
        println!();
        println!("   ⚠️  Muito além de {} threads não melhora performance!", concurrent_threads * 8);
        println!("      (Apenas aumenta latência sem ganho de throughput)");
        println!();
        println!("   🔢 Máximo técnico: {} threads", practical_max);
        println!();

        Ok((practical_max, concurrent_threads as i32))
    }
}

pub struct GTimesScalarBenchmark {
    scalar_buffer: Buffer<u8>,
    max_threads_buffer: Buffer<i32>,
    output_buffer: Buffer<i32>,
    iteration_offset_buffer: Buffer<u64>,
    kernel: Kernel,
    queue: Queue,
    program: Program,
}

impl GTimesScalarBenchmark {
    pub fn new(max_threads: i32) -> Result<Self, String> {
        let (device, context, queue) = Self::get_device_context_and_queue(None)?;

        // Create buffers
        let scalar_buffer = Self::new_buffer::<u8>(&queue, 32)?; // Uint256 = 32 bytes
        let max_threads_buffer = Self::new_buffer::<i32>(&queue, 1)?; // single int
        let output_buffer = Self::new_buffer::<i32>(&queue, max_threads as usize)?; // array para cada thread escrever
        let iteration_offset_buffer = Self::new_buffer::<u64>(&queue, 1)?; // iteration counter

        let program = Self::build_program(device, context)?;

        // Create kernel with 1D configuration
        //
        // COMO FUNCIONA:
        // ===============
        // • global_work_size = TOTAL de threads que vamos executar
        //   - Se passar 65536, serão 65536 threads executando em paralelo
        //
        // • local_work_size = Tamanho do WORK GROUP (512 threads por grupo)
        //   - Threads dentro de um work group compartilham memória local
        //   - Work groups são distribuídos entre os Compute Units da GPU
        //
        // • Número de work groups = global_work_size / local_work_size
        //   - Exemplo: 65536 / 512 = 128 work groups
        //
        // • Dimensões: Usando apenas 1D (uma dimensão)
        //   - Poderia usar 2D: .global_work_size([1024, 1024]) = 1M threads
        //   - Poderia usar 3D: .global_work_size([256, 256, 16]) = 1M threads
        //   - Mas 1D é mais simples para este caso!
        //
        let kernel = match Kernel::builder()
            .program(&program)
            .name("g_times_scalar_compute_kernel")
            .queue(queue.clone())
            .arg(&scalar_buffer)
            .arg(&max_threads_buffer)
            .arg(&output_buffer)
            .arg(&iteration_offset_buffer)
            .global_work_size(max_threads as usize)  // TOTAL de threads (1D)
            .local_work_size(512)                     // Work group size (512 threads/grupo)
            .build()
        {
            Ok(kernel) => kernel,
            Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
        };

        Ok(Self {
            scalar_buffer,
            max_threads_buffer,
            output_buffer,
            iteration_offset_buffer,
            kernel,
            queue,
            program,
        })
    }

    fn new_buffer<T: ocl::OclPrm>(queue: &Queue, len: usize) -> Result<Buffer<T>, String> {
        let buffer = match Buffer::<T>::builder()
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
        let src = include_str!(concat!(env!("OUT_DIR"), "/g_times_scalar_compute_kernel"));

        let program = match Program::builder().src(src).devices(device).build(&context) {
            Ok(program) => program,
            Err(e) => {
                return Err("Error building OpenCL program: ".to_string() + &e.to_string())
            }
        };

        Ok(program)
    }

    pub fn new_with_device(max_threads: i32, device_info: DeviceInfo) -> Result<Self, String> {
        let (device, context, queue) = Self::get_device_context_and_queue(Some(device_info))?;

        // Create buffers
        let scalar_buffer = Self::new_buffer::<u8>(&queue, 32)?; // Uint256 = 32 bytes
        let max_threads_buffer = Self::new_buffer::<i32>(&queue, 1)?; // single int
        let output_buffer = Self::new_buffer::<i32>(&queue, max_threads as usize)?; // array para cada thread escrever
        let iteration_offset_buffer = Self::new_buffer::<u64>(&queue, 1)?; // iteration counter

        let program = Self::build_program(device, context)?;

        // Create kernel with 1D configuration
        //
        // COMO FUNCIONA:
        // ===============
        // • global_work_size = TOTAL de threads que vamos executar
        //   - Se passar 65536, serão 65536 threads executando em paralelo
        //
        // • local_work_size = Tamanho do WORK GROUP (512 threads por grupo)
        //   - Threads dentro de um work group compartilham memória local
        //   - Work groups são distribuídos entre os Compute Units da GPU
        //
        // • Número de work groups = global_work_size / local_work_size
        //   - Exemplo: 65536 / 512 = 128 work groups
        //
        // • Dimensões: Usando apenas 1D (uma dimensão)
        //   - Poderia usar 2D: .global_work_size([1024, 1024]) = 1M threads
        //   - Poderia usar 3D: .global_work_size([256, 256, 16]) = 1M threads
        //   - Mas 1D é mais simples para este caso!
        //
        let kernel = match Kernel::builder()
            .program(&program)
            .name("g_times_scalar_compute_kernel")
            .queue(queue.clone())
            .arg(&scalar_buffer)
            .arg(&max_threads_buffer)
            .arg(&output_buffer)
            .arg(&iteration_offset_buffer)
            .global_work_size(max_threads as usize)  // TOTAL de threads (1D)
            .local_work_size(512)                     // Work group size (512 threads/grupo)
            .build()
        {
            Ok(kernel) => kernel,
            Err(e) => return Err("Error creating kernel: ".to_string() + &e.to_string()),
        };

        Ok(Self {
            scalar_buffer,
            max_threads_buffer,
            output_buffer,
            iteration_offset_buffer,
            kernel,
            queue,
            program,
        })
    }

    pub fn list_devices() -> Result<Vec<(String, String, DeviceInfo)>, String> {
        let mut devices_info = Vec::new();

        let platforms = Platform::list();

        for platform in platforms {
            let platform_name = match platform.name() {
                Ok(name) => name,
                Err(_) => "Unknown Platform".to_string(),
            };

            let devices = match Device::list_all(platform) {
                Ok(devices) => devices,
                Err(_) => continue,
            };

            for device in devices {
                let device_name = match device.name() {
                    Ok(name) => name,
                    Err(_) => "Unknown Device".to_string(),
                };

                devices_info.push((
                    platform_name.clone(),
                    device_name,
                    DeviceInfo {
                        platform,
                        device,
                    },
                ));
            }
        }

        if devices_info.is_empty() {
            return Err("No OpenCL devices found".to_string());
        }

        Ok(devices_info)
    }

    fn get_device_context_and_queue(device_info: Option<DeviceInfo>) -> Result<(Device, Context, Queue), String> {
        let (platform, device) = match device_info {
            Some(info) => (info.platform, info.device),
            None => {
                // Use first platform and first device
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
                (platform, device)
            }
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

    pub fn run_benchmark(&mut self, scalar: Vec<u8>, max_threads: i32, duration_secs: u64) -> Result<f64, String> {
        if scalar.len() != 32 {
            return Err(format!(
                "Scalar must be 32 bytes long, got: {}",
                scalar.len()
            ));
        }

        // Write data to buffers ONCE before the loop (dados não mudam!)
        println!("  📤 Transferindo dados para GPU...");
        let cpu_setup_start = Instant::now();

        self.scalar_buffer.write(&scalar).enq().map_err(|e| e.to_string())?;
        self.max_threads_buffer.write(&vec![max_threads]).enq().map_err(|e| e.to_string())?;

        // Inicializa buffer de output (array de max_threads elementos)
        let output_init = vec![0i32; max_threads as usize];
        self.output_buffer.write(&output_init).enq().map_err(|e| e.to_string())?;

        self.iteration_offset_buffer.write(&vec![0u64]).enq().map_err(|e| e.to_string())?;

        // Wait for transfers to complete
        self.queue.finish().map_err(|e| e.to_string())?;

        let cpu_setup_time = cpu_setup_start.elapsed().as_secs_f64();
        println!("  ✅ Setup completo em {:.4}s", cpu_setup_time);
        println!();
        println!("  ⏱️  Executando kernels continuamente por {} segundos...", duration_secs);
        println!("  💡 A GPU deve ficar em ~100% o tempo todo agora!");
        println!();

        let test_start = Instant::now();
        let mut kernel_count = 0u64;
        let mut total_gpu_time = 0.0f64;
        let mut last_print = Instant::now();
        let mut iteration_offset = 0u64;

        // Loop: lança kernel, espera, repete
        // A cada iteração, incrementamos o offset para forçar cálculos diferentes!
        while test_start.elapsed().as_secs() < duration_secs {
            // Atualiza o offset para este kernel (faz cada execução calcular valores diferentes)
            self.iteration_offset_buffer.write(&vec![iteration_offset]).enq().map_err(|e| e.to_string())?;

            let gpu_start = Instant::now();

            // Lança o kernel
            let enqueue_result = unsafe {
                self.kernel.enq()
            };

            if let Err(e) = enqueue_result {
                eprintln!("  ❌ ERRO ao enfileirar kernel #{}: {}", kernel_count, e);
                return Err(format!("Erro ao enfileirar kernel: {}", e));
            }

            // Espera GPU completar
            let finish_result = self.queue.finish();
            if let Err(e) = finish_result {
                eprintln!("  ❌ ERRO ao esperar kernel #{} completar: {}", kernel_count, e);
                eprintln!("  💡 Isso pode ser TIMEOUT do driver (TDR)!");
                eprintln!("  💡 Tente reduzir o número de ondas (N) ou threads.");
                return Err(format!("Erro ao esperar kernel: {} (possível TDR timeout)", e));
            }

            let gpu_exec_time = gpu_start.elapsed().as_secs_f64();

            // Avisa se kernel demorou muito (pode dar TDR no próximo)
            if gpu_exec_time > 5.0 {
                println!("  ⚠️  Kernel #{} demorou {:.2}s - RISCO DE TDR TIMEOUT!", kernel_count, gpu_exec_time);
            }

            total_gpu_time += gpu_exec_time;
            kernel_count += 1;
            iteration_offset += 1; // Próximo kernel terá offset diferente

            // Print progress every 5 seconds
            if last_print.elapsed().as_secs() >= 5 {
                let elapsed = test_start.elapsed().as_secs();
                println!("  ⏱️  Progresso: {}s / {}s ({} kernels, tempo médio: {:.2}s/kernel)",
                         elapsed, duration_secs, kernel_count, total_gpu_time / kernel_count as f64);
                last_print = Instant::now();
            }
        }

        println!("  ✅ Teste completo!");
        println!();

        let total_elapsed = test_start.elapsed();

        let total_operations = kernel_count * (max_threads as u64);
        let ops_per_second = total_operations as f64 / total_elapsed.as_secs_f64();

        // Calculate how many "waves" of execution happened
        // Get compute units info to estimate concurrent threads
        let compute_units = self.queue.device().info(ocl::enums::DeviceInfo::MaxComputeUnits)
            .map_err(|e| e.to_string())?
            .to_string()
            .parse::<u32>()
            .unwrap_or(128);

        let estimated_concurrent_threads = (compute_units as i32) * 512 * 7; // 7 WGs per CU estimate
        let waves_per_kernel = ((max_threads as f64) / (estimated_concurrent_threads as f64)).ceil() as u64;

        let avg_gpu_time = total_gpu_time / (kernel_count as f64);
        let gpu_percentage = (total_gpu_time / total_elapsed.as_secs_f64()) * 100.0;
        let idle_time = total_elapsed.as_secs_f64() - total_gpu_time;
        let idle_percentage = (idle_time / total_elapsed.as_secs_f64()) * 100.0;

        println!("  ℹ️  Estatísticas:");
        println!("     - Kernels executados: {}", kernel_count);
        println!("     - Threads por kernel: {}", max_threads);
        println!("     - Total de operações: {}", total_operations);
        println!();
        println!("  ⏱️  Tempos:");
        println!("     ╔══════════════════════════════════════════╗");
        println!("     ║ Tempo TOTAL:     {:<10.4}s (100.0%) ║", total_elapsed.as_secs_f64());
        println!("     ╠══════════════════════════════════════════╣");
        println!("     ║ Tempo GPU:       {:<10.4}s ({:>5.1}%) ║", total_gpu_time, gpu_percentage);
        println!("     ║ Tempo ocioso:    {:<10.4}s ({:>5.1}%) ║", idle_time, idle_percentage);
        println!("     ╚══════════════════════════════════════════╝");
        println!("     Setup inicial: {:.4}s (fora do loop)", cpu_setup_time);
        println!();
        println!("  📊 Por kernel:");
        println!("     - Tempo médio de execução: {:.4}s ({:.2} ms)", avg_gpu_time, avg_gpu_time * 1000.0);

        if gpu_percentage > 95.0 {
            println!();
            println!("     ✅ EXCELENTE! GPU em {:.1}% de uso!", gpu_percentage);
            println!("     💯 Kernels executando back-to-back sem idle time!");
        } else if gpu_percentage > 85.0 {
            println!();
            println!("     👍 BOM! GPU em {:.1}% de uso", gpu_percentage);
        } else {
            println!();
            println!("     ⚠️  GPU ocioso {:.1}% do tempo!", idle_percentage);
            println!("     💡 Pode haver overhead de sincronização.");
        }

        println!();
        // Calculate last wave efficiency
        let threads_in_last_wave = if waves_per_kernel > 1 {
            max_threads % estimated_concurrent_threads
        } else {
            max_threads
        };

        let last_wave_efficiency = if waves_per_kernel > 1 {
            (threads_in_last_wave as f64 / estimated_concurrent_threads as f64) * 100.0
        } else {
            100.0
        };

        println!("  📊 Análise de eficiência:");
        println!("     - Threads em paralelo (estimado): ~{}", estimated_concurrent_threads);
        println!("     - Ondas de execução por kernel: {}", waves_per_kernel);

        if waves_per_kernel > 1 {
            println!("     - Threads na última onda: {} / {}", threads_in_last_wave, estimated_concurrent_threads);
            println!("     - Eficiência da última onda: {:.1}%", last_wave_efficiency);
            println!();

            if last_wave_efficiency < 50.0 {
                println!("     ❌ RUIM! Última onda com {:.1}% de uso", last_wave_efficiency);
                println!("     💡 Sugestões de valores eficientes:");

                // Suggest values with good last wave efficiency (>95%)
                let wave_base = estimated_concurrent_threads;
                println!("        • {} threads (1 onda, 100% eficiência)", wave_base);
                println!("        • {} threads (2 ondas, ~95% na última)", (wave_base as f64 * 1.95) as i32);
                println!("        • {} threads (3 ondas, ~95% na última)", (wave_base as f64 * 2.95) as i32);
                println!("        • {} threads (4 ondas, ~95% na última)", (wave_base as f64 * 3.95) as i32);
            } else if last_wave_efficiency < 80.0 {
                println!("     ⚠️  Moderado. Última onda com {:.1}% de uso", last_wave_efficiency);
                println!("     💡 Para melhorar, use:");
                let wave_base = estimated_concurrent_threads;
                println!("        • {} threads ({} ondas, ~95% na última)",
                         (wave_base as f64 * (waves_per_kernel as f64 - 0.05)) as i32,
                         waves_per_kernel);
            } else {
                println!("     ✅ ÓTIMO! Última onda bem utilizada ({:.1}%)", last_wave_efficiency);
            }
        } else {
            println!("     ✅ PERFEITO! Tudo executa em 1 onda, GPU em 1 passada");
        }

        Ok(ops_per_second)
    }
}