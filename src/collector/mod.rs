pub mod options;

mod benchmark;
pub(crate) mod brand;
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

pub use benchmark::{BenchmarkReport, BenchmarkRow};
pub use capability::capability_report;
pub use options::CollectOptions;

use crate::hardware::HardwareReport;

pub fn collect_report(options: CollectOptions) -> HardwareReport {
    platform::collect_report(options)
}

pub fn benchmark_report(options: CollectOptions) -> BenchmarkReport {
    benchmark_report_platform(options)
}

#[cfg(any(windows, target_os = "linux"))]
fn benchmark_report_platform(options: CollectOptions) -> BenchmarkReport {
    platform::benchmark_report(options)
}

#[cfg(not(any(windows, target_os = "linux")))]
fn benchmark_report_platform(options: CollectOptions) -> BenchmarkReport {
    use std::time::Instant;

    let started = Instant::now();
    let report = platform::collect_report(options);

    BenchmarkReport {
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        rows: vec![BenchmarkRow {
            backend: "default".to_string(),
            ok: true,
            elapsed_ms: started.elapsed().as_millis(),
            disks: report.disks.len(),
            memory: report.memory.len(),
            warnings: report.warnings.len(),
            note: "platform default collector".to_string(),
        }],
    }
}

#[cfg(not(any(
    target_os = "android",
    target_os = "linux",
    target_os = "macos",
    windows
)))]
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
