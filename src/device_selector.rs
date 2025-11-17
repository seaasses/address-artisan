use crate::cli::Cli;
use crate::device_info::DeviceInfo;
use crate::device_manager::DeviceManager;

pub struct DeviceConfig {
    pub cpu_threads: u32,
    pub gpu_ids: Option<Vec<usize>>,
    pub gpu_only: bool,
}

impl From<&Cli> for DeviceConfig {
    fn from(cli: &Cli) -> Self {
        DeviceConfig {
            cpu_threads: cli.cpu_threads,
            gpu_ids: cli.gpu.clone(),
            gpu_only: cli.gpu_only,
        }
    }
}

pub struct DeviceSelector;

impl DeviceSelector {
    pub fn select_devices(config: DeviceConfig) -> Result<Vec<DeviceInfo>, String> {
        let mut all_devices = DeviceManager::detect_available_devices();

        if config.cpu_threads != 0 {
            all_devices = Self::configure_cpu_threads(all_devices, config.cpu_threads);
        }

        let available_gpus = Self::collect_available_gpus(&all_devices);

        Self::validate_gpu_availability(&config, &available_gpus)?;

        Ok(Self::filter_devices(all_devices, &config, &available_gpus))
    }

    fn configure_cpu_threads(devices: Vec<DeviceInfo>, thread_count: u32) -> Vec<DeviceInfo> {
        devices
            .into_iter()
            .map(|device| {
                if matches!(device, DeviceInfo::Cpu { .. }) {
                    device.with_threads(thread_count)
                } else {
                    device
                }
            })
            .collect()
    }

    fn collect_available_gpus(devices: &[DeviceInfo]) -> Vec<(usize, DeviceInfo)> {
        let mut gpu_global_index = 0;
        devices
            .iter()
            .filter_map(|device| match device {
                gpu_device @ DeviceInfo::Gpu { is_onboard, .. } => {
                    if !is_onboard {
                        let index = gpu_global_index;
                        gpu_global_index += 1;
                        Some((index, gpu_device.clone()))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect()
    }

    fn validate_gpu_availability(
        config: &DeviceConfig,
        available_gpus: &[(usize, DeviceInfo)],
    ) -> Result<(), String> {
        // Validate specific GPU IDs if provided
        if let Some(ref gpu_ids) = config.gpu_ids {
            if !gpu_ids.is_empty() {
                // Check if all requested GPU IDs are valid
                for requested_id in gpu_ids {
                    if *requested_id >= available_gpus.len() {
                        let error_msg = if available_gpus.is_empty() {
                            format!(
                                "Error: GPU {} does not exist. No GPUs available on this system.",
                                requested_id
                            )
                        } else {
                            let gpu_info: Vec<String> = available_gpus
                                .iter()
                                .map(|(idx, gpu)| format!("  GPU {}: {}", idx, gpu.name()))
                                .collect();
                            format!(
                                "Error: GPU {} does not exist.\nAvailable GPUs:\n{}",
                                requested_id,
                                gpu_info.join("\n")
                            )
                        };
                        return Err(error_msg);
                    }
                }
            } else if available_gpus.is_empty() {
                // --gpu flag without IDs means use all GPUs, but none are available
                return Err(
                    "Error: --gpu flag was used but no GPUs are available on this system."
                        .to_string(),
                );
            }
        } else if config.gpu_only && available_gpus.is_empty() {
            // --gpu-only flag but no GPUs available
            return Err(
                "Error: --gpu-only flag was used but no GPUs are available on this system."
                    .to_string(),
            );
        }

        Ok(())
    }

    fn filter_devices(
        devices: Vec<DeviceInfo>,
        config: &DeviceConfig,
        available_gpus: &[(usize, DeviceInfo)],
    ) -> Vec<DeviceInfo> {
        devices
            .into_iter()
            .filter(|device| {
                match device {
                    DeviceInfo::Gpu { is_onboard, .. } => {
                        if *is_onboard {
                            // Never include onboard GPUs
                            false
                        } else if let Some(ref gpu_ids) = config.gpu_ids {
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
                        } else if config.gpu_only {
                            // --gpu-only without specific --gpu IDs: include all non-onboard GPUs
                            true
                        } else {
                            // No --gpu flag: don't include GPUs
                            false
                        }
                    }
                    DeviceInfo::Cpu { .. } => {
                        // Keep CPU only if --gpu-only is NOT set
                        !config.gpu_only
                    }
                }
            })
            .collect()
    }
}
