use std::process::Command;

use serde_json::Value;

use crate::hardware::{CpuInfo, HardwareReport, HdrtWarning, MemoryDevice, MotherboardInfo};

pub fn collect_report() -> Result<HardwareReport, String> {
    let root = run_json()?;

    Ok(HardwareReport {
        disks: super::disk::collect(&root),
        memory: super::memory::collect(&root),
        cpu: super::cpu::collect(&root),
        motherboard: super::motherboard::collect(&root),
        warnings: Vec::new(),
    })
}

pub fn fallback_report(err: String) -> HardwareReport {
    HardwareReport {
        disks: Vec::new(),
        memory: vec![MemoryDevice {
            slot: "System".to_string(),
            source: "placeholder/windows".to_string(),
            ..MemoryDevice::default()
        }],
        cpu: Some(CpuInfo {
            model: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
            source: "placeholder/windows".to_string(),
            ..CpuInfo::default()
        }),
        motherboard: Some(MotherboardInfo {
            source: "placeholder/windows".to_string(),
            ..MotherboardInfo::default()
        }),
        warnings: vec![HdrtWarning::with_hint(
            "windows-shell-collector-failed",
            err,
            "Run hdrt --backend shell from PowerShell or Administrator PowerShell and try again.",
        )],
    }
}

fn run_json() -> Result<Value, String> {
    let script = include_str!("scripts/collect_hardware.ps1");
    let mut errors = Vec::new();

    for program in ["powershell", "powershell.exe", "pwsh", "pwsh.exe"] {
        match run_program(program, script) {
            Ok(value) => return Ok(value),
            Err(err) => errors.push(format!("{program}: {err}")),
        }
    }

    Err(format!(
        "all PowerShell runners failed: {}",
        errors.join("; ")
    ))
}

fn run_program(program: &str, script: &str) -> Result<Value, String> {
    let output = Command::new(program)
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|err| format!("failed to start: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("collector failed: {}", stderr.trim()));
    }

    serde_json::from_slice(&output.stdout).map_err(|err| format!("bad PowerShell JSON: {err}"))
}
