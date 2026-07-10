use serde::{Deserialize, Serialize};

use super::{
    CpuInfo, DebugRecord, DiskInfo, HdrtWarning, LogicalDiskInfo, MemoryDevice, MotherboardInfo,
};

#[derive(Debug, Clone, Copy)]
pub enum Section {
    Disk,
    PhysicalDisk,
    LogicalDisk,
    Memory,
    Cpu,
    Motherboard,
    All,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareReport {
    pub physical_disks: Vec<DiskInfo>,
    pub logical_disks: Vec<LogicalDiskInfo>,
    pub memory: Vec<MemoryDevice>,
    pub cpu: Option<CpuInfo>,
    pub motherboard: Option<MotherboardInfo>,
    pub warnings: Vec<HdrtWarning>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub debug: Vec<DebugRecord>,
}
