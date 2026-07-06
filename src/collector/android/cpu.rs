use std::fs;

use crate::hardware::{is_unknown, CpuInfo};

use super::command::{field_value, non_empty_or_unknown};

pub(super) fn collect() -> Option<CpuInfo> {
    let text = fs::read_to_string("/proc/cpuinfo").ok()?;
    let mut cpu = CpuInfo {
        source: "/proc/cpuinfo".to_string(),
        ..CpuInfo::default()
    };
    let mut logical_threads = 0usize;

    for line in text.lines() {
        if let Some(value) = field_value(line, "Hardware") {
            if is_unknown(&cpu.model) {
                cpu.model = non_empty_or_unknown(value);
            }
        } else if let Some(value) = field_value(line, "Processor") {
            if is_unknown(&cpu.model) {
                cpu.model = non_empty_or_unknown(value);
            }
        } else if let Some(value) = field_value(line, "model name") {
            if is_unknown(&cpu.model) {
                cpu.model = non_empty_or_unknown(value);
            }
        } else if let Some(value) = field_value(line, "vendor_id") {
            if is_unknown(&cpu.vendor) {
                cpu.vendor = non_empty_or_unknown(value);
            }
        } else if let Some(value) = field_value(line, "CPU implementer") {
            if is_unknown(&cpu.vendor) {
                cpu.vendor = non_empty_or_unknown(value);
            }
        } else if let Some(value) = field_value(line, "cpu MHz") {
            if is_unknown(&cpu.frequency) {
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
    if cpu.logical_threads.is_none() {
        cpu.logical_threads = std::thread::available_parallelism()
            .ok()
            .map(|threads| threads.get());
    }

    Some(cpu)
}
