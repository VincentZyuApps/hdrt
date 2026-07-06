use anyhow::Result;

use crate::app::options::OutputFormat;
use crate::collector::BenchmarkReport;
use crate::hardware::{CapabilityReport, HardwareReport, Section};
use crate::i18n::Lang;

mod json;
mod markdown;
mod table;

pub fn render_report(
    report: &HardwareReport,
    section: Section,
    format: OutputFormat,
    lang: Lang,
) -> Result<String> {
    match format {
        OutputFormat::Json => json::render_report(report, section),
        OutputFormat::Markdown => Ok(markdown::render_report(report, section, lang)),
        OutputFormat::Table | OutputFormat::Wide | OutputFormat::Compact => {
            Ok(table::render_report(report, section, format, lang))
        }
    }
}

pub fn render_capabilities(
    report: &CapabilityReport,
    format: OutputFormat,
    lang: Lang,
) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(report)?),
        OutputFormat::Markdown => Ok(markdown::render_capabilities(report, lang)),
        OutputFormat::Table | OutputFormat::Wide | OutputFormat::Compact => {
            Ok(table::render_capabilities(report, lang))
        }
    }
}

pub fn render_benchmarks(
    report: &BenchmarkReport,
    format: OutputFormat,
    lang: Lang,
) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(report)?),
        OutputFormat::Markdown => Ok(markdown::render_benchmarks(report, lang)),
        OutputFormat::Table | OutputFormat::Wide | OutputFormat::Compact => {
            Ok(table::render_benchmarks(report, lang))
        }
    }
}
