pub mod cli;
pub mod command;
mod help;
pub mod options;
pub(crate) mod spinner;
#[cfg(test)]
mod tests;

use anyhow::{bail, Result};
use clap::Parser;

use crate::collector::{self, CollectOptions};
use crate::emoji;
use crate::hardware::Section;
use crate::i18n::t;
use crate::{render, tui};

use cli::Cli;
use command::Command;
use options::RenderFormat;
use spinner::Spinner;

pub fn run() -> Result<()> {
    if crate::build_info::try_print_version() {
        return Ok(());
    }

    if help::try_print_localized_help() {
        return Ok(());
    }

    let cli = Cli::parse();
    execute(cli)
}

fn execute(cli: Cli) -> Result<()> {
    validate_cli(&cli)?;

    let command = cli.command.clone().unwrap_or(Command::All);

    match command {
        Command::Disk => print_section(&cli, Section::Disk),
        Command::PhysicalDisk => print_section(&cli, Section::PhysicalDisk),
        Command::LogicalDisk => print_section(&cli, Section::LogicalDisk),
        Command::Memory => print_section(&cli, Section::Memory),
        Command::Cpu => print_section(&cli, Section::Cpu),
        Command::Motherboard => print_section(&cli, Section::Motherboard),
        Command::All => print_section(&cli, Section::All),
        Command::Doctor => print_doctor(&cli),
        Command::Bench => print_benchmarks(&cli),
        Command::Tui {
            tab,
            chart_mode,
            border,
            interval,
        } => tui::run(
            tab,
            chart_mode,
            border,
            cli.lang,
            cli.emoji,
            CollectOptions {
                detail: cli.detail,
                backend: cli.backend,
                debug: cli.debug,
            },
            interval,
            cli.no_spinner,
            cli.spinner_style,
            cli.color_enabled(),
            cli.bold_enabled(),
        ),
    }
}

fn validate_cli(cli: &Cli) -> Result<()> {
    if cli.format != RenderFormat::Table && cli.style.is_some() {
        bail!("--style/--table-style only applies to --format table");
    }
    Ok(())
}

fn print_doctor(cli: &Cli) -> Result<()> {
    let spinner = start_spinner(cli, "spinner.doctor");
    let capabilities = collector::capability_report();
    let rendered = render::render_capabilities(
        &capabilities,
        cli.format,
        cli.table_style(),
        cli.color_enabled(),
        cli.bold_enabled(),
        cli.lang,
        cli.emoji,
    )?;
    spinner.finish();
    println!("{rendered}");
    Ok(())
}

fn print_benchmarks(cli: &Cli) -> Result<()> {
    let spinner = start_spinner(cli, "spinner.bench");
    let benchmarks = collector::benchmark_report(CollectOptions {
        detail: cli.detail,
        backend: cli.backend,
        debug: cli.debug,
    });
    let rendered = render::render_benchmarks(
        &benchmarks,
        cli.format,
        cli.table_style(),
        cli.color_enabled(),
        cli.bold_enabled(),
        cli.lang,
        cli.emoji,
    )?;
    spinner.finish();
    println!("{rendered}");
    Ok(())
}

fn print_section(cli: &Cli, section: Section) -> Result<()> {
    let spinner = start_spinner(cli, "spinner.collect");
    let report = collector::collect_report(CollectOptions {
        detail: cli.detail,
        backend: cli.backend,
        debug: cli.debug,
    });
    let rendered = render::render_report(
        &report,
        section,
        cli.format,
        cli.table_style(),
        cli.color_enabled(),
        cli.bold_enabled(),
        cli.debug,
        cli.lang,
        cli.emoji,
    )?;
    spinner.finish();
    println!("{rendered}");
    Ok(())
}

fn start_spinner(cli: &Cli, message_key: &str) -> Spinner {
    let message = emoji::decorate(cli.emoji, message_key, t(cli.lang, message_key));
    Spinner::start(!cli.no_spinner, cli.spinner_style, message)
}
