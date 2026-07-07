use std::fs;

use crate::hardware::{unknown, HdrtWarning, MemoryDevice};

use super::command::format_bytes;

pub(super) fn collect() -> Vec<MemoryDevice> {
    vec![MemoryDevice {
        slot: "System".to_string(),
        size: memory_total().unwrap_or_else(unknown),
        source: "/proc/meminfo".to_string(),
        warnings: vec![HdrtWarning::with_hint(
            "linux-native-memory-slots-unavailable",
            "Native Linux memory collection only reports total memory for now.",
            "Use --backend auto or --backend shell with dmidecode for per-slot module details.",
        )],
        ..MemoryDevice::default()
    }]
}

fn memory_total() -> Option<String> {
    let text = fs::read_to_string("/proc/meminfo").ok()?;
    text.lines().find_map(|line| {
        let rest = line.strip_prefix("MemTotal:")?;
        let kb = rest.split_whitespace().next()?.parse::<u64>().ok()?;
        Some(format_bytes(kb * 1024))
    })
}
