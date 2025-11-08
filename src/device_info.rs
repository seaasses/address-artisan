#[derive(Debug, Clone)]
pub enum DeviceInfo {
    CPU {
        name: String,
        threads: u32,
    },
    GPU {
        name: String,
        device_index: usize,
        platform_index: usize,
    },
}

impl DeviceInfo {
    pub fn name(&self) -> &str {
        match self {
            DeviceInfo::CPU { name, .. } => name,
            DeviceInfo::GPU { name, .. } => name,
        }
    }

    pub fn with_threads(self, threads: u32) -> Self {
        match self {
            DeviceInfo::CPU { name, .. } => DeviceInfo::CPU { name, threads },
            gpu @ DeviceInfo::GPU { .. } => gpu, // GPU doesn't use threads
        }
    }

    pub fn threads(&self) -> Option<u32> {
        match self {
            DeviceInfo::CPU { threads, .. } => Some(*threads),
            DeviceInfo::GPU { .. } => None,
        }
    }
}
