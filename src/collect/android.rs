use crate::cli::DetailLevel;
use crate::model::HardwareReport;

use super::placeholder_report;

pub fn collect_report(_detail: DetailLevel) -> HardwareReport {
    placeholder_report("placeholder/android-termux")
}
