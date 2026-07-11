use clap::Parser;

use super::command::Command;
use super::options::{Backend, DetailLevel, RenderFormat, SpinnerStyle, TableStyle};
use crate::i18n::Lang;

#[derive(Debug, Clone, Parser)]
#[command(name = "hdrt")]
#[command(version = concat!(env!("CARGO_PKG_VERSION"), " (Hardware Device Rust Ratatui)"))]
#[command(about = "Hardware Device Rust Ratatui: cross-platform hardware info CLI/TUI")]
#[command(after_help = "Memory hint: hdrt can be remembered as \"hard rata\".")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// CLI render format.
    #[arg(long, global = true, value_enum, default_value_t = RenderFormat::Table)]
    pub format: RenderFormat,

    /// CLI table style. Alias: --table-style.
    #[arg(long, visible_alias = "table-style", global = true, value_enum)]
    pub style: Option<TableStyle>,

    #[arg(long, global = true, value_enum, default_value_t = DetailLevel::Basic)]
    pub detail: DetailLevel,

    /// Hardware collection backend.
    ///
    /// auto uses native collectors first and may use shell commands to fill missing fields.
    #[arg(long, global = true, value_enum, default_value_t = Backend::Auto)]
    pub backend: Backend,

    /// Disable the interactive loading spinner.
    #[arg(long, global = true)]
    pub no_spinner: bool,

    /// Loading spinner style.
    #[arg(long, global = true, value_enum, default_value_t = SpinnerStyle::Unicode)]
    pub spinner_style: SpinnerStyle,

    /// Enable emoji decorations in CLI output, help, and TUI.
    #[arg(short = 'e', long, global = true)]
    pub emoji: bool,

    /// Disable ANSI colors in CLI output and TUI chrome.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Disable ANSI bold text in CLI output and TUI chrome.
    #[arg(long, global = true)]
    pub no_bold: bool,

    /// Display language for help, table, markdown, and TUI output.
    #[arg(long, global = true, value_enum, default_value_t = Lang::EnUs)]
    pub lang: Lang,

    /// Print additional collector diagnostics after normal output.
    #[arg(long, global = true)]
    pub debug: bool,
}

impl Cli {
    pub fn table_style(&self) -> TableStyle {
        self.style.unwrap_or_default()
    }

    pub fn color_enabled(&self) -> bool {
        !self.no_color && std::env::var_os("NO_COLOR").is_none()
    }

    pub fn bold_enabled(&self) -> bool {
        !self.no_bold
    }
}
