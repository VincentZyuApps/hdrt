use anyhow::Result;
use serde::Serialize;

use crate::hardware::{HardwareReport, HdrtWarning, Section};

pub fn render_report(report: &HardwareReport, section: Section) -> Result<String> {
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
