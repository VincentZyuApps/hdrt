use crate::collector::BenchmarkReport;
use crate::emoji;
use crate::hardware::{CapabilityReport, HardwareReport, HdrtWarning, Section};
use crate::i18n::{display_optional, display_value, t, Lang};

pub fn render_report(report: &HardwareReport, section: Section, lang: Lang, emoji: bool) -> String {
    let mut output = Vec::new();

    if matches!(section, Section::Disk | Section::All) {
        output.push(render_disks(report, lang, emoji));
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

    let warnings = collect_warnings(report, section);
    if !warnings.is_empty() {
        output.push(render_warnings(&warnings, lang, emoji));
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

fn render_disks(report: &HardwareReport, lang: Lang, emoji: bool) -> String {
    let mut lines = vec![
        format!("## {}", label(lang, "section.disk", emoji)),
        String::new(),
        format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            label(lang, "disk.device", emoji),
            label(lang, "disk.model", emoji),
            label(lang, "disk.brand", emoji),
            label(lang, "disk.serial", emoji),
            label(lang, "disk.size", emoji),
            label(lang, "disk.kind", emoji),
            label(lang, "disk.bus", emoji),
            label(lang, "disk.firmware", emoji),
            label(lang, "disk.health", emoji)
        ),
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- |".to_string(),
    ];

    for disk in &report.disks {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            value(&disk.device, lang),
            value(&disk.model, lang),
            value(&disk.brand, lang),
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
            label(lang, "motherboard.serial", emoji),
            value(&board.serial, lang)
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
