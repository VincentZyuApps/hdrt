use serde_json::Value;

use crate::hardware::CpuInfo;

use super::util::{value_string, value_u64};

pub fn collect(root: &Value) -> Option<CpuInfo> {
    let cpu = root.get("Cpu")?;
    Some(CpuInfo {
        model: value_string(cpu, "Name"),
        vendor: value_string(cpu, "Manufacturer"),
        physical_cores: value_u64(cpu, "NumberOfCores").map(|value| value as usize),
        logical_threads: value_u64(cpu, "NumberOfLogicalProcessors").map(|value| value as usize),
        frequency: value_u64(cpu, "MaxClockSpeed")
            .map(|value| format!("{value} MHz"))
            .unwrap_or_else(|| "Unknown".to_string()),
        source: "Win32_Processor".to_string(),
        ..CpuInfo::default()
    })
}
