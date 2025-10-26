use address_artisan::opencl::benchmark::g_times_scalar_benchmark::GTimesScalarBenchmark;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 G Times Scalar Multiplication Benchmark");
    println!("==========================================");
    println!();

    // List available devices
    println!("📋 Dispositivos OpenCL disponíveis:");
    println!();

    let devices = match GTimesScalarBenchmark::list_devices() {
        Ok(devices) => devices,
        Err(e) => {
            eprintln!("❌ Erro ao listar dispositivos: {}", e);
            return Ok(());
        }
    };

    for (idx, (platform, device_name, _)) in devices.iter().enumerate() {
        println!("  [{}] {} - {}", idx, platform, device_name);
    }
    println!();

    // Get user selection
    print!("Escolha o dispositivo (0-{}): ", devices.len() - 1);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let device_idx: usize = match input.trim().parse() {
        Ok(idx) if idx < devices.len() => idx,
        _ => {
            eprintln!("❌ Seleção inválida!");
            return Ok(());
        }
    };

    let (platform, device_name, device_info) = devices.into_iter().nth(device_idx).unwrap();
    println!("✅ Selecionado: {} - {}", platform, device_name);
    println!();

    // Show device capabilities and get concurrent threads info
    let (_max_practical_threads, concurrent_threads) = match device_info.print_info_and_get_concurrent() {
        Ok((max, concurrent)) => (max, concurrent),
        Err(e) => {
            eprintln!("⚠️  Aviso: Não foi possível obter informações do dispositivo: {}", e);
            (1_048_576, 50_000) // Default fallback
        }
    };
    println!();

    // Get number of WAVES from user instead of absolute threads
    print!("Número de ondas de execução (N): 1 = {} threads, 2 = {} threads, etc.: ",
           concurrent_threads,
           ((concurrent_threads as f64 * 1.95) as i32 / 512) * 512);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let waves: f64 = match input.trim().parse() {
        Ok(n) if n > 0.0 => n,
        _ => {
            println!("❌ Valor inválido! Usando padrão: 2 ondas");
            2.0
        }
    };

    // Calculate optimal threads: (N - 0.05) × concurrent_threads, rounded DOWN to multiple of 512
    let exact_threads = (waves - 0.05) * (concurrent_threads as f64);
    let max_threads = ((exact_threads as i32) / 512) * 512; // Round DOWN to multiple of 512

    println!();
    println!("🎯 Threads calculadas automaticamente:");
    println!("   • Ondas solicitadas: {:.2}", waves);
    println!("   • Threads base por onda: {}", concurrent_threads);
    println!("   • Cálculo: ({:.2} - 0.05) × {} = {:.0}", waves, concurrent_threads, exact_threads);
    println!("   • Ajustado para múltiplo de 512: {} threads", max_threads);
    println!("   • Eficiência esperada da última onda: ~95%");
    println!();

    // Get test duration from user
    print!("Duração do teste em segundos (padrão: 10): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let test_duration: u64 = match input.trim().parse() {
        Ok(duration) if duration > 0 => duration,
        _ => {
            println!("Usando duração padrão: 10 segundos");
            10
        }
    };

    // Default test scalar (just some random value)
    let test_scalar = vec![
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11,
        0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
    ];

    println!();
    println!("Configuração:");
    println!("- Threads por kernel: {}", max_threads);
    println!("- Duração do teste: {} segundos", test_duration);
    println!();

    println!("Inicializando OpenCL...");
    let mut benchmark = match GTimesScalarBenchmark::new_with_device(max_threads, device_info) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("❌ Erro ao inicializar OpenCL: {}", e);
            return Ok(());
        }
    };

    println!("✅ OpenCL inicializado com sucesso!");
    println!();

    println!("🔄 Iniciando benchmark contínuo...");
    println!("Pressione Ctrl+C para parar");
    println!();

    let mut test_count = 1;

    loop {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🔄 Teste #{}", test_count);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        match benchmark.run_benchmark(test_scalar.clone(), max_threads, test_duration) {
            Ok(ops_per_second) => {
                println!("\n✅ Resultado:");
                println!("   • Throughput: {:.2} operações/segundo", ops_per_second);
            }
            Err(e) => {
                println!("\n❌ Erro: {}", e);
                break;
            }
        }

        test_count += 1;

        // Pequena pausa entre testes
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }

    Ok(())
}