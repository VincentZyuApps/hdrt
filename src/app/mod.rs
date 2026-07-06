pub mod cli;
pub mod command;
pub mod options;

use anyhow::Result;
use clap::Parser;

use crate::collector::{self, CollectOptions};
use crate::hardware::Section;
use crate::{output, tui};

use cli::Cli;
use command::Command;

pub fn run() -> Result<()> {
    let cli = Cli::parse_from(normalized_args());
    execute(cli)
}

fn execute(cli: Cli) -> Result<()> {
    let command = cli.command.clone().unwrap_or(Command::All);

    match command {
        Command::Disk => print_section(&cli, Section::Disk),
        Command::Memory => print_section(&cli, Section::Memory),
        Command::Cpu => print_section(&cli, Section::Cpu),
        Command::Motherboard => print_section(&cli, Section::Motherboard),
        Command::All => print_section(&cli, Section::All),
        Command::Doctor { bench } => {
            if bench {
                let benchmarks = collector::benchmark_report(CollectOptions {
                    detail: cli.detail,
                    powershell: cli.powershell,
                });
                println!("{}", output::render_benchmarks(&benchmarks, cli.format)?);
            } else {
                let capabilities = collector::capability_report();
                println!("{}", output::render_capabilities(&capabilities, cli.format)?);
            }
            Ok(())
        }
        Command::Tui { tab } => tui::run(tab),
    }
}

fn normalized_args() -> Vec<String> {
    std::env::args()
        .map(|arg| match arg.as_str() {
            "--ps" | "--ps1" => "--powershell".to_string(),
            _ => arg,
        })
        .collect()
}

fn print_section(cli: &Cli, section: Section) -> Result<()> {
    let report = collector::collect_report(CollectOptions {
        detail: cli.detail,
        powershell: cli.powershell,
    });
    println!("{}", output::render_report(&report, section, cli.format)?);
    Ok(())
}
