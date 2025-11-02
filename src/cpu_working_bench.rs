use crate::bench_config::BenchConfig;
use crate::extended_public_key_deriver::ExtendedPublicKeyDeriver;
use crate::extended_public_key_path_walker::ExtendedPublicKeyPathWalker;
use crate::state_handler::StateHandler;
use crate::vanity_address::VanityAddress;
use crate::working_bench::{BenchStats, WorkingBench};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const THREADS_BATCH_SIZE: usize = 10000;

pub struct CPUWorkingBench {
    config: BenchConfig,
    num_threads: u32,

    global_generated_counter: Arc<AtomicUsize>,
    global_found_counter: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
    found_addresses: Arc<Mutex<Vec<(String, Vec<u32>)>>>,

    worker_handles: Vec<JoinHandle<()>>,
    start_time: Option<Instant>,
}

impl CPUWorkingBench {
    pub fn new(config: BenchConfig, num_threads: u32) -> Self {
        Self {
            config,
            num_threads,
            global_generated_counter: Arc::new(AtomicUsize::new(0)),
            global_found_counter: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            found_addresses: Arc::new(Mutex::new(Vec::new())),
            worker_handles: Vec::new(),
            start_time: None,
        }
    }

    pub fn get_generated_counter(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.global_generated_counter)
    }

    pub fn get_found_counter(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.global_found_counter)
    }

    pub fn get_running(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    pub fn get_found_addresses(&self) -> Arc<Mutex<Vec<(String, Vec<u32>)>>> {
        Arc::clone(&self.found_addresses)
    }
}

impl WorkingBench for CPUWorkingBench {
    fn start(&mut self) {
        self.running.store(true, Ordering::Relaxed);
        self.start_time = Some(Instant::now());

        for _ in 0..self.num_threads {
            let xpub = self.config.xpub.clone();
            let prefix = self.config.prefix.as_str().to_string();
            let max_depth = self.config.max_depth;
            let global_generated_counter = Arc::clone(&self.global_generated_counter);
            let global_found_counter = Arc::clone(&self.global_found_counter);
            let running = Arc::clone(&self.running);
            let found_addresses = Arc::clone(&self.found_addresses);

            let handle = thread::spawn(move || {
                let initial_path = vec![rand::random::<u32>() & 0x7FFFFFFF];
                let xpub_path_walker = ExtendedPublicKeyPathWalker::new(initial_path, max_depth);
                let mut xpub_deriver = ExtendedPublicKeyDeriver::new(&xpub);
                let vanity_address = VanityAddress::new(&prefix);
                let mut state_handler = StateHandler::new(
                    Arc::clone(&global_generated_counter),
                    Arc::clone(&global_found_counter),
                    running,
                    THREADS_BATCH_SIZE,
                    Arc::clone(&found_addresses),
                );

                for path in xpub_path_walker {
                    if !state_handler.is_running() {
                        break;
                    }
                    if let Ok(pubkey_hash) = xpub_deriver.get_pubkey_hash_160(&path) {
                        if let Some(address) = vanity_address.get_vanity_address(pubkey_hash) {
                            state_handler.add_found_address(address, path);
                        }
                        state_handler.new_generated();
                    }
                }
            });

            self.worker_handles.push(handle);
        }
    }

    fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }

    fn wait(mut self) {
        for handle in self.worker_handles.drain(..) {
            handle.join().unwrap();
        }
    }

    fn get_stats(&self) -> BenchStats {
        let elapsed = self
            .start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::from_secs(0));

        BenchStats {
            bench_id: format!("cpu-{}-{}", self.config.seed0, self.config.seed1),
            addresses_generated: self.global_generated_counter.load(Ordering::Relaxed) as u64,
            addresses_found: self.global_found_counter.load(Ordering::Relaxed) as u64,
            elapsed_time: elapsed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_config::BenchConfig;
    use crate::extended_public_key::ExtendedPubKey;
    use crate::prefix::Prefix;

    #[test]
    fn test_cpu_working_bench_creation() {
        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1");
        let config = BenchConfig::new(xpub, prefix, 1000, 2000, 10000);

        let bench = CPUWorkingBench::new(config, 4);

        assert_eq!(bench.global_generated_counter.load(Ordering::Relaxed), 0);
        assert_eq!(bench.global_found_counter.load(Ordering::Relaxed), 0);
        assert_eq!(bench.running.load(Ordering::Relaxed), false);
    }

    #[test]
    fn test_atomics_are_shared() {
        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1");
        let config = BenchConfig::new(xpub, prefix, 1000, 2000, 10000);

        let bench = CPUWorkingBench::new(config, 4);

        let generated = bench.get_generated_counter();
        let found = bench.get_found_counter();
        let running = bench.get_running();

        generated.store(100, Ordering::Relaxed);
        found.store(5, Ordering::Relaxed);
        running.store(true, Ordering::Relaxed);

        assert_eq!(bench.global_generated_counter.load(Ordering::Relaxed), 100);
        assert_eq!(bench.global_found_counter.load(Ordering::Relaxed), 5);
        assert_eq!(bench.running.load(Ordering::Relaxed), true);
    }

    #[test]
    fn test_get_stats() {
        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1");
        let config = BenchConfig::new(xpub, prefix, 1000, 2000, 10000);

        let bench = CPUWorkingBench::new(config, 4);

        bench.global_generated_counter.store(1000, Ordering::Relaxed);
        bench.global_found_counter.store(3, Ordering::Relaxed);

        let stats = bench.get_stats();

        assert_eq!(stats.bench_id, "cpu-1000-2000");
        assert_eq!(stats.addresses_generated, 1000);
        assert_eq!(stats.addresses_found, 3);
    }

    #[test]
    fn test_start_sets_running_flag() {
        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1");
        let config = BenchConfig::new(xpub, prefix, 1000, 2000, 10000);

        let mut bench = CPUWorkingBench::new(config, 2);

        assert_eq!(bench.running.load(Ordering::Relaxed), false);

        bench.start();

        assert_eq!(bench.running.load(Ordering::Relaxed), true);
        assert!(bench.start_time.is_some());

        bench.stop();
    }

    #[test]
    fn test_stop_clears_running_flag() {
        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1");
        let config = BenchConfig::new(xpub, prefix, 1000, 2000, 10000);

        let mut bench = CPUWorkingBench::new(config, 2);

        bench.start();
        assert_eq!(bench.running.load(Ordering::Relaxed), true);

        bench.stop();
        assert_eq!(bench.running.load(Ordering::Relaxed), false);
    }
}
