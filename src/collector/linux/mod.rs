mod command;
mod cpu;
mod disk;
mod memory;
mod motherboard;
mod native;
mod native_disk;
mod native_memory;
mod shell;

use std::time::Instant;

use crate::app::options::Backend;
use crate::collector::capability;
use crate::collector::CollectOptions;
use crate::collector::{BenchmarkReport, BenchmarkRow};
use crate::hardware::{
    is_unknown, CpuInfo, DiskInfo, HardwareReport, HdrtWarning, MemoryDevice, MotherboardInfo,
};

pub fn collect_report(options: CollectOptions) -> HardwareReport {
    let mut report = match options.backend {
        Backend::Auto => collect_auto(options),
        Backend::Native => native::collect_report(),
        Backend::Shell => shell::collect_report(options),
    };

    if !capability::is_elevated() {
        report.warnings.push(HdrtWarning::with_hint(
            "not-root",
            "Some Linux hardware fields may be hidden without root privileges.",
            "Run sudo hdrt for more complete disk SMART, memory slot, and board details.",
        ));
    }

    report
}

pub fn benchmark_report(options: CollectOptions) -> BenchmarkReport {
    let rows = vec![
        benchmark_backend(
            "auto",
            options,
            Backend::Auto,
            "native first, shell fills missing fields",
        ),
        benchmark_backend(
            "native",
            options,
            Backend::Native,
            "/sys + /proc + DMI files",
        ),
        benchmark_backend(
            "shell",
            options,
            Backend::Shell,
            "lsblk + smartctl + dmidecode + /proc",
        ),
    ];

    BenchmarkReport {
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        rows,
    }
}

fn benchmark_backend(
    name: &str,
    options: CollectOptions,
    backend: Backend,
    note: &str,
) -> BenchmarkRow {
    let started = Instant::now();
    let report = collect_report(CollectOptions { backend, ..options });

    BenchmarkRow {
        backend: name.to_string(),
        ok: true,
        elapsed_ms: started.elapsed().as_millis(),
        disks: report.physical_disks.len(),
        memory: report.memory.len(),
        warnings: report.warnings.len(),
        note: note.to_string(),
    }
}

fn collect_auto(options: CollectOptions) -> HardwareReport {
    let native = native::collect_report();
    let shell = shell::collect_report(options);

    HardwareReport {
        physical_disks: merge_disks(native.physical_disks, shell.physical_disks),
        logical_disks: Vec::new(),
        memory: merge_memory(native.memory, shell.memory),
        cpu: merge_cpu(native.cpu, shell.cpu),
        motherboard: merge_motherboard(native.motherboard, shell.motherboard),
        warnings: merge_warnings(native.warnings, shell.warnings),
        debug: merge_debug(native.debug, shell.debug),
    }
}

fn merge_disks(mut native: Vec<DiskInfo>, shell: Vec<DiskInfo>) -> Vec<DiskInfo> {
    for shell_disk in shell {
        if let Some(native_disk) = native
            .iter_mut()
            .find(|disk| disk.device == shell_disk.device)
        {
            merge_disk(native_disk, shell_disk);
        } else {
            native.push(shell_disk);
        }
    }
    native
}

fn merge_disk(native: &mut DiskInfo, shell: DiskInfo) {
    let mut used_shell = false;
    used_shell |= fill_string(&mut native.model, shell.model);
    used_shell |= fill_string(&mut native.serial, shell.serial);
    used_shell |= fill_string(&mut native.size, shell.size);
    used_shell |= fill_string(&mut native.media_type, shell.media_type);
    used_shell |= fill_string(&mut native.bus, shell.bus);
    used_shell |= fill_string(&mut native.firmware, shell.firmware);
    used_shell |= fill_string(&mut native.health, shell.health);
    native.warnings.extend(shell.warnings);
    if used_shell {
        native.source = "native + shell".to_string();
    }
}

fn merge_memory(native: Vec<MemoryDevice>, shell: Vec<MemoryDevice>) -> Vec<MemoryDevice> {
    if has_memory_slots(&shell) {
        return shell;
    }
    if native.is_empty() {
        return shell;
    }
    native
}

fn has_memory_slots(memory: &[MemoryDevice]) -> bool {
    memory
        .iter()
        .any(|device| !device.slot.eq_ignore_ascii_case("system"))
}

fn merge_cpu(native: Option<CpuInfo>, shell: Option<CpuInfo>) -> Option<CpuInfo> {
    match (native, shell) {
        (Some(mut native), Some(shell)) => {
            let mut used_shell = false;
            used_shell |= fill_string(&mut native.model, shell.model);
            used_shell |= fill_string(&mut native.vendor, shell.vendor);
            if native.physical_cores.is_none() {
                native.physical_cores = shell.physical_cores;
                used_shell |= native.physical_cores.is_some();
            }
            if native.logical_threads.is_none() {
                native.logical_threads = shell.logical_threads;
                used_shell |= native.logical_threads.is_some();
            }
            used_shell |= fill_string(&mut native.frequency, shell.frequency);
            native.warnings.extend(shell.warnings);
            if used_shell {
                native.source = "native + shell".to_string();
            }
            Some(native)
        }
        (Some(native), None) => Some(native),
        (None, shell) => shell,
    }
}

fn merge_motherboard(
    native: Option<MotherboardInfo>,
    shell: Option<MotherboardInfo>,
) -> Option<MotherboardInfo> {
    match (native, shell) {
        (Some(mut native), Some(shell)) => {
            let mut used_shell = false;
            used_shell |= fill_string(&mut native.manufacturer, shell.manufacturer);
            used_shell |= fill_string(&mut native.product, shell.product);
            used_shell |= fill_string(&mut native.version, shell.version);
            used_shell |= fill_string(&mut native.serial, shell.serial);
            used_shell |= fill_string(&mut native.bios_vendor, shell.bios_vendor);
            used_shell |= fill_string(&mut native.bios_version, shell.bios_version);
            native.warnings.extend(shell.warnings);
            if used_shell {
                native.source = "native + shell".to_string();
            }
            Some(native)
        }
        (Some(native), None) => Some(native),
        (None, shell) => shell,
    }
}

fn merge_warnings(mut native: Vec<HdrtWarning>, shell: Vec<HdrtWarning>) -> Vec<HdrtWarning> {
    native.extend(shell);
    native
}

fn merge_debug(
    mut native: Vec<crate::hardware::DebugRecord>,
    shell: Vec<crate::hardware::DebugRecord>,
) -> Vec<crate::hardware::DebugRecord> {
    native.extend(shell);
    native
}

fn fill_string(target: &mut String, fallback: String) -> bool {
    if is_unknown(target) && !is_unknown(&fallback) {
        *target = fallback;
        true
    } else {
        false
    }
}
