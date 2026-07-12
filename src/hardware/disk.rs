use serde::{Deserialize, Serialize};

use super::unknown;
use super::warning::HdrtWarning;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalDiskInfo {
    pub device: String,
    pub mount_point: String,
    pub file_system: String,
    pub total: String,
    pub used: String,
    pub available: String,
    pub used_percent: f64,
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

impl Default for LogicalDiskInfo {
    fn default() -> Self {
        Self {
            device: unknown(),
            mount_point: unknown(),
            file_system: unknown(),
            total: unknown(),
            used: unknown(),
            available: unknown(),
            used_percent: 0.0,
            source: unknown(),
            warnings: Vec::new(),
        }
    }
}
