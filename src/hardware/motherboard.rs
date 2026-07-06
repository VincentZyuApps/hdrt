use serde::{Deserialize, Serialize};

use super::warning::HdrtWarning;
use super::unknown;

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
