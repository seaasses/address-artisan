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

                            // Detect if GPU is onboard/integrated
                            let is_onboard = Self::is_onboard_gpu(device);

                            gpus.push(DeviceInfo::GPU {
                                name,
                                device_index,
                                platform_index,
                                is_onboard,
                            });
                        }
                    }
                }
            }
        }

        gpus
    }

    fn is_onboard_gpu(device: &Device) -> bool {
        // Method 1: Check HostUnifiedMemory - integrated GPUs share memory with CPU
        if let Ok(unified) = device.info(ocl::enums::DeviceInfo::HostUnifiedMemory) {
            if unified.to_string() == "true" || unified.to_string() == "1" {
                return true;
            }
        }

        // Method 2: Check global memory size - integrated typically has less
        if let Ok(mem) = device.info(ocl::enums::DeviceInfo::GlobalMemSize) {
            if let Ok(mem_str) = mem.to_string().parse::<u64>() {
                // Less than 2GB is typically integrated
                if mem_str < 2_147_483_648 {
                    return true;
                }
            }
        }

        // Method 3: Check device name for common integrated GPU vendors
        if let Ok(name) = device.name() {
            let name_lower = name.to_lowercase();
            // Intel integrated GPUs
            if name_lower.contains("intel") &&
               (name_lower.contains("hd") ||
                name_lower.contains("uhd") ||
                name_lower.contains("iris") ||
                name_lower.contains("integrated")) {
                return true;
            }
            // AMD integrated GPUs (APU)
            if name_lower.contains("amd") &&
               (name_lower.contains("radeon") &&
                (name_lower.contains("graphics") || name_lower.contains("vega"))) {
                return true;
            }
        }

        // Method 4: Check vendor - Intel GPUs without "Arc" are typically integrated
        if let Ok(vendor) = device.vendor() {
            let vendor_lower = vendor.to_lowercase();
            if vendor_lower.contains("intel") {
                if let Ok(name) = device.name() {
                    let name_lower = name.to_lowercase();
                    // Discrete Intel GPUs would have "arc"
                    if !name_lower.contains("arc") {
                        return true;
                    }
                }
            }
        }

        false
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
