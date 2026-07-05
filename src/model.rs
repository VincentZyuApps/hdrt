use serde::{Deserialize, Serialize};

use crate::warning::HdrtWarning;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub device: String,
    pub model: String,
    pub brand: String,
    pub serial: String,
    pub size: String,
    pub media_type: String,
    pub bus: String,
    pub firmware: String,
    pub health: String,
    pub source: String,
    pub warnings: Vec<HdrtWarning>,
}

impl Default for DiskInfo {
    fn default() -> Self {
        Self {
            device: unknown(),
            model: unknown(),
            brand: unknown(),
            serial: unknown(),
            size: unknown(),
            media_type: unknown(),
            bus: unknown(),
            firmware: unknown(),
            health: unknown(),
            source: unknown(),
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDevice {
    pub slot: String,
    pub size: String,
    pub speed: String,
    pub manufacturer: String,
    pub part_number: String,
    pub serial: String,
    pub source: String,
    pub warnings: Vec<HdrtWarning>,
}

impl Default for MemoryDevice {
    fn default() -> Self {
        Self {
            slot: unknown(),
            size: unknown(),
            speed: unknown(),
            manufacturer: unknown(),
            part_number: unknown(),
            serial: unknown(),
            source: unknown(),
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub vendor: String,
    pub physical_cores: Option<usize>,
    pub logical_threads: Option<usize>,
    pub frequency: String,
    pub source: String,
    pub warnings: Vec<HdrtWarning>,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            model: unknown(),
            vendor: unknown(),
            physical_cores: None,
            logical_threads: None,
            frequency: unknown(),
            source: unknown(),
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product: String,
    pub version: String,
    pub serial: String,
    pub bios_vendor: String,
    pub bios_version: String,
    pub source: String,
    pub warnings: Vec<HdrtWarning>,
}

impl Default for MotherboardInfo {
    fn default() -> Self {
        Self {
            manufacturer: unknown(),
            product: unknown(),
            version: unknown(),
            serial: unknown(),
            bios_vendor: unknown(),
            bios_version: unknown(),
            source: unknown(),
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityReport {
    pub platform: String,
    pub arch: String,
    pub elevated: bool,
    pub tools: Vec<ToolStatus>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,
    pub available: bool,
    pub path: Option<String>,
    pub purpose: String,
}

pub fn unknown() -> String {
    "Unknown".to_string()
}
