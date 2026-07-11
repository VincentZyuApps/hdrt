use std::collections::BTreeMap;

use crate::collector::BenchmarkReport;
use crate::emoji;
use crate::hardware::{CapabilityReport, HardwareReport, HdrtWarning, Section};
use crate::i18n::{display_optional, display_value, t, Lang};

use super::warnings;

pub fn render_report(
    report: &HardwareReport,
    section: Section,
    debug_requested: bool,
    lang: Lang,
    emoji: bool,
) -> String {
    let mut output = Vec::new();

    if matches!(section, Section::Disk) {
        output.push(t(lang, "disk.combined_hint").to_string());
    }
    if matches!(
        section,
        Section::Disk | Section::PhysicalDisk | Section::All
    ) {
        output.push(render_physical_disks(report, lang, emoji));
    }
    if matches!(
        section,
        Section::Disk | Section::LogicalDisk | Section::All
    ) {
        output.push(render_logical_disks(report, lang, emoji));
    }
    if matches!(section, Section::Memory | Section::All) {
        output.push(render_memory(report, lang, emoji));
    }
    if matches!(section, Section::Cpu | Section::All) {
        output.push(render_cpu(report, lang, emoji));
    }
    if matches!(section, Section::Motherboard | Section::All) {
        output.push(render_motherboard(report, lang, emoji));
    }

    let warnings = warnings::collect(report, section);
    if !warnings.is_empty() {
        output.push(render_warnings(&warnings, lang, emoji));
    }

    if debug_requested || !report.debug.is_empty() {
        output.push(render_debug(report, section, lang, emoji));
    }

    output.join("\n\n")
}

pub fn render_capabilities(report: &CapabilityReport, lang: Lang, emoji: bool) -> String {
    let mut lines = vec![
        format!(
            "# {}",
            emoji::decorate(emoji, "doctor.title", "hdrt doctor")
        ),
        String::new(),
        format!(
            "- {}: `{}`",
            label(lang, "platform", emoji),
            report.platform
        ),
        format!("- {}: `{}`", label(lang, "arch", emoji), report.arch),
        format!(
            "- {}: `{}`",
            label(lang, "elevated", emoji),
            yes_no(report.elevated, lang)
        ),
        String::new(),
        format!(
            "| {} | {} | {} | {} |",
            label(lang, "doctor.name", emoji),
            label(lang, "doctor.available", emoji),
            label(lang, "doctor.path", emoji),
            label(lang, "doctor.purpose", emoji)
        ),
        "| --- | --- | --- | --- |".to_string(),
    ];

    for tool in &report.tools {
        lines.push(format!(
            "| {} | {} | {} | {} |",
            tool.name,
            yes_no(tool.available, lang),
            tool.path
                .as_deref()
                .map(|path| value(path, lang))
                .unwrap_or_else(|| t(lang, "unknown").to_string()),
            tool.purpose
        ));
    }

    if !report.notes.is_empty() {
        lines.push(String::new());
        lines.push(format!("## {}", label(lang, "notes", emoji)));
        for note in &report.notes {
            lines.push(format!("- {note}"));
        }
    }

    lines.join("\n")
}

pub fn render_benchmarks(report: &BenchmarkReport, lang: Lang, emoji: bool) -> String {
    let mut lines = vec![
        format!(
            "# {}",
            emoji::decorate(emoji, "bench.title", "hdrt backend benchmark")
        ),
        String::new(),
        format!(
            "- {}: `{}`",
            label(lang, "platform", emoji),
            report.platform
        ),
        format!("- {}: `{}`", label(lang, "arch", emoji), report.arch),
        String::new(),
        format!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            label(lang, "bench.backend", emoji),
            label(lang, "bench.ok", emoji),
            label(lang, "bench.elapsed", emoji),
            label(lang, "bench.disks", emoji),
            label(lang, "bench.memory", emoji),
            label(lang, "bench.warnings", emoji),
            label(lang, "bench.note", emoji)
        ),
        "| --- | --- | --- | --- | --- | --- | --- |".to_string(),
    ];

    for row in &report.rows {
        lines.push(format!(
            "| {} | {} | {} ms | {} | {} | {} | {} |",
            row.backend,
            yes_no(row.ok, lang),
            row.elapsed_ms,
            row.disks,
            row.memory,
            row.warnings,
            row.note
        ));
    }

    lines.join("\n")
}

fn render_physical_disks(report: &HardwareReport, lang: Lang, emoji: bool) -> String {
    let mut lines = vec![
        format!("## {}", label(lang, "section.physical_disk", emoji)),
        String::new(),
        format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            label(lang, "disk.device", emoji),
            label(lang, "disk.model", emoji),
            label(lang, "disk.serial", emoji),
            label(lang, "disk.size", emoji),
            label(lang, "disk.kind", emoji),
            label(lang, "disk.bus", emoji),
            label(lang, "disk.firmware", emoji),
            label(lang, "disk.health", emoji)
        ),
        "| --- | --- | --- | --- | --- | --- | --- | --- |".to_string(),
    ];

    for disk in &report.physical_disks {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            value(&disk.device, lang),
            value(&disk.model, lang),
            value(&disk.serial, lang),
            value(&disk.size, lang),
            value(&disk.media_type, lang),
            value(&disk.bus, lang),
            value(&disk.firmware, lang),
            value(&disk.health, lang)
        ));
    }

    lines.join("\n")
}

fn render_logical_disks(report: &HardwareReport, lang: Lang, emoji: bool) -> String {
    let mut lines = vec![
        format!("## {}", label(lang, "section.logical_disk", emoji)),
        String::new(),
        format!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            label(lang, "disk.device", emoji),
            label(lang, "disk.mount", emoji),
            label(lang, "disk.filesystem", emoji),
            label(lang, "disk.size", emoji),
            label(lang, "disk.used", emoji),
            label(lang, "disk.available", emoji),
            label(lang, "disk.used_percent", emoji)
        ),
        "| --- | --- | --- | --- | --- | --- | --- |".to_string(),
    ];

    for disk in &report.logical_disks {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {:.1}% |",
            value(&disk.device, lang),
            value(&disk.mount_point, lang),
            value(&disk.file_system, lang),
            value(&disk.total, lang),
            value(&disk.used, lang),
            value(&disk.available, lang),
            disk.used_percent
        ));
    }

    lines.join("\n")
}

fn render_memory(report: &HardwareReport, lang: Lang, emoji: bool) -> String {
    let mut lines = vec![
        format!("## {}", label(lang, "section.memory", emoji)),
        String::new(),
        format!(
            "| {} | {} | {} | {} | {} | {} |",
            label(lang, "memory.slot", emoji),
            label(lang, "memory.size", emoji),
            label(lang, "memory.speed", emoji),
            label(lang, "memory.manufacturer", emoji),
            label(lang, "memory.part_number", emoji),
            label(lang, "memory.serial", emoji)
        ),
        "| --- | --- | --- | --- | --- | --- |".to_string(),
    ];

    for memory in &report.memory {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} |",
            value(&memory.slot, lang),
            value(&memory.size, lang),
            value(&memory.speed, lang),
            value(&memory.manufacturer, lang),
            value(&memory.part_number, lang),
            value(&memory.serial, lang)
        ));
    }

    lines.join("\n")
}

fn render_cpu(report: &HardwareReport, lang: Lang, emoji: bool) -> String {
    let Some(cpu) = &report.cpu else {
        return format!(
            "## {}\n\n{}",
            label(lang, "section.cpu", emoji),
            t(lang, "no_data")
        );
    };

    [
        format!("## {}", label(lang, "section.cpu", emoji)),
        String::new(),
        format!(
            "- {}: `{}`",
            label(lang, "cpu.model", emoji),
            value(&cpu.model, lang)
        ),
        format!(
            "- {}: `{}`",
            label(lang, "cpu.vendor", emoji),
            value(&cpu.vendor, lang)
        ),
        format!(
            "- {}: `{}`",
            label(lang, "cpu.physical_cores", emoji),
            display_optional(lang, cpu.physical_cores)
        ),
        format!(
            "- {}: `{}`",
            label(lang, "cpu.logical_threads", emoji),
            display_optional(lang, cpu.logical_threads)
        ),
        format!(
            "- {}: `{}`",
            label(lang, "cpu.frequency", emoji),
            value(&cpu.frequency, lang)
        ),
    ]
    .join("\n")
}

fn render_motherboard(report: &HardwareReport, lang: Lang, emoji: bool) -> String {
    let Some(board) = &report.motherboard else {
        return format!(
            "## {}\n\n{}",
            label(lang, "section.motherboard", emoji),
            t(lang, "no_data")
        );
    };

    [
        format!("## {}", label(lang, "section.motherboard", emoji)),
        String::new(),
        format!(
            "- {}: `{}`",
            label(lang, "motherboard.manufacturer", emoji),
            value(&board.manufacturer, lang)
        ),
        format!(
            "- {}: `{}`",
            label(lang, "motherboard.product", emoji),
            value(&board.product, lang)
        ),
        format!(
            "- {}: `{}`",
            label(lang, "motherboard.version", emoji),
            value(&board.version, lang)
        ),
        format!(
            "- {}: `{}`",
            label(lang, "motherboard.bios_vendor", emoji),
            value(&board.bios_vendor, lang)
        ),
        format!(
            "- {}: `{}`",
            label(lang, "motherboard.bios_version", emoji),
            value(&board.bios_version, lang)
        ),
    ]
    .join("\n")
}

fn value(value: &str, lang: Lang) -> String {
    display_value(lang, value)
}

fn yes_no(value: bool, lang: Lang) -> String {
    t(lang, if value { "yes" } else { "no" }).to_string()
}

fn render_warnings(warnings: &[HdrtWarning], lang: Lang, emoji: bool) -> String {
    let mut lines = vec![
        format!("## {}", label(lang, "warnings", emoji)),
        String::new(),
    ];

    for warning in warnings {
        lines.push(format!("- `{}`: {}", warning.code, warning.message));
        if let Some(hint) = &warning.hint {
            lines.push(format!("  - {}: {hint}", label(lang, "hint", emoji)));
        }
    }

    lines.join("\n")
}

fn render_debug(report: &HardwareReport, section: Section, lang: Lang, emoji: bool) -> String {
    let mut lines = vec![
        format!("## {}", label(lang, "debug.summary", emoji)),
        String::new(),
        format!("### {}", label(lang, "debug.collector_summary", emoji)),
    ];

    let mut counts = BTreeMap::<String, usize>::new();
    for record in &report.debug {
        *counts.entry(record.source.clone()).or_default() += 1;
    }
    if counts.is_empty() {
        lines.push(format!("- {}", t(lang, "debug.no_records")));
    } else {
        for (source, count) in counts {
            lines.push(format!(
                "- `{source}`: {} {}",
                count,
                t(lang, "debug.records_count")
            ));
        }
    }

    lines.push(String::new());
    lines.push(format!("### {}", label(lang, "debug.hidden_fields", emoji)));
    let hidden_start = lines.len();
    if matches!(section, Section::Disk | Section::LogicalDisk | Section::All) {
        for disk in &report.logical_disks {
            let label = if disk.mount_point.trim().is_empty() {
                &disk.device
            } else {
                &disk.mount_point
            };
            lines.push(format!(
                "- `logical_disk.source[{label}]`: `{}`",
                value(&disk.source, lang)
            ));
        }
    }
    if matches!(section, Section::Motherboard | Section::All) {
        if let Some(board) = &report.motherboard {
            lines.push(format!(
                "- `motherboard.serial`: `{}`",
                value(&board.serial, lang)
            ));
        }
    }
    if lines.len() == hidden_start {
        lines.push(format!("- {}", t(lang, "debug.none")));
    }

    lines.push(String::new());
    lines.push(format!("## {}", label(lang, "debug.records", emoji)));
    if report.debug.is_empty() {
        lines.push(format!("- {}", t(lang, "debug.no_records")));
        return lines.join("\n");
    }

    for (index, record) in report.debug.iter().enumerate() {
        lines.push(String::new());
        lines.push(format!("### {}. {}", index + 1, record.target));
        lines.push(String::new());
        lines.push(format!("- source: `{}`", record.source));

        if let Some(note) = record.note.as_deref().filter(|note| !note.is_empty()) {
            lines.push(format!("- note: {note}"));
        }

        if record.fields.is_empty() {
            lines.push(format!("- fields: {}", t(lang, "debug.none")));
        } else {
            lines.push("- fields:".to_string());
            for (key, value) in &record.fields {
                lines.push(format!("  - `{key}`: `{value}`"));
            }
        }
    }

    lines.join("\n")
}

fn label(lang: Lang, key: &str, enabled: bool) -> String {
    emoji::decorate(enabled, key, t(lang, key))
}
