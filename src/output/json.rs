use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

use crate::collector::BenchmarkReport;
use crate::emoji;
use crate::hardware::{CapabilityReport, HardwareReport, HdrtWarning, Section};
use crate::i18n::{t, Lang};

pub fn render_report(
    report: &HardwareReport,
    section: Section,
    lang: Lang,
    emoji: bool,
) -> Result<String> {
    #[derive(Serialize)]
    struct SelectedReport<'a, T> {
        data: T,
        warnings: &'a [HdrtWarning],
        #[serde(skip_serializing_if = "Option::is_none")]
        debug: Option<&'a Vec<crate::hardware::DebugRecord>>,
    }

    let debug = (!report.debug.is_empty()).then_some(&report.debug);
    let value = match section {
        Section::Disk => serde_json::to_value(SelectedReport {
            data: &report.disks,
            warnings: &report.warnings,
            debug,
        })?,
        Section::Memory => serde_json::to_value(SelectedReport {
            data: &report.memory,
            warnings: &report.warnings,
            debug,
        })?,
        Section::Cpu => serde_json::to_value(SelectedReport {
            data: &report.cpu,
            warnings: &report.warnings,
            debug,
        })?,
        Section::Motherboard => serde_json::to_value(SelectedReport {
            data: &report.motherboard,
            warnings: &report.warnings,
            debug,
        })?,
        Section::All => serde_json::to_value(report)?,
    };

    if emoji {
        return render_decorated(
            section_title(section, lang, emoji),
            labels(report_label_keys(section), lang, emoji),
            value,
        );
    }

    Ok(serde_json::to_string_pretty(&value)?)
}

pub fn render_capabilities(report: &CapabilityReport, lang: Lang, emoji: bool) -> Result<String> {
    let value = serde_json::to_value(report)?;
    if emoji {
        return render_decorated(
            emoji::decorate(true, "doctor.title", "hdrt doctor"),
            labels(
                &[
                    "platform",
                    "arch",
                    "elevated",
                    "notes",
                    "doctor.name",
                    "doctor.available",
                    "doctor.path",
                    "doctor.purpose",
                ],
                lang,
                emoji,
            ),
            value,
        );
    }

    Ok(serde_json::to_string_pretty(&value)?)
}

pub fn render_benchmarks(report: &BenchmarkReport, lang: Lang, emoji: bool) -> Result<String> {
    let value = serde_json::to_value(report)?;
    if emoji {
        return render_decorated(
            emoji::decorate(true, "bench.title", "hdrt backend benchmark"),
            labels(
                &[
                    "platform",
                    "arch",
                    "bench.backend",
                    "bench.ok",
                    "bench.elapsed",
                    "bench.disks",
                    "bench.memory",
                    "bench.warnings",
                    "bench.note",
                ],
                lang,
                emoji,
            ),
            value,
        );
    }

    Ok(serde_json::to_string_pretty(&value)?)
}

#[derive(Serialize)]
struct DecoratedJson {
    title: String,
    labels: BTreeMap<&'static str, String>,
    data: Value,
}

fn render_decorated(
    title: String,
    labels: BTreeMap<&'static str, String>,
    data: Value,
) -> Result<String> {
    Ok(serde_json::to_string_pretty(&DecoratedJson {
        title,
        labels,
        data,
    })?)
}

fn section_title(section: Section, lang: Lang, emoji: bool) -> String {
    let key = match section {
        Section::Disk => "section.disk",
        Section::Memory => "section.memory",
        Section::Cpu => "section.cpu",
        Section::Motherboard => "section.motherboard",
        Section::All => "section.all",
    };
    emoji::decorate(emoji, key, t(lang, key))
}

fn report_label_keys(section: Section) -> &'static [&'static str] {
    match section {
        Section::Disk => &[
            "section.disk",
            "disk.device",
            "disk.model",
            "disk.brand",
            "disk.serial",
            "disk.size",
            "disk.kind",
            "disk.bus",
            "disk.firmware",
            "disk.health",
            "warnings",
            "hint",
        ],
        Section::Memory => &[
            "section.memory",
            "memory.slot",
            "memory.size",
            "memory.speed",
            "memory.manufacturer",
            "memory.part_number",
            "memory.serial",
            "warnings",
            "hint",
        ],
        Section::Cpu => &[
            "section.cpu",
            "cpu.model",
            "cpu.vendor",
            "cpu.physical_cores",
            "cpu.logical_threads",
            "cpu.frequency",
            "warnings",
            "hint",
        ],
        Section::Motherboard => &[
            "section.motherboard",
            "motherboard.manufacturer",
            "motherboard.product",
            "motherboard.version",
            "motherboard.serial",
            "motherboard.bios_vendor",
            "motherboard.bios_version",
            "warnings",
            "hint",
        ],
        Section::All => &[
            "section.all",
            "section.disk",
            "section.memory",
            "section.cpu",
            "section.motherboard",
            "disk.device",
            "disk.model",
            "disk.brand",
            "disk.serial",
            "disk.size",
            "disk.kind",
            "disk.bus",
            "disk.firmware",
            "disk.health",
            "memory.slot",
            "memory.size",
            "memory.speed",
            "memory.manufacturer",
            "memory.part_number",
            "memory.serial",
            "cpu.model",
            "cpu.vendor",
            "cpu.physical_cores",
            "cpu.logical_threads",
            "cpu.frequency",
            "motherboard.manufacturer",
            "motherboard.product",
            "motherboard.version",
            "motherboard.serial",
            "motherboard.bios_vendor",
            "motherboard.bios_version",
            "warnings",
            "hint",
        ],
    }
}

fn labels(keys: &[&'static str], lang: Lang, emoji: bool) -> BTreeMap<&'static str, String> {
    keys.iter()
        .map(|key| (*key, emoji::decorate(emoji, key, t(lang, key))))
        .collect()
}
