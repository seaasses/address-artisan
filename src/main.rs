mod cli;
mod constants;
mod cpu_workbench;
mod device_info;
mod device_manager;
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
use device_manager::DeviceManager;
use extended_public_key::ExtendedPubKey;
use ground_truth_validator::GroundTruthValidator;
use logger::Logger;
use orchestrator::Orchestrator;
use prefix::Prefix;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    let cli = Cli::parse_args();

    let prefix = Prefix::new(&cli.prefix);
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

    let mut all_devices = DeviceManager::detect_available_devices();

    // Apply custom CPU threads if specified
    if cli.cpu_threads != 0 {
        all_devices = all_devices
            .into_iter()
            .map(|device| {
                if matches!(device, device_info::DeviceInfo::CPU { .. }) {
                    device.with_threads(cli.cpu_threads)
                } else {
                    device
                }
            })
            .collect();
    }

    // First, collect all non-onboard GPUs with global indexing
    let mut gpu_global_index = 0;
    let available_gpus: Vec<(usize, device_info::DeviceInfo)> = all_devices
        .iter()
        .filter_map(|device| {
            match device {
                gpu_device @ device_info::DeviceInfo::GPU { is_onboard, .. } => {
                    if !is_onboard {
                        let index = gpu_global_index;
                        gpu_global_index += 1;
                        Some((index, gpu_device.clone()))
                    } else {
                        None
                    }
                },
                _ => None
            }
        })
        .collect();

    // Print available GPUs for user reference
    if !available_gpus.is_empty() {
        println!("Available GPUs:");
        for (index, gpu) in &available_gpus {
            if let device_info::DeviceInfo::GPU { name, .. } = gpu {
                println!("  GPU {}: {}", index, name);
            }
        }
        println!();
    }

    // Filter devices based on --gpu and --gpu-only flags
    all_devices = all_devices
        .into_iter()
        .filter(|device| {
            match device {
                device_info::DeviceInfo::GPU { is_onboard, .. } => {
                    if *is_onboard {
                        // Never include onboard GPUs
                        false
                    } else if cli.gpu_only {
                        // --gpu-only: include all non-onboard GPUs
                        true
                    } else if let Some(ref gpu_ids) = cli.gpu {
                        // --gpu with specific IDs or empty (all GPUs)
                        if gpu_ids.is_empty() {
                            // --gpu without IDs: use all non-onboard GPUs
                            true
                        } else {
                            // Check if this GPU's global index is in the requested list
                            available_gpus.iter().any(|(idx, gpu_info)| {
                                gpu_info == device && gpu_ids.contains(idx)
                            })
                        }
                    } else {
                        // No --gpu flag: don't include GPUs
                        false
                    }
                },
                device_info::DeviceInfo::CPU { .. } => {
                    // Keep CPU only if --gpu-only is NOT set
                    !cli.gpu_only
                }
            }
        })
        .collect();

    // Calculate total threads (for logging purposes)
    let total_cpu_threads: u32 = all_devices
        .iter()
        .filter_map(|d| d.threads())
        .sum();

    let logger = Logger::new();
    logger.start(&cli.prefix, cli.max_depth, total_cpu_threads);
    println!();

    let mut orchestrator = Orchestrator::new(
        xpub,
        prefix,
        cli.max_depth,
        stop_signal,
        ground_truth_validator,
        logger,
    );

    orchestrator.run(all_devices);
}

