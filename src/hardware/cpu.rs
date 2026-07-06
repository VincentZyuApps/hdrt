use serde::{Deserialize, Serialize};

use super::warning::HdrtWarning;
use super::unknown;

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
