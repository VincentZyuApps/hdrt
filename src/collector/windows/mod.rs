mod basic;
mod cpu;
mod disk;
mod memory;
mod motherboard;
mod native_storage;
mod native_wmi;
pub(crate) mod privilege;
mod registry;
mod shell;
mod util;

use crate::app::options::Backend;
use crate::collector::CollectOptions;
use crate::collector::{BenchmarkReport, BenchmarkRow};
use crate::hardware::{HardwareReport, HdrtWarning};
use std::time::Instant;

pub fn collect_report(options: CollectOptions) -> HardwareReport {
    match options.backend {
        Backend::Auto => collect_auto(options),
        Backend::Native => collect_native(options),
        Backend::Shell => collect_shell(),
    }
}

pub fn benchmark_report(options: CollectOptions) -> BenchmarkReport {
    let rows = vec![
        benchmark_auto(options),
        benchmark_native(options),
        benchmark_shell(),
        benchmark_basic(),
    ];

    BenchmarkReport {
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        rows,
    }
}

fn collect_auto(options: CollectOptions) -> HardwareReport {
    match native_wmi::collect_report(options) {
        Ok(mut report) => {
            add_administrator_warning(&mut report);
            report
        }
        Err(native_err) => match shell::collect_report() {
            Ok(mut report) => {
                report.warnings.push(HdrtWarning::with_hint(
                    "windows-auto-shell-fallback",
                    format!("Native WMI backend failed: {native_err}"),
                    "Using the shell backend because --backend auto permits external collectors.",
                ));
                add_administrator_warning(&mut report);
                report
            }
            Err(shell_err) => {
                let mut report = basic::collect_report();
                report.warnings.push(HdrtWarning::with_hint(
                    "windows-auto-basic-fallback",
                    format!(
                        "Native WMI backend failed: {native_err}; shell backend failed: {shell_err}"
                    ),
                    "Using the basic sysinfo + registry backend.",
                ));
                add_administrator_warning(&mut report);
                report
            }
        },
    }
}

fn collect_native(options: CollectOptions) -> HardwareReport {
    match native_wmi::collect_report(options) {
        Ok(mut report) => {
            add_administrator_warning(&mut report);
            report
        }
        Err(err) => {
            let mut report = basic::collect_report();
            report.warnings.push(HdrtWarning::with_hint(
                "windows-native-basic-fallback",
                format!("Native WMI backend failed: {err}"),
                "Using the native basic sysinfo + registry backend.",
            ));
            add_administrator_warning(&mut report);
            report
        }
    }
}

fn collect_shell() -> HardwareReport {
    match shell::collect_report() {
        Ok(mut report) => {
            add_administrator_warning(&mut report);
            report
        }
        Err(err) => shell::fallback_report(err),
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

fn benchmark_auto(options: CollectOptions) -> BenchmarkRow {
    let started = Instant::now();
    let report = collect_auto(options);
    benchmark_ok(
        "auto",
        started,
        report,
        "native first, shell/basic fallback",
    )
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

fn benchmark_native(options: CollectOptions) -> BenchmarkRow {
    let started = Instant::now();
    let report = collect_native(options);
    benchmark_ok(
        "native",
        started,
        report,
        "Rust WMI/CIM with basic native fallback",
    )
}

fn benchmark_shell() -> BenchmarkRow {
    let started = Instant::now();
    match shell::collect_report() {
        Ok(report) => benchmark_ok("shell", started, report, "external PowerShell/CIM backend"),
        Err(err) => benchmark_err("shell", started, err),
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
        disks: report.physical_disks.len(),
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
