use clap::Subcommand;

use super::options::TuiTab;

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Show physical disk information.
    #[command(visible_alias = "d")]
    Disk,
    /// Show memory module information.
    #[command(visible_aliases = ["m", "mem"])]
    Memory,
    /// Show CPU information.
    #[command(visible_alias = "c")]
    Cpu,
    /// Show motherboard and BIOS information.
    #[command(visible_aliases = ["b", "mb"])]
    Motherboard,
    /// Show all supported hardware sections.
    #[command(visible_alias = "a")]
    All,
    /// Show dependency, privilege, and backend status.
    Doctor,
    /// Benchmark available collection backends.
    Bench,
    /// Open the Ratatui interface.
    Tui {
        #[arg(long, value_enum, default_value_t = TuiTab::Overview)]
        tab: TuiTab,
    },
}
