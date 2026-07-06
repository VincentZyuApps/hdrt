mod command;
mod cpu;
mod disk;
mod memory;
mod motherboard;

use crate::collector::capability;
use crate::collector::CollectOptions;
use crate::hardware::{HardwareReport, HdrtWarning};

pub fn collect_report(options: CollectOptions) -> HardwareReport {
    let mut report = HardwareReport {
        disks: disk::collect(options.detail),
        memory: memory::collect(),
        cpu: cpu::collect(),
        motherboard: motherboard::collect(),
        warnings: Vec::new(),
    };

    if !capability::is_elevated() {
        report.warnings.push(HdrtWarning::with_hint(
            "not-root",
            "Some Linux hardware fields may be hidden without root privileges.",
            "Run sudo hdrt for more complete disk SMART, memory slot, and board details.",
        ));
    }

    report
}
