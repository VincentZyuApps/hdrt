mod command;
mod cpu;
mod disk;
mod memory;
mod motherboard;

use crate::collector::CollectOptions;
use crate::hardware::{HardwareReport, HdrtWarning};

pub fn collect_report(_options: CollectOptions) -> HardwareReport {
    let mut report = HardwareReport {
        physical_disks: Vec::new(),
        logical_disks: Vec::new(),
        memory: memory::collect(),
        cpu: cpu::collect(),
        motherboard: motherboard::collect(),
        warnings: Vec::new(),
        debug: Vec::new(),
    };

    report.warnings.push(HdrtWarning::with_hint(
        "android-termux-backend",
        "Using the Android/Termux backend based on /proc, df, and getprop.",
        "Android may hide low-level disk, board, serial, firmware, and health fields.",
    ));

    report
}
