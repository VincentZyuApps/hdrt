use serde_json::Value;

use crate::hardware::{HdrtWarning, MemoryDevice};

use super::util::{first_known, format_bytes, value_array, value_string, value_u64};

pub fn collect(root: &Value) -> Vec<MemoryDevice> {
    let rows: Vec<MemoryDevice> = value_array(root, "Memory")
        .into_iter()
        .map(|memory| {
            let speed = value_string(memory, "ConfiguredClockSpeed");
            let speed = if speed == "Unknown" {
                value_string(memory, "Speed")
            } else {
                speed
            };

            MemoryDevice {
                slot: first_known(&[
                    value_string(memory, "DeviceLocator"),
                    value_string(memory, "BankLabel"),
                ]),
                size: value_u64(memory, "Capacity")
                    .map(format_bytes)
                    .unwrap_or_else(|| "Unknown".to_string()),
                speed: if speed == "Unknown" {
                    speed
                } else {
                    format!("{speed} MT/s")
                },
                manufacturer: value_string(memory, "Manufacturer"),
                part_number: value_string(memory, "PartNumber"),
                serial: value_string(memory, "SerialNumber"),
                source: "Win32_PhysicalMemory".to_string(),
                ..MemoryDevice::default()
            }
        })
        .collect();

    if rows.is_empty() {
        vec![MemoryDevice {
            slot: "System".to_string(),
            source: "Win32_PhysicalMemory".to_string(),
            warnings: vec![HdrtWarning::with_hint(
                "memory-unavailable",
                "Windows memory module information was not returned by CIM.",
                "Run hdrt --backend shell from Administrator PowerShell and try again.",
            )],
            ..MemoryDevice::default()
        }]
    } else {
        rows
    }
}
