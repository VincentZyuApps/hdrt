use std::collections::HashSet;

use crate::app::options::{RenderFormat, TableStyle};
use crate::hardware::{
    CpuInfo, DebugRecord, DiskInfo, HardwareReport, HdrtWarning, LogicalDiskInfo, MemoryDevice,
    MotherboardInfo, Section,
};
use crate::i18n::Lang;

use super::render_report;

fn fixture_report() -> HardwareReport {
    HardwareReport {
        physical_disks: vec![DiskInfo {
            device: "0".to_string(),
            model: "Fixture SSD".to_string(),
            serial: "DISK-SERIAL".to_string(),
            size: "512.00 GiB".to_string(),
            media_type: "SSD".to_string(),
            bus: "NVMe".to_string(),
            firmware: "1.0".to_string(),
            health: "Healthy".to_string(),
            source: "fixture-physical".to_string(),
            warnings: Vec::new(),
        }],
        logical_disks: vec![LogicalDiskInfo {
            device: "/dev/test1".to_string(),
            mount_point: "/fixture".to_string(),
            file_system: "ext4".to_string(),
            total: "100.00 GiB".to_string(),
            used: "25.00 GiB".to_string(),
            available: "75.00 GiB".to_string(),
            used_percent: 25.0,
            source: "logical-source".to_string(),
            warnings: Vec::new(),
        }],
        memory: vec![MemoryDevice {
            slot: "DIMM0".to_string(),
            size: "16.00 GiB".to_string(),
            speed: "3200 MT/s".to_string(),
            manufacturer: "Fixture Memory".to_string(),
            part_number: "PART-1".to_string(),
            serial: "MEM-SERIAL".to_string(),
            source: "fixture-memory".to_string(),
            warnings: Vec::new(),
        }],
        cpu: Some(fixture_cpu()),
        motherboard: Some(MotherboardInfo {
            manufacturer: "Fixture Board".to_string(),
            product: "Board 1".to_string(),
            version: "1.0".to_string(),
            serial: "BOARD-SECRET".to_string(),
            bios_vendor: "Fixture BIOS".to_string(),
            bios_version: "2.0".to_string(),
            source: "fixture-board".to_string(),
            warnings: Vec::new(),
        }),
        warnings: vec![HdrtWarning::with_hint(
            "fixture-warning",
            "Fixture warning message.",
            "Fixture warning hint.",
        )],
        debug: Vec::new(),
    }
}

fn fixture_cpu() -> CpuInfo {
    CpuInfo {
        model: "Test CPU".to_string(),
        vendor: "Test Vendor".to_string(),
        physical_cores: Some(6),
        logical_threads: Some(12),
        frequency: "2500 MHz".to_string(),
        source: "fixture".to_string(),
        warnings: Vec::new(),
    }
}

fn cpu_report(with_debug: bool) -> HardwareReport {
    HardwareReport {
        cpu: Some(fixture_cpu()),
        debug: if with_debug {
            vec![DebugRecord::new("cpu0", "fixture")
                .field("cores", "6")
                .note("deterministic fixture")]
        } else {
            Vec::new()
        },
        ..HardwareReport::default()
    }
}

fn table(style: TableStyle, color: bool, bold: bool, debug: bool) -> String {
    let mut report = fixture_report();
    if debug {
        report.debug.push(
            DebugRecord::new("cpu0", "fixture")
                .field("cores", "6")
                .note("deterministic fixture"),
        );
    }
    table_report(&report, style, color, bold, debug)
}

fn table_report(
    report: &HardwareReport,
    style: TableStyle,
    color: bool,
    bold: bool,
    debug: bool,
) -> String {
    render_report(
        report,
        Section::All,
        RenderFormat::Table,
        style,
        color,
        bold,
        debug,
        Lang::EnUs,
        false,
    )
    .expect("table rendering should succeed")
}

fn assert_golden(actual: &str, expected: &str) {
    let actual = actual.replace("\r\n", "\n");
    let expected = expected.replace("\r\n", "\n");
    assert_eq!(actual.trim_end(), expected.trim_end());
}

#[test]
fn all_table_styles_produce_distinct_output() {
    let mut report = fixture_report();
    let mut second_disk = report.physical_disks[0].clone();
    second_disk.device = "1".to_string();
    second_disk.model = "Second Fixture SSD".to_string();
    report.physical_disks.push(second_disk);

    let outputs = [
        TableStyle::Rounded,
        TableStyle::Modern,
        TableStyle::Sharp,
        TableStyle::Psql,
        TableStyle::Ascii,
        TableStyle::Blank,
    ]
    .into_iter()
    .map(|style| table_report(&report, style, false, false, false))
    .collect::<HashSet<_>>();

    assert_eq!(outputs.len(), 6);
}

#[test]
fn ascii_style_avoids_unicode_box_drawing() {
    let output = table(TableStyle::Ascii, false, false, false);

    assert!(output.contains('+'));
    assert!(!output
        .chars()
        .any(|character| ['╭', '┌', '╔', '│'].contains(&character)));
}

#[test]
fn blank_style_omits_visible_box_borders() {
    let output = table(TableStyle::Blank, false, false, false);

    assert!(!output
        .chars()
        .any(|character| ['╭', '┌', '╔', '│', '+'].contains(&character)));
    assert!(output.contains("Fixture SSD"));
}

#[test]
fn honors_color_and_bold_switches_independently() {
    let color_only = table(TableStyle::Rounded, true, false, false);
    let bold_only = table(TableStyle::Rounded, false, true, false);
    let plain = table(TableStyle::Rounded, false, false, false);

    assert!(color_only.contains("\x1b[36mdevice\x1b[0m"));
    assert!(bold_only.contains("\x1b[1mdevice\x1b[0m"));
    assert!(!plain.contains("\x1b["));
}

#[test]
fn normal_table_hides_debug_only_fields() {
    let output = table(TableStyle::Rounded, false, false, false);

    assert!(!output.contains("logical-source"));
    assert!(!output.contains("BOARD-SECRET"));
}

#[test]
fn debug_table_restores_hidden_fields_in_summary_order() {
    let output = table(TableStyle::Rounded, false, false, true);
    let collector = output.find("Collector Summary").unwrap();
    let hidden = output.find("Hidden Fields").unwrap();
    let records = output.find("Debug Records").unwrap();

    assert!(collector < hidden && hidden < records);
    assert!(output.contains("logical-source"));
    assert!(output.contains("BOARD-SECRET"));
}

#[test]
fn normal_markdown_hides_debug_only_fields() {
    let output = render_report(
        &fixture_report(),
        Section::All,
        RenderFormat::Markdown,
        TableStyle::Rounded,
        false,
        false,
        false,
        Lang::EnUs,
        false,
    )
    .unwrap();

    assert!(!output.contains("logical-source"));
    assert!(!output.contains("BOARD-SECRET"));
}

#[test]
fn json_is_valid_and_preserves_structured_fields() {
    let output = render_report(
        &fixture_report(),
        Section::All,
        RenderFormat::Json,
        TableStyle::Rounded,
        false,
        false,
        false,
        Lang::EnUs,
        false,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(value["logical_disks"][0]["source"], "logical-source");
    assert_eq!(value["motherboard"]["serial"], "BOARD-SECRET");
}

#[test]
fn markdown_snapshot_matches_fixture() {
    let output = render_report(
        &cpu_report(false),
        Section::Cpu,
        RenderFormat::Markdown,
        TableStyle::Rounded,
        false,
        false,
        false,
        Lang::EnUs,
        false,
    )
    .unwrap();

    assert_golden(&output, include_str!("snapshots/cpu_markdown.txt"));
}

#[test]
fn json_snapshot_matches_fixture() {
    let output = render_report(
        &cpu_report(false),
        Section::Cpu,
        RenderFormat::Json,
        TableStyle::Rounded,
        false,
        false,
        false,
        Lang::EnUs,
        false,
    )
    .unwrap();

    assert_golden(&output, include_str!("snapshots/cpu_json.txt"));
}

#[test]
fn debug_snapshot_matches_fixture() {
    let output = render_report(
        &cpu_report(true),
        Section::Cpu,
        RenderFormat::Markdown,
        TableStyle::Rounded,
        false,
        false,
        true,
        Lang::EnUs,
        false,
    )
    .unwrap();

    assert_golden(&output, include_str!("snapshots/cpu_debug_markdown.txt"));
}
