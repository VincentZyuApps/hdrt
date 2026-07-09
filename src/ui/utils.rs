use std::collections::VecDeque;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::app::options::TuiTab;
use crate::emoji;
use crate::hardware::{HardwareReport, HdrtWarning};
use crate::i18n::{t, Lang};
use crate::telemetry::{self, DiskTelemetry, HISTORY_LIMIT};

pub(super) fn tab_titles(lang: Lang, emoji_enabled: bool) -> Vec<Line<'static>> {
    [
        "section.overview",
        "section.disk",
        "section.memory",
        "section.cpu",
        "section.motherboard",
        "section.health",
        "warnings",
    ]
    .iter()
    .map(|key| Line::from(Span::raw(label(lang, key, emoji_enabled))))
    .collect()
}

pub(super) fn tab_index(tab: TuiTab) -> usize {
    match tab {
        TuiTab::Overview => 0,
        TuiTab::Disk => 1,
        TuiTab::Memory => 2,
        TuiTab::Cpu => 3,
        TuiTab::Motherboard => 4,
        TuiTab::Health => 5,
        TuiTab::Warnings => 6,
    }
}

pub(super) fn label(lang: Lang, key: &str, enabled: bool) -> String {
    emoji::decorate(enabled, key, t(lang, key))
}

pub(super) fn push_history(history: &mut VecDeque<f64>, value: f64) {
    if history.len() >= HISTORY_LIMIT {
        history.pop_front();
    }
    history.push_back(value);
}

pub(super) fn history_points(history: &VecDeque<f64>) -> Vec<(f64, f64)> {
    history
        .iter()
        .enumerate()
        .map(|(index, value)| (index as f64, *value))
        .collect()
}

pub(super) fn history_peak(history: &VecDeque<f64>) -> f64 {
    history.iter().copied().fold(0.0, f64::max)
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub(super) enum MetricKind {
    Percent,
    BytesPerSec,
}

pub(super) fn format_metric(value: f64, kind: MetricKind) -> String {
    match kind {
        MetricKind::Percent => telemetry::format_percent(value),
        MetricKind::BytesPerSec => telemetry::format_rate(value),
    }
}

pub(super) fn average_disk_used_percent(disks: &[DiskTelemetry]) -> f64 {
    let total = disks.iter().map(|disk| disk.total_bytes).sum::<u64>();
    let used = disks.iter().map(|disk| disk.used_bytes).sum::<u64>();
    if total == 0 {
        0.0
    } else {
        used as f64 / total as f64 * 100.0
    }
}

pub(super) fn warning_percent(count: usize) -> f64 {
    ((count.min(10) as f64) / 10.0) * 100.0
}

pub(super) fn disk_label(disk: &DiskTelemetry) -> String {
    if disk.mount_point.is_empty() {
        disk.name.clone()
    } else if disk.name.is_empty() {
        disk.mount_point.clone()
    } else {
        format!("{} ({})", disk.name, disk.mount_point)
    }
}

pub(super) fn push_kv(lines: &mut Vec<Line<'static>>, key: String, value: String) {
    lines.push(Line::from(vec![
        Span::styled(format!("{key}: "), Style::default().fg(Color::Cyan)),
        Span::raw(value),
    ]));
}

pub(super) fn collect_warnings(report: &HardwareReport) -> Vec<HdrtWarning> {
    let mut warnings = report.warnings.clone();
    warnings.extend(report.disks.iter().flat_map(|item| item.warnings.clone()));
    warnings.extend(report.memory.iter().flat_map(|item| item.warnings.clone()));
    if let Some(cpu) = &report.cpu {
        warnings.extend(cpu.warnings.clone());
    }
    if let Some(board) = &report.motherboard {
        warnings.extend(board.warnings.clone());
    }
    warnings
}

pub(super) fn health_color(value: &str) -> Color {
    let normalized = value.to_ascii_lowercase();
    if normalized.contains("ok") || normalized.contains("healthy") || normalized.contains("good") {
        Color::Green
    } else if normalized.contains("unknown") || normalized.contains("未知") {
        Color::Gray
    } else {
        Color::Red
    }
}
