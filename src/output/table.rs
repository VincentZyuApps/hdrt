use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::app::options::OutputFormat;
use crate::collector::BenchmarkReport;
use crate::hardware::{CapabilityReport, HardwareReport, HdrtWarning, Section};

#[derive(Tabled)]
struct DiskRow {
    device: String,
    model: String,
    brand: String,
    serial: String,
    size: String,
    kind: String,
    bus: String,
    firmware: String,
    health: String,
}

#[derive(Tabled)]
struct MemoryRow {
    slot: String,
    size: String,
    speed: String,
    manufacturer: String,
    part_number: String,
    serial: String,
}

#[derive(Tabled)]
struct CpuRow {
    model: String,
    vendor: String,
    physical_cores: String,
    logical_threads: String,
    frequency: String,
}

#[derive(Tabled)]
struct MotherboardRow {
    manufacturer: String,
    product: String,
    version: String,
    serial: String,
    bios_vendor: String,
    bios_version: String,
}

#[derive(Tabled)]
struct ToolRow {
    name: String,
    available: String,
    path: String,
    purpose: String,
}

#[derive(Tabled)]
struct BenchmarkTableRow {
    backend: String,
    ok: String,
    elapsed: String,
    disks: usize,
    memory: usize,
    warnings: usize,
    note: String,
}

pub fn render_report(report: &HardwareReport, section: Section, format: OutputFormat) -> String {
    let mut output = Vec::new();

    if matches!(section, Section::Disk | Section::All) {
        output.push(render_disks(report, format));
    }
    if matches!(section, Section::Memory | Section::All) {
        output.push(render_memory(report, format));
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
    let rows: Vec<ToolRow> = report
        .tools
        .iter()
        .map(|tool| ToolRow {
            name: tool.name.clone(),
            available: if tool.available { "yes" } else { "no" }.to_string(),
            path: tool.path.clone().unwrap_or_else(|| "Unknown".to_string()),
            purpose: tool.purpose.clone(),
        })
        .collect();

    let mut output = vec![
        format!("Platform: {} / {}", report.platform, report.arch),
        format!("Elevated: {}", if report.elevated { "yes" } else { "no" }),
        make_table(rows),
    ];

    if !report.notes.is_empty() {
        output.push("Notes:".to_string());
        output.extend(report.notes.iter().map(|note| format!("- {note}")));
    }

    output.join("\n")
}

pub fn render_benchmarks(report: &BenchmarkReport) -> String {
    let rows: Vec<BenchmarkTableRow> = report
        .rows
        .iter()
        .map(|row| BenchmarkTableRow {
            backend: row.backend.clone(),
            ok: if row.ok { "yes" } else { "no" }.to_string(),
            elapsed: format!("{} ms", row.elapsed_ms),
            disks: row.disks,
            memory: row.memory,
            warnings: row.warnings,
            note: row.note.clone(),
        })
        .collect();

    [
        format!("Platform: {} / {}", report.platform, report.arch),
        make_table(rows),
    ]
    .join("\n")
}

fn render_disks(report: &HardwareReport, format: OutputFormat) -> String {
    let rows: Vec<DiskRow> = report
        .disks
        .iter()
        .map(|disk| DiskRow {
            device: disk.device.clone(),
            model: disk.model.clone(),
            brand: disk.brand.clone(),
            serial: disk.serial.clone(),
            size: disk.size.clone(),
            kind: disk.media_type.clone(),
            bus: disk.bus.clone(),
            firmware: disk.firmware.clone(),
            health: disk.health.clone(),
        })
        .collect();

    section_with_table("Disk", rows, format)
}

fn render_memory(report: &HardwareReport, format: OutputFormat) -> String {
    let rows: Vec<MemoryRow> = report
        .memory
        .iter()
        .map(|memory| MemoryRow {
            slot: memory.slot.clone(),
            size: memory.size.clone(),
            speed: memory.speed.clone(),
            manufacturer: memory.manufacturer.clone(),
            part_number: memory.part_number.clone(),
            serial: memory.serial.clone(),
        })
        .collect();

    section_with_table("Memory", rows, format)
}

fn render_cpu(report: &HardwareReport) -> String {
    let rows: Vec<CpuRow> = report
        .cpu
        .iter()
        .map(|cpu| CpuRow {
            model: cpu.model.clone(),
            vendor: cpu.vendor.clone(),
            physical_cores: cpu
                .physical_cores
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            logical_threads: cpu
                .logical_threads
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            frequency: cpu.frequency.clone(),
        })
        .collect();

    section_with_table("CPU", rows, OutputFormat::Table)
}

fn render_motherboard(report: &HardwareReport) -> String {
    let rows: Vec<MotherboardRow> = report
        .motherboard
        .iter()
        .map(|board| MotherboardRow {
            manufacturer: board.manufacturer.clone(),
            product: board.product.clone(),
            version: board.version.clone(),
            serial: board.serial.clone(),
            bios_vendor: board.bios_vendor.clone(),
            bios_version: board.bios_version.clone(),
        })
        .collect();

    section_with_table("Motherboard", rows, OutputFormat::Table)
}

fn section_with_table<T>(title: &str, rows: Vec<T>, format: OutputFormat) -> String
where
    T: Tabled,
{
    if rows.is_empty() {
        return format!("{title}\nNo data collected.");
    }

    let table = match format {
        OutputFormat::Compact => make_compact_table(rows),
        _ => make_table(rows),
    };

    format!("{title}\n{table}")
}

fn make_table<T: Tabled>(rows: Vec<T>) -> String {
    let mut table = Table::new(rows);
    table.with(Style::rounded());
    table.to_string()
}

fn make_compact_table<T: Tabled>(rows: Vec<T>) -> String {
    let mut table = Table::new(rows);
    table.with(Style::modern());
    table.to_string()
}

fn render_warnings(warnings: &[HdrtWarning]) -> String {
    let mut lines = vec!["Warnings:".to_string()];
    for warning in warnings {
        lines.push(format!("- [{}] {}", warning.code, warning.message));
        if let Some(hint) = &warning.hint {
            lines.push(format!("  hint: {hint}"));
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
