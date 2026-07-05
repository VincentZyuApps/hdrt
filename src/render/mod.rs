use anyhow::Result;
use serde::Serialize;

use crate::cli::OutputFormat;
use crate::model::{CapabilityReport, HardwareReport, Section};
use crate::warning::HdrtWarning;

mod markdown;
mod table;

pub fn render_report(
    report: &HardwareReport,
    section: Section,
    format: OutputFormat,
) -> Result<String> {
    match format {
        OutputFormat::Json => render_report_json(report, section),
        OutputFormat::Markdown => Ok(markdown::render_report(report, section)),
        OutputFormat::Table | OutputFormat::Wide | OutputFormat::Compact => {
            Ok(table::render_report(report, section, format))
        }
    }
}

pub fn render_capabilities(report: &CapabilityReport, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(report)?),
        OutputFormat::Markdown => Ok(markdown::render_capabilities(report)),
        OutputFormat::Table | OutputFormat::Wide | OutputFormat::Compact => {
            Ok(table::render_capabilities(report))
        }
    }
}

fn render_report_json(report: &HardwareReport, section: Section) -> Result<String> {
    #[derive(Serialize)]
    struct SelectedReport<'a, T> {
        data: T,
        warnings: &'a [HdrtWarning],
    }

    let value = match section {
        Section::Disk => serde_json::to_value(SelectedReport {
            data: &report.disks,
            warnings: &report.warnings,
        })?,
        Section::Memory => serde_json::to_value(SelectedReport {
            data: &report.memory,
            warnings: &report.warnings,
        })?,
        Section::Cpu => serde_json::to_value(SelectedReport {
            data: &report.cpu,
            warnings: &report.warnings,
        })?,
        Section::Motherboard => serde_json::to_value(SelectedReport {
            data: &report.motherboard,
            warnings: &report.warnings,
        })?,
        Section::All => serde_json::to_value(report)?,
    };

    Ok(serde_json::to_string_pretty(&value)?)
}
