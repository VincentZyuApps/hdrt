pub mod cli;
pub mod command;
pub mod options;
mod spinner;

use anyhow::Result;
use clap::Parser;

use crate::collector::{self, CollectOptions};
use crate::hardware::Section;
use crate::i18n::t;
use crate::{output, ui};

use cli::Cli;
use command::Command;
use spinner::Spinner;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
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
                let spinner = start_spinner(&cli, "spinner.bench");
                let benchmarks = collector::benchmark_report(CollectOptions {
                    detail: cli.detail,
                    backend: cli.backend,
                });
                let rendered = output::render_benchmarks(&benchmarks, cli.format, cli.lang)?;
                spinner.finish();
                println!("{rendered}");
            } else {
                let spinner = start_spinner(&cli, "spinner.doctor");
                let capabilities = collector::capability_report();
                let rendered = output::render_capabilities(&capabilities, cli.format, cli.lang)?;
                spinner.finish();
                println!("{rendered}");
            }
            Ok(())
        }
        Command::Tui { tab } => ui::run(tab, cli.lang),
    }
}

fn print_section(cli: &Cli, section: Section) -> Result<()> {
    let spinner = start_spinner(cli, "spinner.collect");
    let report = collector::collect_report(CollectOptions {
        detail: cli.detail,
        backend: cli.backend,
    });
    let rendered = output::render_report(&report, section, cli.format, cli.lang)?;
    spinner.finish();
    println!("{rendered}");
    Ok(())
}

fn start_spinner(cli: &Cli, message_key: &str) -> Spinner {
    Spinner::start(!cli.no_spinner, cli.spinner_style, t(cli.lang, message_key))
}
