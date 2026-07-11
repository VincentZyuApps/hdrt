use std::collections::BTreeMap;

use crate::emoji;
use crate::hardware::{HardwareReport, Section};
use crate::i18n::{display_value, t, Lang};

use super::style::TextStyle;

pub(super) fn render(
    report: &HardwareReport,
    section: Section,
    lang: Lang,
    emoji_enabled: bool,
    style: TextStyle,
) -> String {
    let mut lines = Vec::new();
    lines.push(style.title(emoji::decorate(
        emoji_enabled,
        "debug.summary",
        t(lang, "debug.summary"),
    )));
    lines.push(String::new());
    push_collector_summary(&mut lines, report, lang, style);
    push_hidden_fields(&mut lines, report, section, lang, style);
    lines.push(String::new());
    lines.push(style.title(emoji::decorate(
        emoji_enabled,
        "debug.records",
        t(lang, "debug.records"),
    )));
    push_records(&mut lines, report, lang, style);
    lines.join("\n")
}

fn push_collector_summary(
    lines: &mut Vec<String>,
    report: &HardwareReport,
    lang: Lang,
    style: TextStyle,
) {
    lines.push(format!("  {}", style.header(t(lang, "debug.collector_summary"))));
    if report.debug.is_empty() {
        lines.push(format!("    {}", t(lang, "debug.no_records")));
        return;
    }

    let mut counts = BTreeMap::<String, usize>::new();
    for record in &report.debug {
        *counts.entry(record.source.clone()).or_default() += 1;
    }

    for (source, count) in counts {
        lines.push(format!(
            "    {} {}",
            style.key(format!("{source}:")),
            format_args!("{} {}", count, t(lang, "debug.records_count"))
        ));
    }
}

fn push_hidden_fields(
    lines: &mut Vec<String>,
    report: &HardwareReport,
    section: Section,
    lang: Lang,
    style: TextStyle,
) {
    let mut hidden = Vec::new();
    if matches!(section, Section::Disk | Section::LogicalDisk | Section::All)
        && !report.logical_disks.is_empty()
    {
        hidden.push(style.key("logical_disk.source:"));
        for disk in &report.logical_disks {
            let label = if disk.mount_point.trim().is_empty() {
                &disk.device
            } else {
                &disk.mount_point
            };
            hidden.push(format!(
                "      {} = {}",
                display_value(lang, label),
                display_value(lang, &disk.source)
            ));
        }
    }

    if matches!(section, Section::Motherboard | Section::All) {
        if let Some(board) = &report.motherboard {
            hidden.push(format!(
                "{} {}",
                style.key("motherboard.serial:"),
                display_value(lang, &board.serial)
            ));
        }
    }

    lines.push(format!("  {}", style.header(t(lang, "debug.hidden_fields"))));
    if hidden.is_empty() {
        lines.push(format!("    {}", t(lang, "debug.none")));
    } else {
        for item in hidden {
            lines.push(format!("    {item}"));
        }
    }
}

fn push_records(lines: &mut Vec<String>, report: &HardwareReport, lang: Lang, style: TextStyle) {
    if report.debug.is_empty() {
        lines.push(format!("  {}", t(lang, "debug.no_records")));
        return;
    }

    for (index, record) in report.debug.iter().enumerate() {
        lines.push(String::new());
        lines.push(style.header(format!("  [{}] {}", index + 1, record.target)));
        lines.push(format!("    {} {}", style.key("source:"), record.source));

        if let Some(note) = &record.note {
            if !note.is_empty() {
                lines.push(format!("    {} {}", style.key("note:"), style.note(note)));
            }
        }

        if record.fields.is_empty() {
            lines.push(format!("    {} {}", style.key("fields:"), t(lang, "debug.none")));
        } else {
            lines.push(format!("    {}", style.key("fields:")));
            for (key, value) in &record.fields {
                lines.push(format!("      {} {}", style.key(format!("{key}:")), value));
            }
        }
    }
}
