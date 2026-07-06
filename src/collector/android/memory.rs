use std::fs;

use crate::hardware::{unknown, HdrtWarning, MemoryDevice};

use super::command::format_bytes;

pub(super) fn collect() -> Vec<MemoryDevice> {
    vec![MemoryDevice {
        slot: "System".to_string(),
        size: total_memory().unwrap_or_else(unknown),
        source: "/proc/meminfo".to_string(),
        warnings: vec![HdrtWarning::with_hint(
            "android-memory-slot-details-unavailable",
            "Android usually does not expose per-slot memory module details.",
            "Run hdrt on a desktop/server Linux system for DIMM slot, part number, and serial fields.",
        )],
        ..MemoryDevice::default()
    }]
}

fn total_memory() -> Option<String> {
    fs::read_to_string("/proc/meminfo").ok().and_then(|text| {
        text.lines().find_map(|line| {
            let rest = line.strip_prefix("MemTotal:")?;
            let kb = rest.split_whitespace().next()?.parse::<u64>().ok()?;
            Some(format_bytes(kb * 1024))
        })
    })
}
