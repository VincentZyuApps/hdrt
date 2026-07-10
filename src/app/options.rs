use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    Auto,
    Native,
    Shell,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    Table,
    Wide,
    Compact,
    Json,
    Markdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DetailLevel {
    Basic,
    Smart,
    Full,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpinnerStyle {
    Unicode,
    Ascii,
    Dots,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TuiTab {
    Overview,
    #[value(alias = "disk", alias = "physical")]
    PhysicalDisk,
    #[value(alias = "logical")]
    LogicalDisk,
    Memory,
    Cpu,
    Motherboard,
    Warnings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChartMode {
    Gauge,
    Bar,
    Sparkline,
    Line,
    Scatter,
}
