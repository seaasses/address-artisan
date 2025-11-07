mod bitcoin_address_helper;
mod cli;
mod constants;
mod cpu_workbench;
mod device_info;
mod device_manager;
mod events;
mod extended_public_key;
mod extended_public_key_deriver;
mod extended_public_key_path_walker;
mod logger;
mod orchestrator;
mod prefix;
mod workbench;
mod workbench_config;
mod workbench_factory;

use cli::Cli;
use device_manager::DeviceManager;
use extended_public_key::ExtendedPubKey;
use logger::Logger;
use orchestrator::Orchestrator;
use prefix::Prefix;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    let cli = Cli::parse_args();

    let prefix = Prefix::new(&cli.prefix);
    let xpub = ExtendedPubKey::from_str(&cli.xpub).unwrap();

    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = Arc::clone(&stop_signal);

    let logger_for_ctrlc = Logger::new();
    ctrlc::set_handler(move || {
        logger_for_ctrlc.stop_requested();
        stop_signal_clone.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl+C handler");

    let mut devices = DeviceManager::detect_available_devices();

    if cli.cpu_threads != 0 {
        devices = devices
            .into_iter()
            .map(|device| device.with_threads(cli.cpu_threads))
            .collect();
    }

    let actual_threads = devices
        .first()
        .and_then(|d| d.threads())
        .unwrap_or(cli.cpu_threads);

    let logger = Logger::new();
    logger.start(&cli.prefix, cli.max_depth, actual_threads);
    println!();

    let mut orchestrator = Orchestrator::new(xpub, prefix, cli.max_depth, stop_signal, logger);

    orchestrator.run(devices);
}
