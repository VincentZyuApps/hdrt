use crate::collector::CollectOptions;
use crate::hardware::HardwareReport;

pub(super) fn collect_report(options: CollectOptions) -> HardwareReport {
    HardwareReport {
        physical_disks: super::disk::collect(options.detail),
        logical_disks: Vec::new(),
        memory: super::memory::collect(),
        cpu: super::cpu::collect(),
        motherboard: super::motherboard::collect(),
        warnings: Vec::new(),
        debug: Vec::new(),
    }
}
