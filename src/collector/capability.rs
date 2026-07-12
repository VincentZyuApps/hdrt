use crate::hardware::{CapabilityReport, ToolStatus};

pub fn capability_report() -> CapabilityReport {
    let tools = capability_tools();

    let mut notes = vec![elevated_hint().to_string()];
    if cfg!(windows) {
        notes.push(
            "Windows uses --backend auto by default: native Rust WMI/CIM first, shell PowerShell/CIM fallback when needed."
                .to_string(),
        );
    }
    if cfg!(target_os = "linux") {
        notes.push(
            "Linux uses --backend auto by default: native /sys and /proc first, shell tools such as lsblk, smartctl, and dmidecode to fill missing fields."
                .to_string(),
        );
    }
    if cfg!(target_os = "android") {
        notes.push(
            "Android/Termux uses /proc, /sys/block, df, and getprop; physical model, serial, disk health, and firmware fields may still be hidden."
                .to_string(),
        );
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
                purpose:
                    "default Windows backend for disk, memory, CPU, baseboard, and BIOS inventory"
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
                purpose:
                    "fallback backend for CPU, BIOS, baseboard, and physical disk PnP inventory"
                        .to_string(),
            },
            command_tool(
                "powershell",
                "Windows shell backend used by --backend shell or --backend auto fallback",
            ),
            command_tool(
                "pwsh",
                "optional PowerShell 7 executable for the Windows shell backend",
            ),
        ];
    }

    if cfg!(target_os = "android") {
        return vec![
            ToolStatus {
                name: "procfs".to_string(),
                available: true,
                path: Some("/proc/cpuinfo + /proc/meminfo".to_string()),
                purpose: "default Android backend for CPU and memory totals".to_string(),
            },
            ToolStatus {
                name: "sysfs".to_string(),
                available: std::path::Path::new("/sys/block").is_dir(),
                path: Some("/sys/block".to_string()),
                purpose: "best-effort Android physical block device inventory".to_string(),
            },
            command_tool("df", "Android/Termux logical storage inventory"),
            command_tool("getprop", "Android device, board, and OS properties"),
        ];
    }

    if cfg!(target_os = "linux") {
        let mut tools = vec![
            ToolStatus {
                name: "sysfs".to_string(),
                available: true,
                path: Some("/sys/block + /sys/class/dmi/id".to_string()),
                purpose: "native Linux disk, bus, firmware, baseboard, and BIOS inventory"
                    .to_string(),
            },
            ToolStatus {
                name: "procfs".to_string(),
                available: true,
                path: Some("/proc/cpuinfo + /proc/meminfo".to_string()),
                purpose: "native Linux CPU and memory totals".to_string(),
            },
        ];
        tools.extend(
            [
                ("smartctl", "shell SMART, firmware, health, model family"),
                ("dmidecode", "shell memory slots, baseboard, BIOS details"),
                ("nvme", "shell NVMe controller and SMART details"),
                ("lsblk", "shell Linux block device inventory"),
                ("lscpu", "shell Linux CPU details"),
            ]
            .into_iter()
            .map(|(name, purpose)| command_tool(name, purpose)),
        );
        return tools;
    }

    Vec::new()
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
    } else if cfg!(target_os = "android") {
        "Standard Termux cannot gain Android hardware access with sudo; rooted devices may expose more fields when hdrt is launched through su."
    } else {
        "Run sudo hdrt for more complete hardware fields."
    }
}
