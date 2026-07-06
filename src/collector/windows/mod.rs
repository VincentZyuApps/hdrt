mod basic;
mod cpu;
mod disk;
mod memory;
mod motherboard;
mod powershell;
pub(crate) mod privilege;
mod registry;
mod util;

use crate::collector::CollectOptions;
use crate::hardware::{HardwareReport, HdrtWarning};

pub fn collect_report(options: CollectOptions) -> HardwareReport {
    if options.powershell {
        match powershell::collect_report() {
            Ok(mut report) => {
                add_administrator_warning(&mut report);
                report
            }
            Err(err) => powershell::fallback_report(err),
        }
    } else {
        let mut report = basic::collect_report();
        report.warnings.push(HdrtWarning::with_hint(
            "windows-basic-backend",
            "Using the default Windows basic backend.",
            "Run hdrt --powershell for richer CIM fields such as memory part numbers, serials, BIOS, and physical disk health.",
        ));
        report
    }
}

fn add_administrator_warning(report: &mut HardwareReport) {
    if !privilege::is_elevated() {
        report.warnings.push(HdrtWarning::with_hint(
            "not-administrator",
            "Some Windows hardware fields may be hidden without Administrator privileges.",
            "Run hdrt --powershell from Administrator PowerShell for more complete disk, board, and BIOS details.",
        ));
    }
}
