pub mod capability;
pub mod cpu;
pub mod disk;
pub mod memory;
pub mod motherboard;
pub mod report;
pub mod warning;

pub use capability::{CapabilityReport, ToolStatus};
pub use cpu::CpuInfo;
pub use disk::DiskInfo;
pub use memory::MemoryDevice;
pub use motherboard::MotherboardInfo;
pub use report::{HardwareReport, Section};
pub use warning::HdrtWarning;

pub fn unknown() -> String {
    "Unknown".to_string()
}
