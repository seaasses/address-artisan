use crate::display_backend::{BenchStats, UiBackend};
use crate::prefix::Prefix;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

/// A null backend that does nothing - used only for testing
pub struct NullBackend;

impl NullBackend {
    pub fn new(_stop_signal: Arc<AtomicBool>) -> Self {
        NullBackend
    }
}

impl UiBackend for NullBackend {
    fn start(&mut self, _prefixes: &[Prefix], _max_depth: u32, _cpu_threads: u32) {}
    fn workbench_starting(&mut self, _bench_id: &str) {}
    fn workbench_started(&mut self, _bench_id: &str) {}
    fn log_status(&mut self, _bench_stats: &HashMap<String, BenchStats>) {}
    fn log_found_address(
        &mut self,
        _bench_id: &str,
        _address: &str,
        _path: &[u32; 6],
        _prefix_id: u8,
    ) {
    }
    fn log_derivation_error(&mut self) {}
    fn log_false_positive(&mut self, _bench_id: &str, _path: &[u32; 6]) {}
    fn stop_requested(&mut self) {}
    fn workbench_stopping(&mut self, _bench_id: &str) {}
    fn workbench_stopped(&mut self, _bench_id: &str, _total_generated: u64, _elapsed: Duration) {}
    fn final_status(&mut self) {}
}
