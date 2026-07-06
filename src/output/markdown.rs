use crate::collector::BenchmarkReport;
use crate::hardware::{CapabilityReport, HardwareReport, HdrtWarning, Section};
use crate::i18n::{display_optional, display_value, t, Lang};

pub fn render_report(report: &HardwareReport, section: Section, lang: Lang) -> String {
    let mut output = Vec::new();

    if matches!(section, Section::Disk | Section::All) {
        output.push(render_disks(report, lang));
    }
    if matches!(section, Section::Memory | Section::All) {
        output.push(render_memory(report, lang));
    }
    if matches!(section, Section::Cpu | Section::All) {
        output.push(render_cpu(report, lang));
    }
    if matches!(section, Section::Motherboard | Section::All) {
        output.push(render_motherboard(report, lang));
    }

    let warnings = collect_warnings(report, section);
    if !warnings.is_empty() {
        output.push(render_warnings(&warnings, lang));
    }

    output.join("\n\n")
}

pub fn render_capabilities(report: &CapabilityReport, lang: Lang) -> String {
    let mut lines = vec![
        "# hdrt doctor".to_string(),
        String::new(),
        format!("- {}: `{}`", t(lang, "platform"), report.platform),
        format!("- {}: `{}`", t(lang, "arch"), report.arch),
        format!("- {}: `{}`", t(lang, "elevated"), yes_no(report.elevated, lang)),
        String::new(),
        format!(
            "| {} | {} | {} | {} |",
            t(lang, "doctor.name"),
            t(lang, "doctor.available"),
            t(lang, "doctor.path"),
            t(lang, "doctor.purpose")
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
        lines.push(format!("## {}", t(lang, "notes")));
        for note in &report.notes {
            lines.push(format!("- {note}"));
        }
    }

    lines.join("\n")
}

pub fn render_benchmarks(report: &BenchmarkReport, lang: Lang) -> String {
    let mut lines = vec![
        "# hdrt backend benchmark".to_string(),
        String::new(),
        format!("- {}: `{}`", t(lang, "platform"), report.platform),
        format!("- {}: `{}`", t(lang, "arch"), report.arch),
        String::new(),
        format!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            t(lang, "bench.backend"),
            t(lang, "bench.ok"),
            t(lang, "bench.elapsed"),
            t(lang, "bench.disks"),
            t(lang, "bench.memory"),
            t(lang, "bench.warnings"),
            t(lang, "bench.note")
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

fn render_disks(report: &HardwareReport, lang: Lang) -> String {
    let mut lines = vec![
        format!("## {}", t(lang, "section.disk")),
        String::new(),
        format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            t(lang, "disk.device"),
            t(lang, "disk.model"),
            t(lang, "disk.brand"),
            t(lang, "disk.serial"),
            t(lang, "disk.size"),
            t(lang, "disk.kind"),
            t(lang, "disk.bus"),
            t(lang, "disk.firmware"),
            t(lang, "disk.health")
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

fn render_memory(report: &HardwareReport, lang: Lang) -> String {
    let mut lines = vec![
        format!("## {}", t(lang, "section.memory")),
        String::new(),
        format!(
            "| {} | {} | {} | {} | {} | {} |",
            t(lang, "memory.slot"),
            t(lang, "memory.size"),
            t(lang, "memory.speed"),
            t(lang, "memory.manufacturer"),
            t(lang, "memory.part_number"),
            t(lang, "memory.serial")
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

fn render_cpu(report: &HardwareReport, lang: Lang) -> String {
    let Some(cpu) = &report.cpu else {
        return format!("## {}\n\n{}", t(lang, "section.cpu"), t(lang, "no_data"));
    };

    [
        format!("## {}", t(lang, "section.cpu")),
        String::new(),
        format!("- {}: `{}`", t(lang, "cpu.model"), value(&cpu.model, lang)),
        format!("- {}: `{}`", t(lang, "cpu.vendor"), value(&cpu.vendor, lang)),
        format!(
            "- {}: `{}`",
            t(lang, "cpu.physical_cores"),
            display_optional(lang, cpu.physical_cores)
        ),
        format!(
            "- {}: `{}`",
            t(lang, "cpu.logical_threads"),
            display_optional(lang, cpu.logical_threads)
        ),
        format!(
            "- {}: `{}`",
            t(lang, "cpu.frequency"),
            value(&cpu.frequency, lang)
        ),
    ]
    .join("\n")
}

fn render_motherboard(report: &HardwareReport, lang: Lang) -> String {
    let Some(board) = &report.motherboard else {
        return format!(
            "## {}\n\n{}",
            t(lang, "section.motherboard"),
            t(lang, "no_data")
        );
    };

    [
        format!("## {}", t(lang, "section.motherboard")),
        String::new(),
        format!(
            "- {}: `{}`",
            t(lang, "motherboard.manufacturer"),
            value(&board.manufacturer, lang)
        ),
        format!(
            "- {}: `{}`",
            t(lang, "motherboard.product"),
            value(&board.product, lang)
        ),
        format!(
            "- {}: `{}`",
            t(lang, "motherboard.version"),
            value(&board.version, lang)
        ),
        format!(
            "- {}: `{}`",
            t(lang, "motherboard.serial"),
            value(&board.serial, lang)
        ),
        format!(
            "- {}: `{}`",
            t(lang, "motherboard.bios_vendor"),
            value(&board.bios_vendor, lang)
        ),
        format!(
            "- {}: `{}`",
            t(lang, "motherboard.bios_version"),
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

fn render_warnings(warnings: &[HdrtWarning], lang: Lang) -> String {
    let mut lines = vec![format!("## {}", t(lang, "warnings")), String::new()];

    for warning in warnings {
        lines.push(format!("- `{}`: {}", warning.code, warning.message));
        if let Some(hint) = &warning.hint {
            lines.push(format!("  - {}: {hint}", t(lang, "hint")));
        }
    }

    lines.join("\n")
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
