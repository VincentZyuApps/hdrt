pub mod options;

mod capability;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "android")]
use android as platform;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(windows)]
use windows as platform;

pub use capability::capability_report;
pub use options::CollectOptions;

use crate::hardware::HardwareReport;

pub fn collect_report(options: CollectOptions) -> HardwareReport {
    platform::collect_report(options)
}

#[cfg(not(any(target_os = "android", target_os = "linux", target_os = "macos", windows)))]
mod platform {
    use crate::collector::CollectOptions;
    use crate::hardware::{HardwareReport, HdrtWarning};

    pub fn collect_report(_options: CollectOptions) -> HardwareReport {
        let mut report = HardwareReport::default();
        report.warnings.push(HdrtWarning::with_hint(
            "unsupported-platform",
            "This platform does not have a collector yet.",
            "Open an issue with your target platform and available hardware tools.",
        ));
        report
    }
}
