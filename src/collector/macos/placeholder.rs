use crate::hardware::{CpuInfo, HardwareReport, HdrtWarning, MemoryDevice, MotherboardInfo};

pub(super) fn collect(source: &str) -> HardwareReport {
    HardwareReport {
        physical_disks: Vec::new(),
        logical_disks: Vec::new(),
        memory: vec![MemoryDevice {
            slot: "System".to_string(),
            source: source.to_string(),
            warnings: vec![HdrtWarning::with_hint(
                "memory-slot-details-unavailable",
                "Memory details need a platform-specific backend.",
                "Use a platform-specific backend for per-slot details.",
            )],
            ..MemoryDevice::default()
        }],
        cpu: Some(CpuInfo {
            model: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
            source: source.to_string(),
            ..CpuInfo::default()
        }),
        motherboard: Some(MotherboardInfo {
            source: source.to_string(),
            warnings: vec![HdrtWarning::with_hint(
                "motherboard-details-unavailable",
                "Motherboard details need a platform-specific backend.",
                "Use hdrt doctor to inspect available backends.",
            )],
            ..MotherboardInfo::default()
        }),
        warnings: vec![HdrtWarning::with_hint(
            "fallback-collector",
            format!("Using placeholder fallback collector: {source}."),
            "Some hardware fields may be Unknown until the native collector is implemented.",
        )],
        debug: Vec::new(),
    }
}
