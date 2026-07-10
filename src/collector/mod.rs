pub mod options;

mod benchmark;
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

use sysinfo::Disks;

use crate::hardware::{HardwareReport, LogicalDiskInfo};

pub fn collect_report(options: CollectOptions) -> HardwareReport {
    let mut report = platform::collect_report(options);
    report.logical_disks = collect_logical_disks();
    report
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
            disks: report.physical_disks.len(),
            memory: report.memory.len(),
            warnings: report.warnings.len(),
            note: "platform default collector".to_string(),
        }],
    }
}

pub fn collect_logical_disks() -> Vec<LogicalDiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    disks
        .list()
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            LogicalDiskInfo {
                device: disk.name().to_string_lossy().into_owned(),
                mount_point: disk.mount_point().display().to_string(),
                file_system: disk.file_system().to_string_lossy().into_owned(),
                total: crate::telemetry::format_bytes(total),
                used: crate::telemetry::format_bytes(used),
                available: crate::telemetry::format_bytes(available),
                used_percent: if total == 0 {
                    0.0
                } else {
                    used as f64 / total as f64 * 100.0
                },
                source: "sysinfo".to_string(),
                warnings: Vec::new(),
            }
        })
        .collect()
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
