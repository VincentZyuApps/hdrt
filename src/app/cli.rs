use clap::Parser;

use super::command::Command;
use super::options::{DetailLevel, OutputFormat};
use crate::i18n::Lang;

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

    /// Display language for table, markdown, and TUI output.
    #[arg(long, global = true, value_enum, default_value_t = Lang::EnUs)]
    pub lang: Lang,

    /// Use PowerShell/CIM backend on Windows for more detailed hardware fields.
    ///
    /// Aliases: --ps, --ps1
    #[arg(long, global = true)]
    pub powershell: bool,
}
