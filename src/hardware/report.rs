use serde::{Deserialize, Serialize};

use super::{CpuInfo, DebugRecord, DiskInfo, HdrtWarning, MemoryDevice, MotherboardInfo};

#[derive(Debug, Clone, Copy)]
pub enum Section {
    Disk,
    Memory,
    Cpu,
    Motherboard,
    All,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareReport {
    pub disks: Vec<DiskInfo>,
    pub memory: Vec<MemoryDevice>,
    pub cpu: Option<CpuInfo>,
    pub motherboard: Option<MotherboardInfo>,
    pub warnings: Vec<HdrtWarning>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub debug: Vec<DebugRecord>,
}
