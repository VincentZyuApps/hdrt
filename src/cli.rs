use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Parser)]
#[command(name = "hdrt")]
#[command(version = concat!(env!("CARGO_PKG_VERSION"), " (Hardware Device Rust Ratatui)"))]
#[command(about = "Hardware Device Rust Ratatui: cross-platform hardware info CLI/TUI")]
#[command(after_help = "Memory hint: hdrt can be remembered as \"hard rata\".")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,

    #[arg(long, global = true, value_enum, default_value_t = DetailLevel::Basic)]
    pub detail: DetailLevel,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Show physical disk information.
    Disk,
    /// Show memory module information.
    #[command(alias = "memory")]
    Mem,
    /// Show CPU information.
    Cpu,
    /// Show motherboard and BIOS information.
    #[command(alias = "motherboard")]
    Mb,
    /// Show all supported hardware sections.
    All,
    /// Show dependency, privilege, and backend status.
    Doctor,
    /// Open the Ratatui interface.
    Tui {
        #[arg(long, value_enum, default_value_t = TuiTab::Overview)]
        tab: TuiTab,
    },
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
pub enum TuiTab {
    Overview,
    Disk,
    Memory,
    Cpu,
    Motherboard,
    Health,
    Warnings,
}
