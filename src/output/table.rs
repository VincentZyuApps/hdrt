use tabled::builder::Builder;
use tabled::settings::Style;

use crate::app::options::OutputFormat;
use crate::collector::BenchmarkReport;
use crate::emoji;
use crate::hardware::{CapabilityReport, HardwareReport, HdrtWarning, Section};
use crate::i18n::{display_optional, display_value, t, Lang};

pub fn render_report(
    report: &HardwareReport,
    section: Section,
    format: OutputFormat,
    lang: Lang,
    emoji: bool,
) -> String {
    let mut output = Vec::new();

    if matches!(section, Section::Disk | Section::All) {
        output.push(render_disks(report, format, lang, emoji));
    }
    if matches!(section, Section::Memory | Section::All) {
        output.push(render_memory(report, format, lang, emoji));
    }
    if matches!(section, Section::Cpu | Section::All) {
        output.push(render_cpu(report, lang, emoji));
    }
    if matches!(section, Section::Motherboard | Section::All) {
        output.push(render_motherboard(report, lang, emoji));
    }

    let warnings = collect_warnings(report, section);
    if !warnings.is_empty() {
        output.push(render_warnings(&warnings, lang, emoji));
    }

    if !report.debug.is_empty() {
        output.push(render_debug(&report.debug));
    }

    output.join("\n\n")
}

pub fn render_capabilities(report: &CapabilityReport, lang: Lang, emoji: bool) -> String {
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
            OutputFormat::Table,
        ),
    ];

    if !report.notes.is_empty() {
        output.push(format!("{}:", label(lang, "notes", emoji)));
        output.extend(report.notes.iter().map(|note| format!("- {note}")));
    }

    output.join("\n")
}

pub fn render_benchmarks(report: &BenchmarkReport, lang: Lang, emoji: bool) -> String {
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
            OutputFormat::Table,
        ),
    ]
    .join("\n")
}

fn render_disks(report: &HardwareReport, format: OutputFormat, lang: Lang, emoji: bool) -> String {
    let rows: Vec<Vec<String>> = report
        .disks
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
        "section.disk",
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
        format,
        lang,
        emoji,
    )
}

fn render_memory(report: &HardwareReport, format: OutputFormat, lang: Lang, emoji: bool) -> String {
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
        format,
        lang,
        emoji,
    )
}

fn render_cpu(report: &HardwareReport, lang: Lang, emoji: bool) -> String {
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
        OutputFormat::Table,
        lang,
        emoji,
    )
}

fn render_motherboard(report: &HardwareReport, lang: Lang, emoji: bool) -> String {
    let rows: Vec<Vec<String>> = report
        .motherboard
        .iter()
        .map(|board| {
            vec![
                value(&board.manufacturer, lang),
                value(&board.product, lang),
                value(&board.version, lang),
                value(&board.serial, lang),
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
                "motherboard.serial",
                "motherboard.bios_vendor",
                "motherboard.bios_version",
            ],
            lang,
        ),
        rows,
        OutputFormat::Table,
        lang,
        emoji,
    )
}

fn section_with_table(
    title_key: &str,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    format: OutputFormat,
    lang: Lang,
    emoji: bool,
) -> String {
    let title = label(lang, title_key, emoji);
    if rows.is_empty() {
        return format!("{title}\n{}", t(lang, "no_data"));
    }

    format!("{title}\n{}", make_table(headers, rows, format))
}

fn make_table(headers: Vec<String>, rows: Vec<Vec<String>>, format: OutputFormat) -> String {
    let mut builder = Builder::default();
    builder.push_record(headers);
    for row in rows {
        builder.push_record(row);
    }

    let mut table = builder.build();
    match format {
        OutputFormat::Compact => table.with(Style::modern()),
        _ => table.with(Style::rounded()),
    };
    table.to_string()
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

fn render_warnings(warnings: &[HdrtWarning], lang: Lang, emoji: bool) -> String {
    let mut lines = vec![format!("{}:", label(lang, "warnings", emoji))];
    for warning in warnings {
        lines.push(format!("- [{}] {}", warning.code, warning.message));
        if let Some(hint) = &warning.hint {
            lines.push(format!("  {}: {hint}", label(lang, "hint", emoji)));
        }
    }
    lines.join("\n")
}

fn render_debug(records: &[crate::hardware::DebugRecord]) -> String {
    let mut lines = vec!["Debug".to_string()];

    for (index, record) in records.iter().enumerate() {
        lines.push(String::new());
        lines.push(format!("[{}] {}", index + 1, record.target));
        lines.push(format!("  source: {}", record.source));

        if let Some(note) = record.note.as_deref().filter(|note| !note.is_empty()) {
            lines.push(format!("  note: {note}"));
        }

        if record.fields.is_empty() {
            lines.push("  fields: none".to_string());
        } else {
            lines.push("  fields:".to_string());
            for (key, value) in &record.fields {
                lines.push(format!("    {key}: {value}"));
            }
        }
    }

    lines.join("\n")
}

fn label(lang: Lang, key: &str, enabled: bool) -> String {
    emoji::decorate(enabled, key, t(lang, key))
}

fn collect_warnings(report: &HardwareReport, section: Section) -> Vec<HdrtWarning> {
    let mut warnings = report.warnings.clone();

    if matches!(section, Section::Disk | Section::All) {
        warnings.extend(report.disks.iter().flat_map(|item| item.warnings.clone()));
    }
    if matches!(section, Section::Memory | Section::All) {
        warnings.extend(report.memory.iter().flat_map(|item| item.warnings.clone()));
    }
    if matches!(section, Section::Cpu | Section::All) {
        if let Some(cpu) = &report.cpu {
            warnings.extend(cpu.warnings.clone());
        }
    }
    if matches!(section, Section::Motherboard | Section::All) {
        if let Some(board) = &report.motherboard {
            warnings.extend(board.warnings.clone());
        }
    }

    warnings
}
