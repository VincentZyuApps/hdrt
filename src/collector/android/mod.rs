mod command;
mod cpu;
mod logical_disk;
mod memory;
mod motherboard;

use crate::app::options::{Backend, DetailLevel};
use crate::collector::CollectOptions;
use crate::hardware::{HardwareReport, HdrtWarning};

pub fn collect_report(options: CollectOptions) -> HardwareReport {
    let logical = logical_disk::collect(options.debug);
    let mut report = HardwareReport {
        physical_disks: crate::collector::sysfs::collect_physical_disks(),
        logical_disks: logical.disks,
        memory: memory::collect(),
        cpu: cpu::collect(),
        motherboard: motherboard::collect(),
        warnings: logical.warnings,
        debug: logical.debug,
    };

    report.warnings.push(HdrtWarning::with_hint(
        "android-termux-backend",
        "Using the Android/Termux backend based on /proc, /sys/block, df, and getprop.",
        "Android may hide low-level disk, board, serial, firmware, and health fields.",
    ));

    if options.backend != Backend::Auto {
        report.warnings.push(HdrtWarning::with_hint(
            "android-backend-best-effort",
            format!(
                "Android currently uses the same best-effort collectors for --backend {}.",
                backend_label(options.backend)
            ),
            "Use --backend auto unless you are comparing command-line behavior across platforms.",
        ));
    }

    if options.detail != DetailLevel::Basic {
        report.warnings.push(HdrtWarning::with_hint(
            "android-detail-best-effort",
            format!(
                "Android currently collects the same accessible fields for --detail {}.",
                detail_label(options.detail)
            ),
            "Android privacy and SELinux restrictions may hide SMART, serial, firmware, and board details even in full mode.",
        ));
    }

    if report.physical_disks.is_empty() {
        report.warnings.push(HdrtWarning::with_hint(
            "android-physical-disk-unavailable",
            "Android did not expose a readable physical block device through /sys/block.",
            "Physical model, serial, firmware, and health fields often require elevated or vendor-specific access.",
        ));
    }

    let ufs_units = report
        .physical_disks
        .iter()
        .filter(|disk| disk.bus.eq_ignore_ascii_case("ufs"))
        .count();
    if ufs_units > 1 {
        report.warnings.push(HdrtWarning::with_hint(
            "android-ufs-logical-units",
            format!("Android exposed {ufs_units} UFS block logical units."),
            "These entries may be logical units of one UFS device rather than separate physical drives.",
        ));
    }

    report
}

fn backend_label(value: Backend) -> &'static str {
    match value {
        Backend::Auto => "auto",
        Backend::Native => "native",
        Backend::Shell => "shell",
    }
}

fn detail_label(value: DetailLevel) -> &'static str {
    match value {
        DetailLevel::Basic => "basic",
        DetailLevel::Smart => "smart",
        DetailLevel::Full => "full",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_labels_match_cli_values() {
        assert_eq!(backend_label(Backend::Native), "native");
        assert_eq!(detail_label(DetailLevel::Smart), "smart");
    }
}
