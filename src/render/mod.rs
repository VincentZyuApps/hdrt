use anyhow::Result;

use crate::app::options::{RenderFormat, TableStyle};
use crate::collector::BenchmarkReport;
use crate::hardware::{CapabilityReport, HardwareReport, Section};
use crate::i18n::Lang;

mod debug;
mod json;
mod markdown;
mod style;
mod table;
mod warnings;

pub fn render_report(
    report: &HardwareReport,
    section: Section,
    format: RenderFormat,
    table_style: TableStyle,
    color: bool,
    bold: bool,
    debug: bool,
    lang: Lang,
    emoji: bool,
) -> Result<String> {
    let text_style = style::TextStyle::new(color, bold);
    match format {
        RenderFormat::Json => json::render_report(report, section, lang, emoji),
        RenderFormat::Markdown => Ok(markdown::render_report(report, section, debug, lang, emoji)),
        RenderFormat::Table => Ok(table::render_report(
            report,
            section,
            table_style,
            text_style,
            debug,
            lang,
            emoji,
        )),
    }
}

pub fn render_capabilities(
    report: &CapabilityReport,
    format: RenderFormat,
    table_style: TableStyle,
    color: bool,
    bold: bool,
    lang: Lang,
    emoji: bool,
) -> Result<String> {
    let text_style = style::TextStyle::new(color, bold);
    match format {
        RenderFormat::Json => json::render_capabilities(report, lang, emoji),
        RenderFormat::Markdown => Ok(markdown::render_capabilities(report, lang, emoji)),
        RenderFormat::Table => Ok(table::render_capabilities(
            report,
            table_style,
            text_style,
            lang,
            emoji,
        )),
    }
}

pub fn render_benchmarks(
    report: &BenchmarkReport,
    format: RenderFormat,
    table_style: TableStyle,
    color: bool,
    bold: bool,
    lang: Lang,
    emoji: bool,
) -> Result<String> {
    let text_style = style::TextStyle::new(color, bold);
    match format {
        RenderFormat::Json => json::render_benchmarks(report, lang, emoji),
        RenderFormat::Markdown => Ok(markdown::render_benchmarks(report, lang, emoji)),
        RenderFormat::Table => Ok(table::render_benchmarks(
            report,
            table_style,
            text_style,
            lang,
            emoji,
        )),
    }
}
