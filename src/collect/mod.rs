use crate::cli::DetailLevel;
use crate::model::{CapabilityReport, HardwareReport, ToolStatus};
use crate::privilege;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "android")]
use android as platform;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(windows)]
use windows as platform;

pub fn collect_report(detail: DetailLevel) -> HardwareReport {
    platform::collect_report(detail)
}

pub fn capability_report() -> CapabilityReport {
    let tool_specs = [
        ("smartctl", "SMART, firmware, health, model family"),
        ("dmidecode", "memory slots, baseboard, BIOS details"),
        ("nvme", "NVMe controller and SMART details"),
        ("lsblk", "Linux block device inventory"),
        ("lscpu", "Linux CPU details"),
    ];

    let tools = tool_specs
        .into_iter()
        .map(|(name, purpose)| {
            let path = which::which(name)
                .ok()
                .map(|path| path.to_string_lossy().to_string());
            ToolStatus {
                name: name.to_string(),
                available: path.is_some(),
                path,
                purpose: purpose.to_string(),
            }
        })
        .collect();

    let mut notes = vec![privilege::elevated_hint().to_string()];
    if cfg!(target_os = "android") {
        notes.push("Android/Termux usually exposes fewer low-level hardware fields.".to_string());
    }

    CapabilityReport {
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        elevated: privilege::is_elevated(),
        tools,
        notes,
    }
}

#[cfg(not(any(target_os = "android", target_os = "linux", target_os = "macos", windows)))]
mod platform {
    use crate::cli::DetailLevel;
    use crate::model::HardwareReport;
    use crate::warning::HdrtWarning;

    pub fn collect_report(_detail: DetailLevel) -> HardwareReport {
        let mut report = HardwareReport::default();
        report.warnings.push(HdrtWarning::with_hint(
            "unsupported-platform",
            "This platform does not have a collector yet.",
            "Open an issue with your target platform and available hardware tools.",
        ));
        report
    }
}

#[cfg(any(target_os = "android", target_os = "macos", windows))]
fn placeholder_report(source: &str) -> HardwareReport {
    use crate::model::{CpuInfo, MemoryDevice, MotherboardInfo};
    use crate::warning::HdrtWarning;

    let memory = vec![MemoryDevice {
        slot: "System".to_string(),
        source: source.to_string(),
        warnings: vec![HdrtWarning::with_hint(
            "memory-slot-details-unavailable",
            "Memory details need a platform-specific backend.",
            "Use a platform-specific backend for per-slot details.",
        )],
        ..MemoryDevice::default()
    }];

    let cpu = Some(CpuInfo {
        model: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        source: source.to_string(),
        ..CpuInfo::default()
    });

    let motherboard = Some(MotherboardInfo {
        source: source.to_string(),
        warnings: vec![HdrtWarning::with_hint(
            "motherboard-details-unavailable",
            "Motherboard details need a platform-specific backend.",
            "Use hdrt doctor to inspect available backends.",
        )],
        ..MotherboardInfo::default()
    });

    HardwareReport {
        disks: Vec::new(),
        memory,
        cpu,
        motherboard,
        warnings: vec![HdrtWarning::with_hint(
            "fallback-collector",
            format!("Using placeholder fallback collector: {source}."),
            "Some hardware fields may be Unknown until the native collector is implemented.",
        )],
    }
}

#[cfg(target_os = "linux")]
fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}
