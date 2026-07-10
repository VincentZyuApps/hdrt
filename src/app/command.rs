use clap::Subcommand;

use super::options::{ChartMode, TuiTab};

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Show physical and logical disk information.
    #[command(visible_alias = "d")]
    Disk,
    /// Show physical disk information.
    #[command(visible_alias = "pd")]
    PhysicalDisk,
    /// Show logical disk information.
    #[command(visible_alias = "ld")]
    LogicalDisk,
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
        /// Initial TUI chart mode.
        #[arg(long, value_enum, default_value_t = ChartMode::Gauge)]
        chart_mode: ChartMode,
        /// TUI refresh interval in milliseconds.
        #[arg(short = 't', long, default_value_t = crate::telemetry::DEFAULT_INTERVAL_MS)]
        interval: u64,
    },
}
