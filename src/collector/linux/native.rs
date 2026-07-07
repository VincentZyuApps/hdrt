use crate::hardware::{HardwareReport, HdrtWarning};

pub(super) fn collect_report() -> HardwareReport {
    let mut report = HardwareReport {
        disks: super::native_disk::collect(),
        memory: super::native_memory::collect(),
        cpu: super::cpu::collect(),
        motherboard: super::motherboard::collect(),
        warnings: Vec::new(),
    };

    if report.disks.is_empty() {
        report.warnings.push(HdrtWarning::with_hint(
            "linux-native-disk-empty",
            "Native Linux disk collection returned no block devices.",
            "Use --backend auto or --backend shell to try lsblk-based collection.",
        ));
    }

    report
}
