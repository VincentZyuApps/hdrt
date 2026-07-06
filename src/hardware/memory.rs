use serde::{Deserialize, Serialize};

use super::warning::HdrtWarning;
use super::unknown;

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
