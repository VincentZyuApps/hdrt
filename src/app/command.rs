use clap::Subcommand;

use super::options::TuiTab;

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
