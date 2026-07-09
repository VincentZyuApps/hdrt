use serde::{Deserialize, Serialize};

use super::warning::HdrtWarning;
use super::unknown;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub device: String,
    pub model: String,
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
