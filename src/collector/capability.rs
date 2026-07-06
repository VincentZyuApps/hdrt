use crate::hardware::{CapabilityReport, ToolStatus};

pub fn capability_report() -> CapabilityReport {
    let tools = capability_tools();

    let mut notes = vec![elevated_hint().to_string()];
    if cfg!(windows) {
        notes.push(
            "Windows uses native Rust WMI/CIM by default, then falls back to sysinfo + registry; use --powershell, --ps, or --ps1 only for explicit PowerShell/CIM comparison."
                .to_string(),
        );
    }
    if cfg!(target_os = "android") {
        notes.push("Android/Termux usually exposes fewer low-level hardware fields.".to_string());
    }

    CapabilityReport {
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        elevated: is_elevated(),
        tools,
        notes,
    }
}

fn capability_tools() -> Vec<ToolStatus> {
    if cfg!(windows) {
        return vec![
            ToolStatus {
                name: "native-wmi".to_string(),
                available: true,
                path: Some("WMI COM".to_string()),
                purpose: "default Windows backend for disk, memory, CPU, baseboard, and BIOS inventory"
                    .to_string(),
            },
            ToolStatus {
                name: "sysinfo".to_string(),
                available: true,
                path: Some("built-in Rust crate".to_string()),
                purpose: "fallback backend for logical disks, memory total, and CPU basics"
                    .to_string(),
            },
            ToolStatus {
                name: "windows-registry".to_string(),
                available: true,
                path: Some("HKLM".to_string()),
                purpose: "fallback backend for CPU, BIOS, baseboard, and physical disk PnP inventory"
                    .to_string(),
            },
            command_tool(
                "powershell",
                "optional --powershell/--ps/--ps1 CIM backend for richer Windows hardware fields",
            ),
            command_tool(
                "pwsh",
                "optional PowerShell 7 executable; hdrt currently prefers Windows PowerShell",
            ),
        ];
    }

    [
        ("smartctl", "SMART, firmware, health, model family"),
        ("dmidecode", "memory slots, baseboard, BIOS details"),
        ("nvme", "NVMe controller and SMART details"),
        ("lsblk", "Linux block device inventory"),
        ("lscpu", "Linux CPU details"),
    ]
    .into_iter()
    .map(|(name, purpose)| command_tool(name, purpose))
    .collect()
}

fn command_tool(name: &str, purpose: &str) -> ToolStatus {
    let path = which::which(name)
        .ok()
        .map(|path| path.to_string_lossy().to_string());
    ToolStatus {
        name: name.to_string(),
        available: path.is_some(),
        path,
        purpose: purpose.to_string(),
    }
}

pub(crate) fn is_elevated() -> bool {
    is_elevated_platform()
}

#[cfg(unix)]
fn is_elevated_platform() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(windows)]
fn is_elevated_platform() -> bool {
    super::windows::privilege::is_elevated()
}

#[cfg(not(any(unix, windows)))]
fn is_elevated_platform() -> bool {
    false
}

fn elevated_hint() -> &'static str {
    if cfg!(windows) {
        "Run hdrt from an Administrator terminal for more complete hardware fields."
    } else {
        "Run sudo hdrt for more complete hardware fields."
    }
}
