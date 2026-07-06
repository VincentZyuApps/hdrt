use sysinfo::{Disks, System};

use crate::hardware::{CpuInfo, DiskInfo, HardwareReport, MemoryDevice, MotherboardInfo};

use super::util::{first_known, format_bytes};

pub fn collect_report() -> HardwareReport {
    let mut system = System::new_all();
    system.refresh_all();

    HardwareReport {
        disks: collect_disks(),
        memory: collect_memory(&system),
        cpu: collect_cpu(&system),
        motherboard: collect_motherboard(),
        warnings: Vec::new(),
    }
}

fn collect_disks() -> Vec<DiskInfo> {
    let mut disks = super::registry::physical_disks();
    let logical_disks = collect_logical_disks();

    if disks.is_empty() {
        logical_disks
    } else {
        disks.extend(logical_disks);
        disks
    }
}

fn collect_logical_disks() -> Vec<DiskInfo> {
    Disks::new_with_refreshed_list()
        .iter()
        .map(|disk| {
            let name = disk.name().to_string_lossy().to_string();
            let mount = disk.mount_point().to_string_lossy().to_string();

            DiskInfo {
                device: first_known(&[mount.clone(), name.clone()]),
                model: first_known(&[name, mount]),
                size: format_bytes(disk.total_space()),
                media_type: "Logical".to_string(),
                source: "sysinfo/logical-volume".to_string(),
                ..DiskInfo::default()
            }
        })
        .collect()
}

fn collect_memory(system: &System) -> Vec<MemoryDevice> {
    vec![MemoryDevice {
        slot: "System".to_string(),
        size: format_bytes(system.total_memory()),
        source: "sysinfo".to_string(),
        ..MemoryDevice::default()
    }]
}

fn collect_cpu(system: &System) -> Option<CpuInfo> {
    let registry_cpu = super::registry::cpu_info();

    match system.cpus().first() {
        Some(cpu) => Some(CpuInfo {
            model: first_known(&[
                cpu.brand().to_string(),
                registry_cpu
                    .as_ref()
                    .map(|cpu| cpu.model.clone())
                    .unwrap_or_else(|| "Unknown".to_string()),
            ]),
            vendor: first_known(&[
                cpu.vendor_id().to_string(),
                registry_cpu
                    .as_ref()
                    .map(|cpu| cpu.vendor.clone())
                    .unwrap_or_else(|| "Unknown".to_string()),
            ]),
            physical_cores: System::physical_core_count(),
            logical_threads: Some(system.cpus().len()),
            frequency: if cpu.frequency() > 0 {
                format!("{} MHz", cpu.frequency())
            } else {
                registry_cpu
                    .as_ref()
                    .map(|cpu| cpu.frequency.clone())
                    .unwrap_or_else(|| "Unknown".to_string())
            },
            source: "sysinfo + registry".to_string(),
            ..CpuInfo::default()
        }),
        None => registry_cpu,
    }
}

fn collect_motherboard() -> Option<MotherboardInfo> {
    super::registry::motherboard_info().or_else(|| {
        Some(MotherboardInfo {
            source: "sysinfo".to_string(),
            ..MotherboardInfo::default()
        })
    })
}
