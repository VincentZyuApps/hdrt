use tabled::builder::Builder;
use tabled::settings::Style;

use crate::app::options::TableStyle;
use crate::collector::BenchmarkReport;
use crate::emoji;
use crate::hardware::{CapabilityReport, HardwareReport, Section};
use crate::i18n::{display_optional, display_value, t, Lang};

use super::debug;
use super::style::{style_table_header, TextStyle};
use super::warnings;

pub(super) fn render_report(
    report: &HardwareReport,
    section: Section,
    table_style: TableStyle,
    text_style: TextStyle,
    debug_requested: bool,
    lang: Lang,
    emoji: bool,
) -> String {
    let mut output = Vec::new();

    if matches!(section, Section::Disk) {
        output.push(render_disk_hint(lang));
    }
    if matches!(
        section,
        Section::Disk | Section::PhysicalDisk | Section::All
    ) {
        output.push(render_physical_disks(
            report,
            table_style,
            text_style,
            lang,
            emoji,
        ));
    }
    if matches!(section, Section::Disk | Section::LogicalDisk | Section::All) {
        output.push(render_logical_disks(
            report,
            table_style,
            text_style,
            lang,
            emoji,
        ));
    }
    if matches!(section, Section::Memory | Section::All) {
        output.push(render_memory(report, table_style, text_style, lang, emoji));
    }
    if matches!(section, Section::Cpu | Section::All) {
        output.push(render_cpu(report, table_style, text_style, lang, emoji));
    }
    if matches!(section, Section::Motherboard | Section::All) {
        output.push(render_motherboard(
            report,
            table_style,
            text_style,
            lang,
            emoji,
        ));
    }

    let warnings = warnings::collect(report, section);
    if !warnings.is_empty() {
        output.push(warnings::render(&warnings, lang, emoji, text_style));
    }

    if debug_requested || !report.debug.is_empty() {
        output.push(debug::render(report, section, lang, emoji, text_style));
    }

    output.join("\n\n")
}

pub(super) fn render_capabilities(
    report: &CapabilityReport,
    table_style: TableStyle,
    text_style: TextStyle,
    lang: Lang,
    emoji: bool,
) -> String {
    let rows: Vec<Vec<String>> = report
        .tools
        .iter()
        .map(|tool| {
            vec![
                tool.name.clone(),
                yes_no(tool.available, lang),
                tool.path
                    .as_deref()
                    .map(|path| value(path, lang))
                    .unwrap_or_else(|| t(lang, "unknown").to_string()),
                tool.purpose.clone(),
            ]
        })
        .collect();

    let mut output = vec![
        format!(
            "{}: {} / {}",
            label(lang, "platform", emoji),
            report.platform,
            report.arch
        ),
        format!(
            "{}: {}",
            label(lang, "elevated", emoji),
            yes_no(report.elevated, lang)
        ),
        make_table(
            table_headers(
                &[
                    "doctor.name",
                    "doctor.available",
                    "doctor.path",
                    "doctor.purpose",
                ],
                lang,
            ),
            rows,
            table_style,
            text_style,
        ),
    ];

    if !report.notes.is_empty() {
        output.push(format!("{}:", label(lang, "notes", emoji)));
        output.extend(report.notes.iter().map(|note| format!("- {note}")));
    }

    output.join("\n")
}

pub(super) fn render_benchmarks(
    report: &BenchmarkReport,
    table_style: TableStyle,
    text_style: TextStyle,
    lang: Lang,
    emoji: bool,
) -> String {
    let rows: Vec<Vec<String>> = report
        .rows
        .iter()
        .map(|row| {
            vec![
                row.backend.clone(),
                yes_no(row.ok, lang),
                format!("{} ms", row.elapsed_ms),
                row.disks.to_string(),
                row.memory.to_string(),
                row.warnings.to_string(),
                row.note.clone(),
            ]
        })
        .collect();

    [
        format!(
            "{}: {} / {}",
            label(lang, "platform", emoji),
            report.platform,
            report.arch
        ),
        make_table(
            table_headers(
                &[
                    "bench.backend",
                    "bench.ok",
                    "bench.elapsed",
                    "bench.disks",
                    "bench.memory",
                    "bench.warnings",
                    "bench.note",
                ],
                lang,
            ),
            rows,
            table_style,
            text_style,
        ),
    ]
    .join("\n")
}

fn render_disk_hint(lang: Lang) -> String {
    t(lang, "disk.combined_hint").to_string()
}

fn render_physical_disks(
    report: &HardwareReport,
    table_style: TableStyle,
    text_style: TextStyle,
    lang: Lang,
    emoji: bool,
) -> String {
    let rows: Vec<Vec<String>> = report
        .physical_disks
        .iter()
        .map(|disk| {
            vec![
                value(&disk.device, lang),
                value(&disk.model, lang),
                value(&disk.serial, lang),
                value(&disk.size, lang),
                value(&disk.media_type, lang),
                value(&disk.bus, lang),
                value(&disk.firmware, lang),
                value(&disk.health, lang),
            ]
        })
        .collect();

    section_with_table(
        "section.physical_disk",
        table_headers(
            &[
                "disk.device",
                "disk.model",
                "disk.serial",
                "disk.size",
                "disk.kind",
                "disk.bus",
                "disk.firmware",
                "disk.health",
            ],
            lang,
        ),
        rows,
        table_style,
        text_style,
        lang,
        emoji,
    )
}

fn render_logical_disks(
    report: &HardwareReport,
    table_style: TableStyle,
    text_style: TextStyle,
    lang: Lang,
    emoji: bool,
) -> String {
    let rows: Vec<Vec<String>> = report
        .logical_disks
        .iter()
        .map(|disk| {
            vec![
                value(&disk.device, lang),
                value(&disk.mount_point, lang),
                value(&disk.file_system, lang),
                value(&disk.total, lang),
                value(&disk.used, lang),
                value(&disk.available, lang),
                format!("{:.1}%", disk.used_percent),
            ]
        })
        .collect();

    section_with_table(
        "section.logical_disk",
        table_headers(
            &[
                "disk.device",
                "disk.mount",
                "disk.filesystem",
                "disk.size",
                "disk.used",
                "disk.available",
                "disk.used_percent",
            ],
            lang,
        ),
        rows,
        table_style,
        text_style,
        lang,
        emoji,
    )
}

fn render_memory(
    report: &HardwareReport,
    table_style: TableStyle,
    text_style: TextStyle,
    lang: Lang,
    emoji: bool,
) -> String {
    let rows: Vec<Vec<String>> = report
        .memory
        .iter()
        .map(|memory| {
            vec![
                value(&memory.slot, lang),
                value(&memory.size, lang),
                value(&memory.speed, lang),
                value(&memory.manufacturer, lang),
                value(&memory.part_number, lang),
                value(&memory.serial, lang),
            ]
        })
        .collect();

    section_with_table(
        "section.memory",
        table_headers(
            &[
                "memory.slot",
                "memory.size",
                "memory.speed",
                "memory.manufacturer",
                "memory.part_number",
                "memory.serial",
            ],
            lang,
        ),
        rows,
        table_style,
        text_style,
        lang,
        emoji,
    )
}

fn render_cpu(
    report: &HardwareReport,
    table_style: TableStyle,
    text_style: TextStyle,
    lang: Lang,
    emoji: bool,
) -> String {
    let rows: Vec<Vec<String>> = report
        .cpu
        .iter()
        .map(|cpu| {
            vec![
                value(&cpu.model, lang),
                value(&cpu.vendor, lang),
                display_optional(lang, cpu.physical_cores),
                display_optional(lang, cpu.logical_threads),
                value(&cpu.frequency, lang),
            ]
        })
        .collect();

    section_with_table(
        "section.cpu",
        table_headers(
            &[
                "cpu.model",
                "cpu.vendor",
                "cpu.physical_cores",
                "cpu.logical_threads",
                "cpu.frequency",
            ],
            lang,
        ),
        rows,
        table_style,
        text_style,
        lang,
        emoji,
    )
}

fn render_motherboard(
    report: &HardwareReport,
    table_style: TableStyle,
    text_style: TextStyle,
    lang: Lang,
    emoji: bool,
) -> String {
    let rows: Vec<Vec<String>> = report
        .motherboard
        .iter()
        .map(|board| {
            vec![
                value(&board.manufacturer, lang),
                value(&board.product, lang),
                value(&board.version, lang),
                value(&board.bios_vendor, lang),
                value(&board.bios_version, lang),
            ]
        })
        .collect();

    section_with_table(
        "section.motherboard",
        table_headers(
            &[
                "motherboard.manufacturer",
                "motherboard.product",
                "motherboard.version",
                "motherboard.bios_vendor",
                "motherboard.bios_version",
            ],
            lang,
        ),
        rows,
        table_style,
        text_style,
        lang,
        emoji,
    )
}

fn section_with_table(
    title_key: &str,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    table_style: TableStyle,
    text_style: TextStyle,
    lang: Lang,
    emoji: bool,
) -> String {
    let title = text_style.title(label(lang, title_key, emoji));
    if rows.is_empty() {
        return format!("{title}\n{}", t(lang, "no_data"));
    }

    format!(
        "{title}\n{}",
        make_table(headers, rows, table_style, text_style)
    )
}

fn make_table(
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    table_style: TableStyle,
    text_style: TextStyle,
) -> String {
    let styled_headers = headers.clone();
    let mut builder = Builder::default();
    builder.push_record(headers);
    for row in rows {
        builder.push_record(row);
    }

    let mut table = builder.build();
    match table_style {
        TableStyle::Rounded => table.with(Style::rounded()),
        TableStyle::Modern => table.with(Style::modern()),
        TableStyle::Sharp => table.with(Style::sharp()),
        TableStyle::Psql => table.with(Style::psql()),
        TableStyle::Ascii => table.with(Style::ascii()),
        TableStyle::Blank => table.with(Style::blank()),
    };
    style_table_header(table.to_string(), &styled_headers, table_style, text_style)
}

fn headers(keys: &[&str], lang: Lang, emoji: bool) -> Vec<String> {
    keys.iter().map(|key| label(lang, key, emoji)).collect()
}

fn table_headers(keys: &[&str], lang: Lang) -> Vec<String> {
    // Emoji widths vary across terminals; keeping table cells plain preserves borders.
    headers(keys, lang, false)
}

fn value(value: &str, lang: Lang) -> String {
    display_value(lang, value)
}

fn yes_no(value: bool, lang: Lang) -> String {
    t(lang, if value { "yes" } else { "no" }).to_string()
}

fn label(lang: Lang, key: &str, enabled: bool) -> String {
    emoji::decorate(enabled, key, t(lang, key))
}
