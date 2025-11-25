use crate::prefix::Prefix;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub trait UiBackend: Send {
    fn start(&mut self, prefixes: &[Prefix], max_depth: u32, cpu_threads: u32);
    fn workbench_starting(&mut self, bench_id: &str);
    fn workbench_started(&mut self, bench_id: &str);

    fn log_status(&mut self, bench_stats: &HashMap<String, BenchStats>);

    fn log_found_address(&mut self, bench_id: &str, address: &str, path: &[u32; 6], prefix_id: u8);

    fn log_derivation_error(&mut self);

    fn log_false_positive(&mut self, bench_id: &str, path: &[u32; 6]);

    fn stop_requested(&mut self);

    fn workbench_stopping(&mut self, bench_id: &str);

    fn workbench_stopped(&mut self, bench_id: &str, total_generated: u64, elapsed: Duration);

    fn final_status(&mut self);
}

/// Statistics for a single workbench
#[derive(Clone)]
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

    /// Returns the runtime in seconds since the workbench started
    pub fn runtime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
