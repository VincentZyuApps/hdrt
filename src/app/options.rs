use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    Auto,
    Native,
    Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderFormat {
    Table,
    Json,
    Markdown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TableStyle {
    #[value(alias = "round")]
    #[default]
    Rounded,
    Modern,
    Sharp,
    Psql,
    #[value(alias = "plain")]
    Ascii,
    Blank,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TuiBorder {
    #[value(alias = "round")]
    #[default]
    Rounded,
    #[value(alias = "square")]
    Plain,
    Double,
    Thick,
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
