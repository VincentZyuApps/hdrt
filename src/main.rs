mod cli;
mod collect;
mod model;
mod privilege;
mod render;
mod tui;
mod warning;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};
use model::Section;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command.clone().unwrap_or(Command::All);

    match command {
        Command::Disk => print_section(&cli, Section::Disk),
        Command::Mem => print_section(&cli, Section::Memory),
        Command::Cpu => print_section(&cli, Section::Cpu),
        Command::Mb => print_section(&cli, Section::Motherboard),
        Command::All => print_section(&cli, Section::All),
        Command::Doctor => {
            let capabilities = collect::capability_report();
            println!("{}", render::render_capabilities(&capabilities, cli.format)?);
            Ok(())
        }
        Command::Tui { tab } => tui::run(tab),
    }
}

fn print_section(cli: &Cli, section: Section) -> Result<()> {
    let report = collect::collect_report(cli.detail);
    println!("{}", render::render_report(&report, section, cli.format)?);
    Ok(())
}
