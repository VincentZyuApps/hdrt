use std::collections::HashMap;
use std::fs;
use std::process::Command;

use crate::cli::DetailLevel;
use crate::model::{CpuInfo, DiskInfo, HardwareReport, MemoryDevice, MotherboardInfo};
use crate::privilege;
use crate::warning::HdrtWarning;

use super::format_bytes;

pub fn collect_report(detail: DetailLevel) -> HardwareReport {
    let mut report = HardwareReport {
        disks: collect_disks(detail),
        memory: collect_memory(),
        cpu: collect_cpu(),
        motherboard: collect_motherboard(),
        warnings: Vec::new(),
    };

    if !privilege::is_elevated() {
        report.warnings.push(HdrtWarning::with_hint(
            "not-root",
            "Some Linux hardware fields may be hidden without root privileges.",
            "Run sudo hdrt for more complete disk SMART, memory slot, and board details.",
        ));
    }

    report
}

fn collect_disks(detail: DetailLevel) -> Vec<DiskInfo> {
    let output = run_command(
        "lsblk",
        &[
            "-d", "-P", "-o", "NAME,MODEL,SERIAL,SIZE,ROTA,TYPE,TRAN,VENDOR,REV",
        ],
    );

    let Ok(output) = output else {
        return vec![DiskInfo {
            warnings: vec![HdrtWarning::with_hint(
                "lsblk-unavailable",
                "Could not run lsblk to collect physical disk inventory.",
                "Install util-linux or run hdrt on a Linux system with lsblk available.",
            )],
            ..DiskInfo::default()
        }];
    };

    let mut disks: Vec<DiskInfo> = output
        .lines()
        .filter_map(|line| {
            let values = parse_key_values(line);
            let name = values.get("NAME")?.to_string();
            let rota = values.get("ROTA").map(String::as_str).unwrap_or("");
            let media_type = match rota {
                "0" => "SSD/NVMe",
                "1" => "HDD",
                _ => "Unknown",
            };

            Some(DiskInfo {
                device: name,
                model: value_or_unknown(values.get("MODEL")),
                serial: value_or_unknown(values.get("SERIAL")),
                size: value_or_unknown(values.get("SIZE")),
                media_type: media_type.to_string(),
                bus: value_or_unknown(values.get("TRAN")),
                firmware: value_or_unknown(values.get("REV")),
                source: "lsblk".to_string(),
                ..DiskInfo::default()
            })
        })
        .collect();

    if matches!(detail, DetailLevel::Smart | DetailLevel::Full) {
        enrich_disks_with_smartctl(&mut disks);
    }

    disks
}

fn enrich_disks_with_smartctl(disks: &mut [DiskInfo]) {
    if which::which("smartctl").is_err() {
        for disk in disks {
            disk.warnings.push(HdrtWarning::with_hint(
                "smartctl-missing",
                "smartctl is not installed, so SMART details are unavailable.",
                "Install smartmontools and run sudo hdrt disk --detail smart.",
            ));
        }
        return;
    }

    for disk in disks {
        let path = format!("/dev/{}", disk.device);
        match run_command("smartctl", &["-a", &path]) {
            Ok(output) => apply_smartctl_output(disk, &output),
            Err(err) => disk.warnings.push(HdrtWarning::with_hint(
                "smartctl-failed",
                format!("smartctl failed for {path}: {err}"),
                "Run sudo hdrt disk --detail smart for more complete SMART information.",
            )),
        }
    }
}

fn apply_smartctl_output(disk: &mut DiskInfo, output: &str) {
    for line in output.lines() {
        if let Some(value) = line.strip_prefix("Model Family:") {
            disk.brand = non_empty_or_unknown(value.trim());
            disk.source = "lsblk + smartctl".to_string();
        } else if let Some(value) = line.strip_prefix("Firmware Version:") {
            disk.firmware = non_empty_or_unknown(value.trim());
        } else if let Some(value) =
            line.strip_prefix("SMART overall-health self-assessment test result:")
        {
            disk.health = non_empty_or_unknown(value.trim());
        }
    }
}

fn collect_memory() -> Vec<MemoryDevice> {
    if which::which("dmidecode").is_ok() {
        match run_command("dmidecode", &["-t", "memory"]) {
            Ok(output) => {
                let devices = parse_dmidecode_memory(&output);
                if !devices.is_empty() {
                    return devices;
                }
            }
            Err(_err) => {}
        }
    }

    vec![memory_from_proc()]
}

fn memory_from_proc() -> MemoryDevice {
    let size = fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                let rest = line.strip_prefix("MemTotal:")?;
                let kb = rest.split_whitespace().next()?.parse::<u64>().ok()?;
                Some(format_bytes(kb * 1024))
            })
        })
        .unwrap_or_else(|| "Unknown".to_string());

    MemoryDevice {
        slot: "System".to_string(),
        size,
        source: "/proc/meminfo".to_string(),
        warnings: vec![HdrtWarning::with_hint(
            "dmidecode-unavailable-or-denied",
            "Per-slot memory details are unavailable.",
            "Install dmidecode and run sudo hdrt mem for slot, part number, and serial fields.",
        )],
        ..MemoryDevice::default()
    }
}

fn parse_dmidecode_memory(output: &str) -> Vec<MemoryDevice> {
    let mut devices = Vec::new();
    let mut current: Option<MemoryDevice> = None;

    for line in output.lines() {
        if line.trim() == "Memory Device" {
            flush_memory(&mut devices, current.take());
            current = Some(MemoryDevice {
                source: "dmidecode".to_string(),
                ..MemoryDevice::default()
            });
            continue;
        }

        let Some(device) = current.as_mut() else {
            continue;
        };

        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("Locator:") {
            device.slot = non_empty_or_unknown(value.trim());
        } else if let Some(value) = trimmed.strip_prefix("Size:") {
            device.size = non_empty_or_unknown(value.trim());
        } else if trimmed.starts_with("Speed:") && !trimmed.starts_with("Configured") {
            if let Some(value) = trimmed.strip_prefix("Speed:") {
                device.speed = non_empty_or_unknown(value.trim());
            }
        } else if let Some(value) = trimmed.strip_prefix("Manufacturer:") {
            device.manufacturer = non_empty_or_unknown(value.trim());
        } else if let Some(value) = trimmed.strip_prefix("Part Number:") {
            device.part_number = non_empty_or_unknown(value.trim());
        } else if let Some(value) = trimmed.strip_prefix("Serial Number:") {
            device.serial = non_empty_or_unknown(value.trim());
        }
    }

    flush_memory(&mut devices, current);
    devices
}

fn flush_memory(devices: &mut Vec<MemoryDevice>, current: Option<MemoryDevice>) {
    if let Some(device) = current {
        if !device.size.contains("No Module") && device.size != "Unknown" {
            devices.push(device);
        }
    }
}

fn collect_cpu() -> Option<CpuInfo> {
    let text = fs::read_to_string("/proc/cpuinfo").ok()?;
    let mut cpu = CpuInfo {
        source: "/proc/cpuinfo".to_string(),
        ..CpuInfo::default()
    };
    let mut logical_threads = 0usize;

    for line in text.lines() {
        if let Some(value) = field_value(line, "model name") {
            if cpu.model == "Unknown" {
                cpu.model = value.to_string();
            }
        } else if let Some(value) = field_value(line, "vendor_id") {
            if cpu.vendor == "Unknown" {
                cpu.vendor = value.to_string();
            }
        } else if let Some(value) = field_value(line, "cpu MHz") {
            if cpu.frequency == "Unknown" {
                cpu.frequency = format!("{value} MHz");
            }
        } else if field_value(line, "processor").is_some() {
            logical_threads += 1;
        } else if let Some(value) = field_value(line, "cpu cores") {
            if cpu.physical_cores.is_none() {
                cpu.physical_cores = value.parse().ok();
            }
        }
    }

    if logical_threads > 0 {
        cpu.logical_threads = Some(logical_threads);
    }

    Some(cpu)
}

fn collect_motherboard() -> Option<MotherboardInfo> {
    let read_dmi = |name: &str| -> String {
        fs::read_to_string(format!("/sys/class/dmi/id/{name}"))
            .map(|value| non_empty_or_unknown(value.trim()))
            .unwrap_or_else(|_| "Unknown".to_string())
    };

    Some(MotherboardInfo {
        manufacturer: read_dmi("board_vendor"),
        product: read_dmi("board_name"),
        version: read_dmi("board_version"),
        serial: read_dmi("board_serial"),
        bios_vendor: read_dmi("bios_vendor"),
        bios_version: read_dmi("bios_version"),
        source: "/sys/class/dmi/id".to_string(),
        warnings: if privilege::is_elevated() {
            Vec::new()
        } else {
            vec![HdrtWarning::with_hint(
                "dmi-permission",
                "Some DMI fields may be hidden without root privileges.",
                "Run sudo hdrt mb for more complete board details.",
            )]
        },
    })
}

fn parse_key_values(line: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        let key_start = i;
        while i < chars.len() && chars[i] != '=' {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        let key: String = chars[key_start..i].iter().collect();
        i += 1;

        if i >= chars.len() || chars[i] != '"' {
            break;
        }
        i += 1;
        let value_start = i;
        while i < chars.len() && chars[i] != '"' {
            i += 1;
        }
        let value: String = chars[value_start..i].iter().collect();
        if i < chars.len() {
            i += 1;
        }
        values.insert(key, value);
    }

    values
}

fn field_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let (line_key, value) = line.split_once(':')?;
    if line_key.trim() == key {
        Some(value.trim())
    } else {
        None
    }
}

fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| err.to_string())?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|err| err.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

fn value_or_unknown(value: Option<&String>) -> String {
    value
        .map(|value| non_empty_or_unknown(value.trim()))
        .unwrap_or_else(|| "Unknown".to_string())
}

fn non_empty_or_unknown(value: &str) -> String {
    if value.trim().is_empty() {
        "Unknown".to_string()
    } else {
        value.trim().to_string()
    }
}
