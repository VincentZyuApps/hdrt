use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub platform: String,
    pub arch: String,
    pub rows: Vec<BenchmarkRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRow {
    pub backend: String,
    pub ok: bool,
    pub elapsed_ms: u128,
    pub disks: usize,
    pub memory: usize,
    pub warnings: usize,
    pub note: String,
}
