use crate::emoji;
use crate::hardware::{HardwareReport, HdrtWarning, Section};
use crate::i18n::{t, Lang};

use super::style::TextStyle;

pub(super) fn collect(report: &HardwareReport, section: Section) -> Vec<HdrtWarning> {
    let mut warnings = report.warnings.clone();

    if matches!(
        section,
        Section::Disk | Section::PhysicalDisk | Section::All
    ) {
        warnings.extend(
            report
                .physical_disks
                .iter()
                .flat_map(|item| item.warnings.clone()),
        );
    }
    if matches!(
        section,
        Section::Disk | Section::LogicalDisk | Section::All
    ) {
        warnings.extend(
            report
                .logical_disks
                .iter()
                .flat_map(|item| item.warnings.clone()),
        );
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

pub(super) fn render(
    warnings: &[HdrtWarning],
    lang: Lang,
    emoji_enabled: bool,
    style: TextStyle,
) -> String {
    let warning_label = emoji::decorate(emoji_enabled, "warnings", t(lang, "warnings"));
    let hint_label = emoji::decorate(emoji_enabled, "hint", t(lang, "hint"));
    let mut lines = vec![format!("{}:", style.warning(warning_label))];

    for warning in warnings {
        lines.push(format!(
            "- [{}] {}",
            style.warning(&warning.code),
            warning.message
        ));
        if let Some(hint) = &warning.hint {
            lines.push(format!("  {}: {hint}", style.key(&hint_label)));
        }
    }

    lines.join("\n")
}
