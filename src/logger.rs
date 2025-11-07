use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Logger
    }

    pub fn start(&self, prefix: &str, max_depth: u32, cpu_threads: u32) {
        println!("Starting address search");
        println!("  Prefix: {}", prefix);
        println!("  Max depth: {}", max_depth);
        println!("  CPU threads: {}", cpu_threads);
    }

    pub fn workbench_started(&self, bench_id: &str) {
        println!("✓ {} workbench started", bench_id);
    }

    pub fn log_status(&self, bench_stats: &HashMap<String, BenchStats>) {
        println!("\n--- Status ---");

        let mut total_generated = 0u64;
        let mut total_hashrate = 0.0;

        for (bench_id, stats) in bench_stats {
            let elapsed = stats.start_time.elapsed().as_secs_f64();
            let hashrate = if elapsed > 0.0 {
                stats.total_generated as f64 / elapsed
            } else {
                0.0
            };

            println!(
                "  {:<8} - {:.0} addr/s ({} generated, {:.1}s elapsed)",
                bench_id, hashrate, stats.total_generated, elapsed
            );

            total_generated += stats.total_generated;
            total_hashrate += hashrate;
        }

        println!(
            "📊 TOTAL    - {:.0} addr/s ({} generated)",
            total_hashrate, total_generated
        );
        println!();
    }

    pub fn log_found_address(&self, bench_id: &str, address: &str, path: &[u32; 6]) {
        println!();
        println!("🎉 MATCH FOUND from {}!", bench_id);
        println!("   Address: {}", address);
        println!(
            "   Path: m/{}/{}/{}/{}/{}/{}",
            path[0], path[1], path[2], path[3], path[4], path[5]
        );
    }

    pub fn log_derivation_error(&self) {
        eprintln!("⚠️  Failed to derive address from path");
    }

    pub fn stop_requested(&self) {
        println!("\n⏸  Stop requested, shutting down workbenches...");
    }

    pub fn workbench_stopped(&self, bench_id: &str, total_generated: u64, elapsed: Duration) {
        println!(
            "✓ {} stopped ({} generated in {:.1}s)",
            bench_id,
            total_generated,
            elapsed.as_secs_f64()
        );
    }

    /// Logs final status when all workbenches are done
    pub fn final_status(&self) {
        println!("\n✅ All workbenches stopped");
    }
}

/// Statistics for a single workbench
pub struct BenchStats {
    pub start_time: Instant,
    pub total_generated: u64,
}

impl BenchStats {
    pub fn new(start_time: Instant) -> Self {
        BenchStats {
            start_time,
            total_generated: 0,
        }
    }
}
