use crate::device_info::DeviceInfo;
use crate::display_backend::{BenchStats, UiBackend};
use crate::events::{EventSender, WorkbenchEvent};
use crate::extended_public_key::ExtendedPubKey;
use crate::ground_truth_validator::GroundTruthValidator;
use crate::prefix::Prefix;
use crate::workbench::Workbench;
use crate::workbench_config::WorkbenchConfig;
use crate::workbench_factory::WorkbenchFactory;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// Orchestrator Constants
const GRACEFUL_SHUTDOWN_TIMEOUT_SECS: u64 = 5;
const STATUS_LOG_INTERVAL_SECS: u64 = 2;

pub struct Orchestrator {
    xpub: ExtendedPubKey,
    prefixes: Vec<Prefix>,
    max_depth: u32,
    num_addresses: u32,
    found_addresses: u32,

    stop_signal: Arc<AtomicBool>,

    event_tx: Sender<WorkbenchEvent>,
    event_rx: Receiver<WorkbenchEvent>,

    ground_truth_validator: GroundTruthValidator,

    backend: Box<dyn UiBackend>,
}

impl Orchestrator {
    pub fn new(
        xpub: ExtendedPubKey,
        prefixes: Vec<Prefix>,
        max_depth: u32,
        num_addresses: u32,
        stop_signal: Arc<AtomicBool>,
        ground_truth_validator: GroundTruthValidator,
        backend: Box<dyn UiBackend>,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::channel();

        Self {
            xpub,
            prefixes,
            max_depth,
            num_addresses,
            found_addresses: 0,
            stop_signal,
            event_tx,
            event_rx,
            ground_truth_validator,
            backend,
        }
    }

    pub fn run(&mut self, devices: Vec<DeviceInfo>) {
        for device in devices.iter() {
            self.spawn_workbench(device.clone());
        }

        let mut bench_stats: HashMap<String, BenchStats> = HashMap::new();
        let mut bench_ids: Vec<String> = Vec::new();
        let mut running_benches = devices.len();
        let mut last_log_time = Instant::now();
        let mut stop_time: Option<Instant> = None;

        loop {
            // Check if stop was requested externally (e.g., Ctrl+C)
            if self.stop_signal.load(Ordering::Relaxed) && stop_time.is_none() {
                self.backend.stop_requested();
                for id in &bench_ids {
                    self.backend.workbench_stopping(id);
                }
                stop_time = Some(Instant::now());
            }

            // Use timeout after stop is requested
            let event = if let Some(stop_instant) = stop_time {
                let elapsed = stop_instant.elapsed();
                let timeout = Duration::from_secs(GRACEFUL_SHUTDOWN_TIMEOUT_SECS);
                if elapsed >= timeout {
                    // Timeout passed since stop - exit anyway
                    break;
                }
                let remaining = timeout - elapsed;
                self.event_rx.recv_timeout(remaining)
            } else {
                self.event_rx
                    .recv()
                    .map_err(|_| std::sync::mpsc::RecvTimeoutError::Disconnected)
            };

            let event = match event {
                Ok(e) => e,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Timeout reached - all workbenches should have stopped by now
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // Channel disconnected
                    break;
                }
            };
            match event {
                WorkbenchEvent::Started {
                    bench_id,
                    timestamp,
                } => {
                    self.handle_started(bench_id, timestamp, &mut bench_stats, &mut bench_ids);
                }

                WorkbenchEvent::Progress {
                    bench_id,
                    addresses_generated,
                } => {
                    self.handle_progress(
                        bench_id,
                        addresses_generated,
                        &mut bench_stats,
                        &mut last_log_time,
                    );
                }

                WorkbenchEvent::PotentialMatch {
                    bench_id,
                    path,
                    prefix_id,
                } => {
                    if !self.should_stop() {
                        self.handle_potential_match(bench_id, path, prefix_id);

                        if self.should_stop() {
                            self.backend.stop_requested();
                            // Notify all workbenches that they are stopping
                            for id in &bench_ids {
                                self.backend.workbench_stopping(id);
                            }
                            self.stop_signal.store(true, Ordering::Relaxed);
                            stop_time = Some(Instant::now()); // Start 5 second timeout
                        }
                    }
                }

                WorkbenchEvent::Stopped {
                    bench_id,
                    total_generated,
                    elapsed,
                } => {
                    self.handle_stopped(bench_id, total_generated, elapsed);
                    running_benches -= 1;

                    if running_benches == 0 {
                        break;
                    }
                }
            }
        }

        self.backend.final_status();
    }

    fn spawn_workbench(&mut self, device: DeviceInfo) {
        let xpub = self.xpub.clone();
        let prefixes = self.prefixes.clone();
        let max_depth = self.max_depth;
        let event_tx = self.event_tx.clone();
        let stop_signal = Arc::clone(&self.stop_signal);

        // Create bench_name with device_index for GPUs
        let bench_name = match &device {
            DeviceInfo::Gpu { device_index, .. } => {
                format!("{}_{}", device_index, device.name())
            }
            DeviceInfo::Cpu { .. } => device.name().to_string(),
        };

        // Notify that workbench is starting
        self.backend.workbench_starting(&bench_name);

        let thread_name = format!("{}-bench", bench_name);
        thread::Builder::new()
            .name(thread_name.clone())
            .spawn(move || {
                let seed0 = rand::random::<u32>() & 0x7FFFFFFF;
                let seed1 = rand::random::<u32>() & 0x7FFFFFFF;
                let config = WorkbenchConfig::new(xpub, prefixes, seed0, seed1, max_depth);
                let event_sender = EventSender::new(event_tx, bench_name);

                let bench = WorkbenchFactory::create(
                    device,
                    config,
                    event_sender.clone(),
                    stop_signal.clone(),
                );

                run_workbench(bench, event_sender, stop_signal);
            })
            .unwrap_or_else(|_| panic!("Failed to spawn {} thread", thread_name));
    }

    fn handle_started(
        &mut self,
        bench_id: String,
        timestamp: Instant,
        bench_stats: &mut HashMap<String, BenchStats>,
        bench_ids: &mut Vec<String>,
    ) {
        self.backend.workbench_started(&bench_id);
        bench_stats.insert(bench_id.clone(), BenchStats::new(timestamp));
        bench_ids.push(bench_id);
    }

    fn handle_progress(
        &mut self,
        bench_id: String,
        addresses_generated: u64,
        bench_stats: &mut HashMap<String, BenchStats>,
        last_log_time: &mut Instant,
    ) {
        if let Some(stats) = bench_stats.get_mut(&bench_id) {
            stats.total_generated += addresses_generated;
        }

        if last_log_time.elapsed() >= Duration::from_secs(STATUS_LOG_INTERVAL_SECS) {
            self.backend.log_status(bench_stats);
            *last_log_time = Instant::now();
        }
    }

    fn should_stop(&self) -> bool {
        self.num_addresses > 0 && self.found_addresses >= self.num_addresses
    }

    fn handle_potential_match(&mut self, bench_id: String, path: [u32; 6], prefix_id: u8) {
        // Get the prefix using prefix_id
        let prefix = &self.prefixes[prefix_id as usize];

        // Validate and get address in one derivation to avoid double derivation
        match self
            .ground_truth_validator
            .validate_and_get_address(prefix, &path)
        {
            Ok(Some(address)) => {
                // Match confirmed - log it and increment counter
                self.backend
                    .log_found_address(&bench_id, &address, &path, prefix_id);
                self.found_addresses += 1;
            }
            Ok(None) => {
                // False positive from range matching - not a real match
                self.backend.log_false_positive(&bench_id, &path);
            }
            Err(_) => {
                self.backend.log_derivation_error();
            }
        }
    }

    fn handle_stopped(&mut self, bench_id: String, total_generated: u64, elapsed: Duration) {
        self.backend
            .workbench_stopped(&bench_id, total_generated, elapsed);
    }
}

pub fn run_workbench(
    workbench: Box<dyn Workbench + Send>,
    event_sender: EventSender,
    stop_signal: Arc<AtomicBool>,
) {
    let start_time = Instant::now();

    // Started event is now sent by each workbench implementation

    workbench.start();

    while !stop_signal.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(100));
    }

    workbench.wait();

    let total_generated = workbench.total_generated();
    let elapsed = start_time.elapsed();
    event_sender.stopped(total_generated, elapsed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extended_public_key::ExtendedPubKey;
    use crate::null_backend::NullBackend;

    fn create_test_orchestrator(num_addresses: u32) -> (Orchestrator, Arc<AtomicBool>) {
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let prefixes = vec![Prefix::new("1").unwrap()];
        let stop_signal = Arc::new(AtomicBool::new(false));
        let ground_truth_validator = GroundTruthValidator::new(xpub_str).unwrap();
        let backend: Box<dyn UiBackend> = Box::new(NullBackend::new(Arc::clone(&stop_signal)));

        let orchestrator = Orchestrator::new(
            xpub,
            prefixes,
            10000,
            num_addresses,
            Arc::clone(&stop_signal),
            ground_truth_validator,
            backend,
        );

        (orchestrator, stop_signal)
    }

    #[test]
    fn test_handle_started_registers_bench() {
        let (mut orch, _) = create_test_orchestrator(1);
        let mut bench_stats = HashMap::new();
        let mut bench_ids = Vec::new();

        orch.handle_started(
            "test-bench".to_string(),
            Instant::now(),
            &mut bench_stats,
            &mut bench_ids,
        );

        assert!(bench_stats.contains_key("test-bench"));
        assert_eq!(bench_stats.get("test-bench").unwrap().total_generated, 0);
    }

    #[test]
    fn test_handle_progress_accumulates_delta() {
        let (mut orch, _) = create_test_orchestrator(1);
        let mut bench_stats = HashMap::new();
        let mut bench_ids = Vec::new();
        let mut last_log_time = Instant::now();

        orch.handle_started(
            "bench1".to_string(),
            Instant::now(),
            &mut bench_stats,
            &mut bench_ids,
        );

        orch.handle_progress(
            "bench1".to_string(),
            100,
            &mut bench_stats,
            &mut last_log_time,
        );
        assert_eq!(bench_stats.get("bench1").unwrap().total_generated, 100);

        orch.handle_progress(
            "bench1".to_string(),
            200,
            &mut bench_stats,
            &mut last_log_time,
        );
        assert_eq!(bench_stats.get("bench1").unwrap().total_generated, 300);

        orch.handle_progress(
            "bench1".to_string(),
            50,
            &mut bench_stats,
            &mut last_log_time,
        );
        assert_eq!(bench_stats.get("bench1").unwrap().total_generated, 350);
    }

    #[test]
    fn test_handle_potential_match_derives_address() {
        let (mut orch, _) = create_test_orchestrator(1);
        let path = [1000, 2000, 0, 0, 0, 0];

        orch.handle_potential_match("cpu".to_string(), path, 0);

        // Should have incremented found_addresses
        assert_eq!(orch.found_addresses, 1);
        // Should indicate we should stop (found 1 of 1 requested)
        assert!(orch.should_stop());
    }

    #[test]
    fn test_handle_stopped_does_not_panic() {
        let (mut orch, _) = create_test_orchestrator(1);

        orch.handle_stopped("bench1".to_string(), 5000, Duration::from_secs(10));
    }

    #[test]
    fn test_spawn_workbench_sends_started_event() {
        let (mut orch, stop_signal) = create_test_orchestrator(1);

        orch.spawn_workbench(DeviceInfo::Cpu {
            name: "cpu_2".to_string(),
            threads: 2,
        });

        let event = orch.event_rx.recv_timeout(Duration::from_secs(2)).unwrap();

        match event {
            WorkbenchEvent::Started { bench_id, .. } => {
                assert_eq!(bench_id, "cpu_2");
            }
            _ => panic!("Expected Started event"),
        }

        stop_signal.store(true, Ordering::Relaxed);
    }

    #[test]
    fn test_num_addresses_stops_after_one() {
        let (mut orch, _) = create_test_orchestrator(1);
        let path = [1000, 2000, 0, 0, 0, 0];

        // First match should trigger stop condition
        orch.handle_potential_match("cpu".to_string(), path, 0);
        assert_eq!(orch.found_addresses, 1);
        assert!(orch.should_stop());
    }

    #[test]
    fn test_num_addresses_stops_after_multiple() {
        let (mut orch, _) = create_test_orchestrator(3);
        let path1 = [1000, 2000, 0, 0, 0, 0];
        let path2 = [1001, 2001, 0, 0, 0, 0];
        let path3 = [1002, 2002, 0, 0, 0, 0];

        // First match should not trigger stop
        orch.handle_potential_match("cpu".to_string(), path1, 0);
        assert_eq!(orch.found_addresses, 1);
        assert!(!orch.should_stop());

        // Second match should not trigger stop
        orch.handle_potential_match("cpu".to_string(), path2, 0);
        assert_eq!(orch.found_addresses, 2);
        assert!(!orch.should_stop());

        // Third match should trigger stop
        orch.handle_potential_match("cpu".to_string(), path3, 0);
        assert_eq!(orch.found_addresses, 3);
        assert!(orch.should_stop());
    }

    #[test]
    fn test_num_addresses_zero_never_stops() {
        let (mut orch, _) = create_test_orchestrator(0);

        // Test multiple matches, should never trigger stop
        for i in 0..10 {
            let path = [1000 + i, 2000 + i, 0, 0, 0, 0];
            orch.handle_potential_match("cpu".to_string(), path, 0);
            assert!(!orch.should_stop(), "Should not stop at match {}", i + 1);
            assert_eq!(orch.found_addresses, i + 1);
        }
    }

    #[test]
    fn test_discards_events_exceeding_limit() {
        let (mut orch, _) = create_test_orchestrator(2);

        // Simulate 5 potential matches coming in (like a simple prefix finding multiple addresses)
        let paths = [
            [1000, 2000, 0, 0, 0, 0],
            [1001, 2001, 0, 0, 0, 0],
            [1002, 2002, 0, 0, 0, 0],
            [1003, 2003, 0, 0, 0, 0],
            [1004, 2004, 0, 0, 0, 0],
        ];

        // Send all 5 events to the channel (simulating batch results)
        for path in paths.iter() {
            orch.event_tx
                .send(WorkbenchEvent::PotentialMatch {
                    bench_id: "test".to_string(),
                    path: *path,
                    prefix_id: 0,
                })
                .unwrap();
        }

        // Process events like run() does
        let mut processed = 0;
        while let Ok(event) = orch.event_rx.try_recv() {
            if let WorkbenchEvent::PotentialMatch {
                bench_id,
                path,
                prefix_id,
            } = event
            {
                // Only process if we haven't found enough addresses yet
                if !orch.should_stop() {
                    orch.handle_potential_match(bench_id, path, prefix_id);
                    processed += 1;
                }
                // Otherwise, silently discard (this is what we're testing)
            }
        }

        // Should have processed exactly 2 addresses (the limit)
        assert_eq!(
            orch.found_addresses, 2,
            "Should have found exactly 2 addresses"
        );
        assert_eq!(processed, 2, "Should have processed exactly 2 events");
        assert!(
            orch.should_stop(),
            "Should indicate stop after reaching limit"
        );
    }
}
