use crate::cpu_workbench::CPUWorkbench;
use crate::device_info::DeviceInfo;
use crate::events::EventSender;
use crate::workbench::Workbench;
use crate::workbench_config::WorkbenchConfig;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct WorkbenchFactory;

impl WorkbenchFactory {
    pub fn create(
        device: DeviceInfo,
        config: WorkbenchConfig,
        event_sender: EventSender,
        stop_signal: Arc<AtomicBool>,
    ) -> Box<dyn Workbench + Send> {
        match device {
            DeviceInfo::CPU { threads, .. } => Box::new(CPUWorkbench::new(
                config,
                threads,
                event_sender,
                stop_signal,
            )),
        }
    }
}
