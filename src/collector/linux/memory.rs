use std::fs;

use crate::hardware::{HdrtWarning, MemoryDevice};

use super::command::{format_bytes, non_empty_or_unknown, run_command};

pub(super) fn collect() -> Vec<MemoryDevice> {
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
