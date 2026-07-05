use crate::cli::DetailLevel;
use crate::model::HardwareReport;

use super::sysinfo_report;

pub fn collect_report(_detail: DetailLevel) -> HardwareReport {
    sysinfo_report("sysinfo/windows-fallback")
}
