use std::fs;

use crate::hardware::CpuInfo;

use super::command::field_value;

pub(super) fn collect() -> Option<CpuInfo> {
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
