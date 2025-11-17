#[derive(Debug, Clone, PartialEq)]
pub enum DeviceInfo {
    Cpu {
        name: String,
        threads: u32,
    },
    Gpu {
        name: String,
        device_index: usize,
        platform_index: usize,
        is_onboard: bool,
    },
}

impl DeviceInfo {
    pub fn name(&self) -> &str {
        match self {
            DeviceInfo::Cpu { name, .. } => name,
            DeviceInfo::Gpu { name, .. } => name,
        }
    }

    pub fn with_threads(self, threads: u32) -> Self {
        match self {
            DeviceInfo::Cpu { name, .. } => DeviceInfo::Cpu { name, threads },
            gpu @ DeviceInfo::Gpu { .. } => gpu, // GPU doesn't use threads
        }
    }

    pub fn threads(&self) -> Option<u32> {
        match self {
            DeviceInfo::Cpu { threads, .. } => Some(*threads),
            DeviceInfo::Gpu { .. } => None,
        }
    }
}
