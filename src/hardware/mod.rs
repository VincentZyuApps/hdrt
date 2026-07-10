pub mod capability;
pub mod cpu;
pub mod debug;
pub mod disk;
pub mod memory;
pub mod motherboard;
pub mod report;
pub mod warning;

pub use capability::{CapabilityReport, ToolStatus};
pub use cpu::CpuInfo;
pub use debug::DebugRecord;
pub use disk::{DiskInfo, LogicalDiskInfo};
pub use memory::MemoryDevice;
pub use motherboard::MotherboardInfo;
pub use report::{HardwareReport, Section};
pub use warning::HdrtWarning;

pub const UNKNOWN: &str = "Unknown";

pub fn unknown() -> String {
    UNKNOWN.to_string()
}

pub fn is_unknown(value: &str) -> bool {
    value == UNKNOWN
}
