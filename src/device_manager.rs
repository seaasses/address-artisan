use crate::device_info::DeviceInfo;
use ocl::{Device, Platform};
use sysinfo::System;

pub struct DeviceManager;

impl DeviceManager {
    pub fn detect_available_devices() -> Vec<DeviceInfo> {
        let mut devices = Vec::new();

        // Detect CPU
        let cpu_name = Self::detect_cpu_name();
        let cpu_threads = Self::detect_cpu_threads();

        devices.push(DeviceInfo::CPU {
            name: cpu_name,
            threads: cpu_threads,
        });

        // Detect GPUs
        devices.extend(Self::detect_gpus());

        devices
    }

    fn detect_gpus() -> Vec<DeviceInfo> {
        let mut gpus = Vec::new();

        // Get all platforms - Platform::list() returns Vec<Platform> directly
        let platforms = Platform::list();

        for (platform_index, platform) in platforms.iter().enumerate() {
            if let Ok(platform_devices) = Device::list_all(*platform) {
                for (device_index, device) in platform_devices.iter().enumerate() {
                    if let Ok(device_type) = device.info(ocl::enums::DeviceInfo::Type) {
                        // Only include GPU devices
                        if device_type.to_string().contains("GPU") {
                            let name = device
                                .name()
                                .unwrap_or_else(|_| format!("GPU_{}", device_index))
                                .replace(" ", "_");

                            gpus.push(DeviceInfo::GPU {
                                name,
                                device_index,
                                platform_index,
                            });
                        }
                    }
                }
            }
        }

        gpus
    }

    fn detect_cpu_threads() -> u32 {
        let mut system = System::new();
        system.refresh_cpu_all();

        System::physical_core_count().unwrap_or_else(|| system.cpus().len()) as u32
    }

    fn detect_cpu_name() -> String {
        let mut system = System::new();
        system.refresh_cpu_all();

        if let Some(cpu) = system.cpus().first() {
            cpu.brand()
                .trim()
                .replace("  ", " ")
                .replace(" ", "_")
                .replace("(R)", "")
                .replace("(TM)", "")
                .replace("(tm)", "")
                .trim_matches('_')
                .to_string()
        } else {
            "Unknown_CPU".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_available_devices_returns_cpu() {
        let devices = DeviceManager::detect_available_devices();

        assert!(devices.len() >= 1, "Should detect at least one device");

        // First device should always be CPU
        match &devices[0] {
            DeviceInfo::CPU { name, threads } => {
                assert!(!name.is_empty());
                assert_ne!(name, "");
                assert!(*threads > 0);
            }
            DeviceInfo::GPU { .. } => {
                panic!("First device should be CPU, not GPU");
            }
        }
    }

    #[test]
    fn test_device_name_is_detected() {
        let devices = DeviceManager::detect_available_devices();

        let name = devices[0].name();
        assert!(!name.is_empty());
        assert!(!name.contains("  "));
    }

    #[test]
    fn test_detect_cpu_threads_returns_valid_count() {
        let threads = DeviceManager::detect_cpu_threads();

        assert!(threads > 0);
        assert!(threads < 128);
    }

    #[test]
    fn test_device_with_threads_override() {
        let devices = DeviceManager::detect_available_devices();
        let device = devices[0].clone();

        let modified_device = device.with_threads(8);

        assert_eq!(modified_device.threads(), Some(8));
    }

    #[test]
    fn test_detect_cpu_name_returns_valid_string() {
        let cpu_name = DeviceManager::detect_cpu_name();

        assert!(!cpu_name.is_empty());
        assert!(!cpu_name.contains("(R)"));
        assert!(!cpu_name.contains("(TM)"));
        assert!(!cpu_name.contains("(tm)"));
    }
}
