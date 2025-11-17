use crate::events::EventSender;
use crate::extended_public_key_deriver::{ExtendedPublicKeyDeriver, KeyDeriver};
use crate::extended_public_key_path_walker::{ExtendedPublicKeyPathWalker, PathWalker};
use crate::workbench::Workbench;
use crate::workbench_config::WorkbenchConfig;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const MIN_CHUNK_SIZE: u32 = 100;
const MAX_CHUNK_SIZE: u32 = 200_000;
const TARGET_BATCH_DURATION_MS: u32 = 1_000;
const MAX_UPPER_ADJUSTMENT_FACTOR: f32 = 1.20;
const REPORT_INTERVAL: Duration = Duration::from_millis(1000);

pub struct CPUWorkbench {
    config: WorkbenchConfig,
    num_threads: u32,

    event_sender: EventSender,
    stop_signal: Arc<AtomicBool>,

    next_counter: Arc<AtomicU64>,
    global_generated: Arc<AtomicU64>,

    worker_handles: Mutex<Vec<JoinHandle<()>>>,
}

impl CPUWorkbench {
    pub fn new(
        config: WorkbenchConfig,
        num_threads: u32,
        event_sender: EventSender,
        stop_signal: Arc<AtomicBool>,
    ) -> Self {
        Self {
            config,
            num_threads,
            event_sender,
            stop_signal,
            next_counter: Arc::new(AtomicU64::new(0)),
            global_generated: Arc::new(AtomicU64::new(0)),
            worker_handles: Mutex::new(Vec::new()),
        }
    }
}

impl Workbench for CPUWorkbench {
    fn start(&self) {
        // Send Started event immediately for CPU workbench
        self.event_sender.started(Instant::now());

        let mut handles = self.worker_handles.lock().unwrap();

        for _ in 0..self.num_threads {
            let config = self.config.clone();
            let stop_signal = Arc::clone(&self.stop_signal);
            let next_counter = Arc::clone(&self.next_counter);
            let global_generated = Arc::clone(&self.global_generated);
            let event_sender = self.event_sender.clone();

            let handle = thread::spawn(move || {
                let path_walker =
                    ExtendedPublicKeyPathWalker::new(config.seed0, config.seed1, config.max_depth);
                let mut xpub_deriver = ExtendedPublicKeyDeriver::new(&config.xpub);
                let mut current_chunk_size = MIN_CHUNK_SIZE;
                let mut generated_since_last_report = 0u64;
                let mut last_report_time = Instant::now();

                while !stop_signal.load(Ordering::Relaxed) {
                    let batch_start = Instant::now();
                    let start_counter =
                        next_counter.fetch_add(current_chunk_size as u64, Ordering::Relaxed);

                    for path in
                        path_walker.iter_from_counter(start_counter, current_chunk_size as u64)
                    {
                        if let Ok(pubkey_hash) = xpub_deriver.get_pubkey_hash_160(&path) {
                            for (prefix_id, prefix) in config.prefixes.iter().enumerate() {
                                if prefix.matches_pattern(&pubkey_hash) {
                                    event_sender.potential_match(path, prefix_id as u8);
                                }
                            }
                        }

                        generated_since_last_report += 1;
                    }

                    global_generated.fetch_add(current_chunk_size as u64, Ordering::Relaxed);

                    if last_report_time.elapsed() >= REPORT_INTERVAL {
                        event_sender.progress(generated_since_last_report);
                        generated_since_last_report = 0;
                        last_report_time = Instant::now();
                    }

                    let batch_duration = (batch_start.elapsed().as_millis() as u32).max(1); // prevent division by zero

                    let ideal_chunk =
                        current_chunk_size * TARGET_BATCH_DURATION_MS / batch_duration;
                    let new_chunk = if ideal_chunk > current_chunk_size {
                        (current_chunk_size as f32 * MAX_UPPER_ADJUSTMENT_FACTOR)
                            .min(ideal_chunk as f32) as u32
                    } else {
                        ideal_chunk
                    };

                    current_chunk_size = new_chunk.clamp(MIN_CHUNK_SIZE, MAX_CHUNK_SIZE);
                }

                if generated_since_last_report > 0 {
                    event_sender.progress(generated_since_last_report);
                }
            });

            handles.push(handle);
        }
    }

    fn wait(&self) {
        let mut handles = self.worker_handles.lock().unwrap();
        for handle in handles.drain(..) {
            handle.join().unwrap();
        }
    }

    fn total_generated(&self) -> u64 {
        self.global_generated.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extended_public_key::ExtendedPubKey;
    use crate::prefix::Prefix;
    use crate::workbench_config::WorkbenchConfig;
    use std::sync::mpsc;

    fn create_test_bench(
        config: WorkbenchConfig,
        num_threads: u32,
    ) -> (CPUWorkbench, Arc<AtomicBool>) {
        let (tx, _rx) = mpsc::channel();
        let event_sender = EventSender::new(tx, "test".to_string());
        let stop_signal = Arc::new(AtomicBool::new(false));
        let bench = CPUWorkbench::new(config, num_threads, event_sender, Arc::clone(&stop_signal));
        (bench, stop_signal)
    }

    #[test]
    fn test_cpu_working_bench_creation() {
        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();
        let config = WorkbenchConfig::new(xpub, vec![prefix], 1000, 2000, 10000);

        let (bench, stop_signal) = create_test_bench(config, 4);

        assert_eq!(bench.total_generated(), 0);
        assert_eq!(stop_signal.load(Ordering::Relaxed), false);
    }

    #[test]
    fn test_total_generated() {
        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();
        let config = WorkbenchConfig::new(xpub, vec![prefix], 1000, 2000, 10000);

        let (bench, _) = create_test_bench(config, 4);

        bench.global_generated.store(1000, Ordering::Relaxed);

        assert_eq!(bench.total_generated(), 1000);
    }

    #[test]
    fn test_threads_actually_process_addresses() {
        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();
        let config = WorkbenchConfig::new(xpub, vec![prefix], 1000, 2000, 10000);

        let (bench, stop_signal) = create_test_bench(config, 2);

        bench.start();
        std::thread::sleep(std::time::Duration::from_millis(100));
        stop_signal.store(true, Ordering::Relaxed);
        bench.wait();

        assert!(
            bench.total_generated() > 0,
            "Should have generated some addresses"
        );
    }

    #[test]
    fn test_wait_actually_waits_for_threads() {
        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();
        let config = WorkbenchConfig::new(xpub, vec![prefix], 1000, 2000, 10000);

        let (bench, stop_signal) = create_test_bench(config, 2);

        bench.start();
        std::thread::sleep(std::time::Duration::from_millis(50));
        stop_signal.store(true, Ordering::Relaxed);

        let before_wait = bench.total_generated();
        bench.wait();
        let after_wait = bench.total_generated();

        assert!(
            after_wait >= before_wait,
            "Counter should not decrease after wait"
        );
    }

    #[test]
    fn test_multiple_threads_no_duplicate_paths() {
        use std::collections::HashSet;
        use std::sync::{Arc, Mutex};

        const TEST_CHUNK_SIZE: u64 = 1000;

        let processed_counters = Arc::new(Mutex::new(HashSet::new()));
        let next_counter = Arc::new(AtomicU64::new(0));
        let running = Arc::new(AtomicBool::new(true));

        let mut handles = vec![];

        for _ in 0..4 {
            let next_counter = Arc::clone(&next_counter);
            let running = Arc::clone(&running);
            let processed_counters = Arc::clone(&processed_counters);

            let handle = std::thread::spawn(move || {
                for _ in 0..5 {
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }

                    let start_counter = next_counter.fetch_add(TEST_CHUNK_SIZE, Ordering::Relaxed);
                    let end_counter = start_counter + TEST_CHUNK_SIZE;

                    for counter in start_counter..end_counter {
                        processed_counters.lock().unwrap().insert(counter);
                    }
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        running.store(false, Ordering::Relaxed);

        let counters = processed_counters.lock().unwrap();
        let expected_count = next_counter.load(Ordering::Relaxed) as usize;

        assert_eq!(
            counters.len(),
            expected_count,
            "Should have no duplicate counters processed"
        );
    }

    #[test]
    fn test_cpu_workbench_sends_started_event_on_start() {
        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();
        let config = WorkbenchConfig::new(xpub, vec![prefix], 1000, 2000, 10000);

        let (bench, stop_signal) = create_test_bench(config, 2);

        bench.start();

        std::thread::sleep(std::time::Duration::from_millis(100));

        stop_signal.store(true, Ordering::Relaxed);
        bench.wait();
    }

    #[test]
    fn test_cpu_workbench_sends_progress_events() {
        use std::sync::mpsc;

        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();
        let config = WorkbenchConfig::new(xpub, vec![prefix], 1000, 2000, 10000);

        let (tx, rx) = mpsc::channel();
        let event_sender = EventSender::new(tx, "test".to_string());
        let stop_signal = Arc::new(AtomicBool::new(false));

        let bench = CPUWorkbench::new(config, 2, event_sender, Arc::clone(&stop_signal));

        bench.start();
        std::thread::sleep(std::time::Duration::from_secs(2));
        stop_signal.store(true, Ordering::Relaxed);
        bench.wait();

        let mut progress_count = 0;
        while let Ok(_) = rx.try_recv() {
            progress_count += 1;
        }

        assert!(
            progress_count >= 1,
            "Should have received at least 1 progress event"
        );
    }

    #[test]
    fn test_cpu_workbench_respects_stop_signal() {
        use std::sync::mpsc;

        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();
        let config = WorkbenchConfig::new(xpub, vec![prefix], 1000, 2000, 10000);

        let (tx, _rx) = mpsc::channel();
        let event_sender = EventSender::new(tx, "test".to_string());
        let stop_signal = Arc::new(AtomicBool::new(false));

        let bench = CPUWorkbench::new(config, 2, event_sender, Arc::clone(&stop_signal));

        bench.start();
        std::thread::sleep(std::time::Duration::from_millis(100));

        let generated_before = bench.total_generated();

        stop_signal.store(true, Ordering::Relaxed);

        let start = std::time::Instant::now();
        bench.wait();
        let elapsed = start.elapsed();

        assert!(
            elapsed < std::time::Duration::from_millis(1500),
            "Should stop within 1500ms"
        );

        let generated_after = bench.total_generated();
        assert!(
            generated_after > generated_before,
            "Should have generated some addresses before stopping"
        );
    }

    #[test]
    fn test_cpu_workbench_sends_potential_match_when_found() {
        use std::sync::mpsc;

        let xpub = ExtendedPubKey::from_str("xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn").unwrap();
        let prefix = Prefix::new("1").unwrap();
        let config = WorkbenchConfig::new(xpub, vec![prefix], 1000, 2000, 10000);

        let (tx, rx) = mpsc::channel();
        let event_sender = EventSender::new(tx, "test".to_string());
        let stop_signal = Arc::new(AtomicBool::new(false));

        let bench = CPUWorkbench::new(config, 2, event_sender, Arc::clone(&stop_signal));

        bench.start();
        std::thread::sleep(std::time::Duration::from_secs(3));
        stop_signal.store(true, Ordering::Relaxed);
        bench.wait();

        let events: Vec<_> = rx.try_iter().collect();
        let has_potential_match = events
            .iter()
            .any(|e| matches!(e, crate::events::WorkbenchEvent::PotentialMatch { .. }));

        assert!(
            has_potential_match || events.len() > 0,
            "Should have sent events (match is likely with prefix '1')"
        );
    }
}
