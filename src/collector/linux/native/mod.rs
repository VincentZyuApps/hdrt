use crate::collector::sysfs;
use crate::hardware::{HardwareReport, HdrtWarning};

mod memory;

pub(super) fn collect_report() -> HardwareReport {
    let mut report = HardwareReport {
        physical_disks: sysfs::collect_physical_disks(),
        logical_disks: Vec::new(),
        memory: memory::collect(),
        cpu: super::cpu::collect(),
        motherboard: super::motherboard::collect(),
        warnings: Vec::new(),
        debug: Vec::new(),
    };

    if report.physical_disks.is_empty() {
        report.warnings.push(HdrtWarning::with_hint(
            "linux-native-disk-empty",
            "Native Linux disk collection returned no block devices.",
            "Use --backend auto or --backend shell to try lsblk-based collection.",
        ));
    }

    report
}
