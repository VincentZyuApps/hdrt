mod placeholder;

use crate::collector::CollectOptions;
use crate::hardware::HardwareReport;

pub fn collect_report(_options: CollectOptions) -> HardwareReport {
    placeholder::collect("placeholder/macos")
}
