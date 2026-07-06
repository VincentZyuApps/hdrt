mod basic;
mod cpu;
mod disk;
mod memory;
mod motherboard;
mod native_wmi;
mod powershell;
pub(crate) mod privilege;
mod registry;
mod util;

use crate::collector::CollectOptions;
use crate::collector::{BenchmarkReport, BenchmarkRow};
use crate::hardware::{HardwareReport, HdrtWarning};
use std::time::Instant;

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
        match native_wmi::collect_report() {
            Ok(mut report) => {
                add_administrator_warning(&mut report);
                report
            }
            Err(err) => {
                let mut report = basic::collect_report();
                report.warnings.push(HdrtWarning::with_hint(
                    "windows-native-wmi-fallback",
                    format!("Native WMI backend failed: {err}"),
                    "Using the basic sysinfo + registry backend. Run hdrt --powershell for a PowerShell/CIM comparison.",
                ));
                report
            }
        }
    }
}

pub fn benchmark_report(_options: CollectOptions) -> BenchmarkReport {
    let rows = vec![
        benchmark_basic(),
        benchmark_native_wmi(),
        benchmark_powershell(),
    ];

    BenchmarkReport {
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        rows,
    }
}

fn add_administrator_warning(report: &mut HardwareReport) {
    if !privilege::is_elevated() {
        report.warnings.push(HdrtWarning::with_hint(
            "not-administrator",
            "Some Windows hardware fields may be hidden without Administrator privileges.",
            "Run hdrt from an Administrator terminal for more complete disk, board, and BIOS details.",
        ));
    }
}

fn benchmark_basic() -> BenchmarkRow {
    let started = Instant::now();
    let report = basic::collect_report();
    benchmark_ok(
        "basic",
        started,
        report,
        "sysinfo + registry fallback backend",
    )
}

fn benchmark_native_wmi() -> BenchmarkRow {
    let started = Instant::now();
    match native_wmi::collect_report() {
        Ok(report) => benchmark_ok("native-wmi", started, report, "Rust WMI/CIM backend"),
        Err(err) => benchmark_err("native-wmi", started, err),
    }
}

fn benchmark_powershell() -> BenchmarkRow {
    let started = Instant::now();
    match powershell::collect_report() {
        Ok(report) => benchmark_ok("powershell", started, report, "external PowerShell/CIM backend"),
        Err(err) => benchmark_err("powershell", started, err),
    }
}

fn benchmark_ok(
    backend: impl Into<String>,
    started: Instant,
    report: HardwareReport,
    note: impl Into<String>,
) -> BenchmarkRow {
    BenchmarkRow {
        backend: backend.into(),
        ok: true,
        elapsed_ms: started.elapsed().as_millis(),
        disks: report.disks.len(),
        memory: report.memory.len(),
        warnings: report.warnings.len(),
        note: note.into(),
    }
}

fn benchmark_err(backend: impl Into<String>, started: Instant, err: String) -> BenchmarkRow {
    BenchmarkRow {
        backend: backend.into(),
        ok: false,
        elapsed_ms: started.elapsed().as_millis(),
        disks: 0,
        memory: 0,
        warnings: 0,
        note: err,
    }
}
