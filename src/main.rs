mod cli;
mod constants;
mod cpu_workbench;
mod device_info;
mod device_manager;
mod device_selector;
mod events;
mod extended_public_key;
mod extended_public_key_deriver;
mod extended_public_key_path_walker;
mod gpu_workbench;
mod ground_truth_validator;
mod logger;
mod opencl;
mod orchestrator;
mod prefix;
mod workbench;
mod workbench_config;
mod workbench_factory;

use cli::Cli;
use device_selector::{DeviceConfig, DeviceSelector};
use extended_public_key::ExtendedPubKey;
use ground_truth_validator::GroundTruthValidator;
use logger::Logger;
use orchestrator::Orchestrator;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    let cli = Cli::parse_args();

    let prefixes = cli.prefixes.clone();
    let xpub = ExtendedPubKey::from_str(&cli.xpub).unwrap();
    let ground_truth_validator = GroundTruthValidator::new(&cli.xpub)
        .expect("Failed to create ground truth validator");

    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = Arc::clone(&stop_signal);

    let logger_for_ctrlc = Logger::new();
    ctrlc::set_handler(move || {
        logger_for_ctrlc.stop_requested();
        stop_signal_clone.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl+C handler");

    // Use DeviceSelector to handle all device selection logic
    let device_config = DeviceConfig::from(&cli);
    let selected_devices = match DeviceSelector::select_devices(device_config) {
        Ok(devices) => devices,
        Err(error_msg) => {
            eprintln!("{}", error_msg);
            std::process::exit(1);
        }
    };

    // Calculate total threads (for logging purposes)
    let total_cpu_threads: u32 = selected_devices
        .iter()
        .filter_map(|d| d.threads())
        .sum();

    let logger = Logger::new();
    // Format prefixes for display: "1A, 1B, 1C"
    let prefixes_str = prefixes
        .iter()
        .map(|p| p.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    logger.start(&prefixes_str, cli.max_depth, total_cpu_threads);
    println!();

    let mut orchestrator = Orchestrator::new(
        xpub,
        prefixes,
        cli.max_depth,
        stop_signal,
        ground_truth_validator,
        logger,
    );

    orchestrator.run(selected_devices);
}

