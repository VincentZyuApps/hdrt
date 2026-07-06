use crate::collector::BenchmarkReport;
use crate::hardware::{CapabilityReport, HardwareReport, HdrtWarning, Section};

pub fn render_report(report: &HardwareReport, section: Section) -> String {
    let mut output = Vec::new();

    if matches!(section, Section::Disk | Section::All) {
        output.push(render_disks(report));
    }
    if matches!(section, Section::Memory | Section::All) {
        output.push(render_memory(report));
    }
    if matches!(section, Section::Cpu | Section::All) {
        output.push(render_cpu(report));
    }
    if matches!(section, Section::Motherboard | Section::All) {
        output.push(render_motherboard(report));
    }

    let warnings = collect_warnings(report, section);
    if !warnings.is_empty() {
        output.push(render_warnings(&warnings));
    }

    output.join("\n\n")
}

pub fn render_capabilities(report: &CapabilityReport) -> String {
    let mut lines = vec![
        "# hdrt doctor".to_string(),
        String::new(),
        format!("- Platform: `{}`", report.platform),
        format!("- Arch: `{}`", report.arch),
        format!("- Elevated: `{}`", report.elevated),
        String::new(),
        "| Tool | Available | Path | Purpose |".to_string(),
        "| --- | --- | --- | --- |".to_string(),
    ];

    for tool in &report.tools {
        lines.push(format!(
            "| {} | {} | {} | {} |",
            tool.name,
            if tool.available { "yes" } else { "no" },
            tool.path.as_deref().unwrap_or("Unknown"),
            tool.purpose
        ));
    }

    if !report.notes.is_empty() {
        lines.push(String::new());
        lines.push("## Notes".to_string());
        for note in &report.notes {
            lines.push(format!("- {note}"));
        }
    }

    lines.join("\n")
}

pub fn render_benchmarks(report: &BenchmarkReport) -> String {
    let mut lines = vec![
        "# hdrt backend benchmark".to_string(),
        String::new(),
        format!("- Platform: `{}`", report.platform),
        format!("- Arch: `{}`", report.arch),
        String::new(),
        "| Backend | OK | Elapsed | Disks | Memory | Warnings | Note |".to_string(),
        "| --- | --- | --- | --- | --- | --- | --- |".to_string(),
    ];

    for row in &report.rows {
        lines.push(format!(
            "| {} | {} | {} ms | {} | {} | {} | {} |",
            row.backend,
            if row.ok { "yes" } else { "no" },
            row.elapsed_ms,
            row.disks,
            row.memory,
            row.warnings,
            row.note
        ));
    }

    lines.join("\n")
}

fn render_disks(report: &HardwareReport) -> String {
    let mut lines = vec![
        "## Disk".to_string(),
        String::new(),
        "| Device | Model | Brand | Serial | Size | Type | Bus | Firmware | Health |".to_string(),
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- |".to_string(),
    ];

    for disk in &report.disks {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            disk.device,
            disk.model,
            disk.brand,
            disk.serial,
            disk.size,
            disk.media_type,
            disk.bus,
            disk.firmware,
            disk.health
        ));
    }

    lines.join("\n")
}

fn render_memory(report: &HardwareReport) -> String {
    let mut lines = vec![
        "## Memory".to_string(),
        String::new(),
        "| Slot | Size | Speed | Manufacturer | Part Number | Serial |".to_string(),
        "| --- | --- | --- | --- | --- | --- |".to_string(),
    ];

    for memory in &report.memory {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} |",
            memory.slot,
            memory.size,
            memory.speed,
            memory.manufacturer,
            memory.part_number,
            memory.serial
        ));
    }

    lines.join("\n")
}

fn render_cpu(report: &HardwareReport) -> String {
    let Some(cpu) = &report.cpu else {
        return "## CPU\n\nNo data collected.".to_string();
    };

    [
        "## CPU".to_string(),
        String::new(),
        format!("- Model: `{}`", cpu.model),
        format!("- Vendor: `{}`", cpu.vendor),
        format!(
            "- Physical cores: `{}`",
            cpu.physical_cores
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        ),
        format!(
            "- Logical threads: `{}`",
            cpu.logical_threads
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        ),
        format!("- Frequency: `{}`", cpu.frequency),
    ]
    .join("\n")
}

fn render_motherboard(report: &HardwareReport) -> String {
    let Some(board) = &report.motherboard else {
        return "## Motherboard\n\nNo data collected.".to_string();
    };

    [
        "## Motherboard".to_string(),
        String::new(),
        format!("- Manufacturer: `{}`", board.manufacturer),
        format!("- Product: `{}`", board.product),
        format!("- Version: `{}`", board.version),
        format!("- Serial: `{}`", board.serial),
        format!("- BIOS vendor: `{}`", board.bios_vendor),
        format!("- BIOS version: `{}`", board.bios_version),
    ]
    .join("\n")
}

fn render_warnings(warnings: &[HdrtWarning]) -> String {
    let mut lines = vec!["## Warnings".to_string(), String::new()];

    for warning in warnings {
        lines.push(format!("- `{}`: {}", warning.code, warning.message));
        if let Some(hint) = &warning.hint {
            lines.push(format!("  - Hint: {hint}"));
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
