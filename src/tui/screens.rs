use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::Frame;

use crate::hardware::{is_unknown, DiskInfo};
use crate::i18n::t;
use crate::telemetry;

use super::cpu::draw_core_gauges;
use super::panels::{draw_disk_list, draw_memory_inventory, draw_physical_disk_list};
use super::state::TuiState;
use super::utils::{average_disk_used_percent, disk_label, label, MetricKind};
use super::widgets::{
    draw_comparison_widget, draw_empty, draw_gauge_panel, draw_history_widget, draw_io_widget,
    ComparisonItem,
};

pub(super) fn draw_overview(frame: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(1)])
        .split(area);

    draw_summary_gauges(frame, chunks[0], state);

    let body = if chunks[1].width >= 100 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(34),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(chunks[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(34),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(chunks[1])
    };

    draw_history_widget(
        frame,
        body[0],
        &format!(
            "{} {}",
            label(state.lang, "section.cpu", state.emoji),
            telemetry::format_percent(state.latest.cpu_total_percent)
        ),
        &state.cpu_history,
        state.latest.cpu_total_percent,
        MetricKind::Percent,
        Color::Cyan,
        state.chart_mode,
        state.interval,
        state.style,
    );
    draw_history_widget(
        frame,
        body[1],
        &format!(
            "{} {}",
            label(state.lang, "section.memory", state.emoji),
            telemetry::format_percent(state.latest.memory.used_percent)
        ),
        &state.memory_history,
        state.latest.memory.used_percent,
        MetricKind::Percent,
        Color::Magenta,
        state.chart_mode,
        state.interval,
        state.style,
    );
    if state.disk_io_available() {
        draw_io_widget(
            frame,
            body[2],
            t(state.lang, "tui.disk_io"),
            &state.disk_read_history,
            &state.disk_write_history,
            Color::Green,
            Color::Yellow,
            state.chart_mode,
            state.interval,
            state.style,
        );
    } else {
        draw_empty(
            frame,
            body[2],
            t(state.lang, "tui.io_unavailable_message"),
            state.style,
        );
    }
}

pub(super) fn draw_cpu(frame: &mut Frame, area: Rect, state: &TuiState) {
    let core_rows = state.latest.cpu_cores_percent.len().div_ceil(2) as u16;
    let core_height = core_rows.saturating_add(2).max(3);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(core_height), Constraint::Min(1)])
        .split(area);

    draw_core_gauges(frame, chunks[0], state);
    draw_history_widget(
        frame,
        chunks[1],
        &format!(
            "{} {}",
            t(state.lang, "tui.cpu_total"),
            telemetry::format_percent(state.latest.cpu_total_percent)
        ),
        &state.cpu_history,
        state.latest.cpu_total_percent,
        MetricKind::Percent,
        Color::Cyan,
        state.chart_mode,
        state.interval,
        state.style,
    );
}

pub(super) fn draw_memory(frame: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(1)])
        .split(area);

    let gauges = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    draw_gauge_panel(
        frame,
        gauges[0],
        &format!(
            "{} {}/{}",
            t(state.lang, "tui.memory_used"),
            telemetry::format_bytes(state.latest.memory.used_bytes),
            telemetry::format_bytes(state.latest.memory.total_bytes)
        ),
        state.latest.memory.used_percent,
        Color::Magenta,
        state.style,
    );
    draw_gauge_panel(
        frame,
        gauges[1],
        &format!(
            "{} {}/{}",
            t(state.lang, "tui.swap_used"),
            telemetry::format_bytes(state.latest.memory.swap_used_bytes),
            telemetry::format_bytes(state.latest.memory.swap_total_bytes)
        ),
        state.latest.memory.swap_used_percent,
        Color::Yellow,
        state.style,
    );

    if chunks[1].width >= 110 && !state.report.memory.is_empty() {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(50), Constraint::Length(42)])
            .split(chunks[1]);
        draw_history_widget(
            frame,
            body[0],
            t(state.lang, "tui.memory_history"),
            &state.memory_history,
            state.latest.memory.used_percent,
            MetricKind::Percent,
            Color::Magenta,
            state.chart_mode,
            state.interval,
            state.style,
        );
        draw_memory_inventory(frame, body[1], state);
    } else {
        draw_history_widget(
            frame,
            chunks[1],
            t(state.lang, "tui.memory_history"),
            &state.memory_history,
            state.latest.memory.used_percent,
            MetricKind::Percent,
            Color::Magenta,
            state.chart_mode,
            state.interval,
            state.style,
        );
    }
}

pub(super) fn draw_physical_disk(frame: &mut Frame, area: Rect, state: &mut TuiState) {
    if state.report.physical_disks.is_empty() {
        draw_empty(frame, area, t(state.lang, "no_data"), state.style);
        return;
    }

    let page = disk_page_layout(area);
    draw_selected_physical_disk_gauge(frame, page[0], state);
    let chunks = disk_body_chunks(page[1]);
    draw_physical_disk_list(frame, chunks[0], state);
    let items = state
        .report
        .physical_disks
        .iter()
        .map(|disk| ComparisonItem {
            label: physical_disk_name(disk),
            value: parse_size_bytes(&disk.size).unwrap_or_default(),
            display: disk.size.clone(),
        })
        .collect::<Vec<_>>();
    draw_comparison_widget(
        frame,
        chunks[1],
        t(state.lang, "section.physical_disk"),
        &items,
        None,
        state.chart_mode,
        Color::Cyan,
        state.style,
    );
}

pub(super) fn draw_logical_disk(frame: &mut Frame, area: Rect, state: &mut TuiState) {
    if state.latest.disks.is_empty() {
        draw_empty(frame, area, t(state.lang, "no_data"), state.style);
        return;
    }

    let page = disk_page_layout(area);
    draw_selected_logical_disk_gauge(frame, page[0], state);
    let chunks = disk_body_chunks(page[1]);
    draw_disk_list(frame, chunks[0], state);
    draw_selected_disk_io(frame, chunks[1], state);
}

fn draw_selected_physical_disk_gauge(frame: &mut Frame, area: Rect, state: &TuiState) {
    let selected = state
        .report
        .physical_disks
        .get(state.selected_physical_disk)
        .unwrap_or(&state.report.physical_disks[0]);
    let selected_size = parse_size_bytes(&selected.size).unwrap_or_default();
    let max_size = state
        .report
        .physical_disks
        .iter()
        .filter_map(|disk| parse_size_bytes(&disk.size))
        .fold(0.0, f64::max)
        .max(1.0);
    draw_gauge_panel(
        frame,
        area,
        &format!(
            "{} {} {}",
            t(state.lang, "section.physical_disk"),
            physical_disk_name(selected),
            selected.size
        ),
        selected_size / max_size * 100.0,
        Color::Cyan,
        state.style,
    );
}

fn draw_selected_logical_disk_gauge(frame: &mut Frame, area: Rect, state: &TuiState) {
    let Some(disk) = state.selected_disk() else {
        draw_empty(frame, area, t(state.lang, "no_data"), state.style);
        return;
    };
    draw_gauge_panel(
        frame,
        area,
        &format!(
            "{} {} {}/{}",
            t(state.lang, "tui.disk_used"),
            disk_label(disk),
            telemetry::format_bytes(disk.used_bytes),
            telemetry::format_bytes(disk.total_bytes)
        ),
        disk.used_percent,
        Color::Green,
        state.style,
    );
}

fn draw_selected_disk_io(frame: &mut Frame, area: Rect, state: &TuiState) {
    let Some(disk) = state.selected_disk() else {
        draw_empty(frame, area, t(state.lang, "no_data"), state.style);
        return;
    };
    if !disk.io_available {
        draw_empty(
            frame,
            area,
            t(state.lang, "tui.io_unavailable_message"),
            state.style,
        );
        return;
    }
    let Some(history) = state.selected_disk_history() else {
        draw_empty(frame, area, t(state.lang, "no_data"), state.style);
        return;
    };

    let title = disk_label(disk);
    draw_io_widget(
        frame,
        area,
        &title,
        &history.read,
        &history.write,
        Color::Green,
        Color::Yellow,
        state.chart_mode,
        state.interval,
        state.style,
    );
}

fn draw_summary_gauges(frame: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(area);

    draw_gauge_panel(
        frame,
        chunks[0],
        t(state.lang, "tui.cpu_total"),
        state.latest.cpu_total_percent,
        Color::Cyan,
        state.style,
    );
    draw_gauge_panel(
        frame,
        chunks[1],
        &format!(
            "{} {}",
            t(state.lang, "tui.memory_used"),
            telemetry::format_bytes(state.latest.memory.used_bytes)
        ),
        state.latest.memory.used_percent,
        Color::Magenta,
        state.style,
    );
    let io_rate = if state.disk_io_available() {
        telemetry::format_rate(
            state
                .latest
                .disks
                .iter()
                .filter(|disk| disk.io_available)
                .map(|disk| disk.read_bytes_per_sec + disk.write_bytes_per_sec)
                .sum::<f64>(),
        )
    } else {
        t(state.lang, "tui.io_unavailable").to_string()
    };
    draw_gauge_panel(
        frame,
        chunks[2],
        &format!("{} {}", t(state.lang, "tui.disk_io"), io_rate),
        average_disk_used_percent(&state.latest.disks),
        Color::Green,
        state.style,
    );
}

fn disk_page_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(1)])
        .split(area)
        .to_vec()
}

fn disk_body_chunks(area: Rect) -> Vec<Rect> {
    if area.width >= 90 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(618, 1618), Constraint::Ratio(1000, 1618)])
            .split(area)
            .to_vec()
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area)
            .to_vec()
    }
}

fn physical_disk_name(disk: &DiskInfo) -> String {
    if is_unknown(&disk.model) {
        disk.device.clone()
    } else {
        disk.model.clone()
    }
}

fn parse_size_bytes(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case(crate::hardware::UNKNOWN) {
        return None;
    }

    let number = value
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    let amount = number.parse::<f64>().ok()?;
    let unit = value[number.len()..].trim().to_ascii_lowercase();
    let multiplier = if unit.starts_with("t") {
        1024.0_f64.powi(4)
    } else if unit.starts_with("g") {
        1024.0_f64.powi(3)
    } else if unit.starts_with("m") {
        1024.0_f64.powi(2)
    } else if unit.starts_with("k") {
        1024.0
    } else {
        1.0
    };
    Some(amount * multiplier)
}
