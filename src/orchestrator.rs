use crate::device_info::DeviceInfo;
use crate::events::{EventSender, WorkbenchEvent};
use crate::extended_public_key::ExtendedPubKey;
use crate::ground_truth_validator::GroundTruthValidator;
use crate::logger::{BenchStats, Logger};
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

pub struct Orchestrator {
    xpub: ExtendedPubKey,
    prefixes: Vec<Prefix>,
    max_depth: u32,

    stop_signal: Arc<AtomicBool>,

    event_tx: Sender<WorkbenchEvent>,
    event_rx: Receiver<WorkbenchEvent>,

    ground_truth_validator: GroundTruthValidator,

    logger: Logger,
}

impl Orchestrator {
    pub fn new(
        xpub: ExtendedPubKey,
        prefixes: Vec<Prefix>,
        max_depth: u32,
        stop_signal: Arc<AtomicBool>,
        ground_truth_validator: GroundTruthValidator,
        logger: Logger,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::channel();

        Self {
            xpub,
            prefixes,
            max_depth,
            stop_signal,
            event_tx,
            event_rx,
            ground_truth_validator,
            logger,
        }
    }

    pub fn run(&mut self, devices: Vec<DeviceInfo>) {
        for device in devices.iter() {
            self.spawn_workbench(device.clone());
        }

        let mut bench_stats: HashMap<String, BenchStats> = HashMap::new();
        let mut running_benches = devices.len();
        let mut last_log_time = Instant::now();

        while let Ok(event) = self.event_rx.recv() {
            match event {
                WorkbenchEvent::Started {
                    bench_id,
                    timestamp,
                } => {
                    self.handle_started(bench_id, timestamp, &mut bench_stats);
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

                WorkbenchEvent::PotentialMatch { bench_id, path, prefix_id } => {
                    if self.handle_potential_match(bench_id, path, prefix_id) {
                        self.logger.stop_requested();
                        self.stop_signal.store(true, Ordering::Relaxed);
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

        self.logger.final_status();
    }

    fn spawn_workbench(&self, device: DeviceInfo) {
        let xpub = self.xpub.clone();
        let prefixes = self.prefixes.clone();
        let max_depth = self.max_depth;
        let event_tx = self.event_tx.clone();
        let stop_signal = Arc::clone(&self.stop_signal);

        // Create bench_name with device_index for GPUs
        let bench_name = match &device {
            DeviceInfo::GPU { device_index, .. } => {
                format!("{}_{}", device_index, device.name())
            },
            DeviceInfo::CPU { .. } => {
                device.name().to_string()
            }
        };

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
        &self,
        bench_id: String,
        timestamp: Instant,
        bench_stats: &mut HashMap<String, BenchStats>,
    ) {
        self.logger.workbench_started(&bench_id);
        bench_stats.insert(bench_id, BenchStats::new(timestamp));
    }

    fn handle_progress(
        &self,
        bench_id: String,
        addresses_generated: u64,
        bench_stats: &mut HashMap<String, BenchStats>,
        last_log_time: &mut Instant,
    ) {
        if let Some(stats) = bench_stats.get_mut(&bench_id) {
            stats.total_generated += addresses_generated;
        }

        if last_log_time.elapsed() >= Duration::from_secs(3) {
            self.logger.log_status(bench_stats);
            *last_log_time = Instant::now();
        }
    }

    fn handle_potential_match(&mut self, bench_id: String, path: [u32; 6], prefix_id: u8) -> bool {
        // Get the prefix using prefix_id
        let prefix = &self.prefixes[prefix_id as usize];

        // Validate and get address in one derivation to avoid double derivation
        match self
            .ground_truth_validator
            .validate_and_get_address(prefix, &path)
        {
            Ok(Some(address)) => {
                // Match confirmed - log it
                self.logger.log_found_address(&bench_id, &address, &path);
                true
            }
            Ok(None) => {
                // False positive from range matching - not a real match
                self.logger.log_false_positive(&bench_id, &path);
                false
            }
            Err(_) => {
                self.logger.log_derivation_error();
                false
            }
        }
    }

    fn handle_stopped(&self, bench_id: String, total_generated: u64, elapsed: Duration) {
        self.logger
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

    fn create_test_orchestrator() -> (Orchestrator, Arc<AtomicBool>) {
        let xpub_str = "xpub6CbJVZm8i81HtKFhs61SQw5tR7JxPMdYmZbrhx7UeFdkPG75dX2BNctqPdFxHLU1bKXLPotWbdfNVWmea1g3ggzEGnDAxKdpJcqCUpc5rNn";
        let xpub = ExtendedPubKey::from_str(xpub_str).unwrap();
        let prefixes = vec![Prefix::new("1").unwrap()];
        let stop_signal = Arc::new(AtomicBool::new(false));
        let ground_truth_validator = GroundTruthValidator::new(xpub_str).unwrap();
        let logger = Logger::new();

        let orchestrator = Orchestrator::new(
            xpub,
            prefixes,
            10000,
            Arc::clone(&stop_signal),
            ground_truth_validator,
            logger,
        );

        (orchestrator, stop_signal)
    }

    #[test]
    fn test_handle_started_registers_bench() {
        let (orch, _) = create_test_orchestrator();
        let mut bench_stats = HashMap::new();

        orch.handle_started("test-bench".to_string(), Instant::now(), &mut bench_stats);

        assert!(bench_stats.contains_key("test-bench"));
        assert_eq!(bench_stats.get("test-bench").unwrap().total_generated, 0);
    }

    #[test]
    fn test_handle_progress_accumulates_delta() {
        let (orch, _) = create_test_orchestrator();
        let mut bench_stats = HashMap::new();
        let mut last_log_time = Instant::now();

        orch.handle_started("bench1".to_string(), Instant::now(), &mut bench_stats);

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
        let (mut orch, _) = create_test_orchestrator();
        let path = [1000, 2000, 0, 0, 0, 0];

        let result = orch.handle_potential_match("cpu".to_string(), path, 0);

        assert_eq!(result, true);
    }

    #[test]
    fn test_handle_stopped_does_not_panic() {
        let (orch, _) = create_test_orchestrator();

        orch.handle_stopped("bench1".to_string(), 5000, Duration::from_secs(10));
    }

    #[test]
    fn test_spawn_workbench_sends_started_event() {
        let (orch, stop_signal) = create_test_orchestrator();

        orch.spawn_workbench(DeviceInfo::CPU {
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
}
