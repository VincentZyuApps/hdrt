use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::Frame;

use crate::i18n::t;
use crate::telemetry;

use super::panels::{draw_core_gauges, draw_disk_list, draw_memory_inventory};
use super::state::{ChartMode, TuiState};
use super::utils::{average_disk_used_percent, disk_label, label, MetricKind};
use super::widgets::{
    draw_bar_chart, draw_empty, draw_gauge_panel, draw_history_widget, draw_io_widget,
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
    );
    draw_io_widget(
        frame,
        body[2],
        t(state.lang, "tui.disk_io"),
        &state.disk_read_history,
        &state.disk_write_history,
        Color::Green,
        Color::Yellow,
        state.chart_mode,
    );
}

pub(super) fn draw_cpu(frame: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    draw_history_widget(
        frame,
        chunks[0],
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
    );

    if state.latest.cpu_cores_percent.is_empty() {
        draw_empty(frame, chunks[1], t(state.lang, "no_data"));
    } else if matches!(state.chart_mode, ChartMode::Gauge) {
        draw_core_gauges(frame, chunks[1], state);
    } else {
        let bars = state
            .latest
            .cpu_cores_percent
            .iter()
            .enumerate()
            .map(|(index, value)| (index.to_string(), value.round() as u64))
            .collect::<Vec<_>>();
        draw_bar_chart(
            frame,
            chunks[1],
            t(state.lang, "tui.cpu_cores"),
            bars,
            100,
            Color::Blue,
        );
    }
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
        );
    }
}

pub(super) fn draw_disk(frame: &mut Frame, area: Rect, state: &TuiState) {
    if state.latest.disks.is_empty() {
        draw_empty(frame, area, t(state.lang, "no_data"));
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(1)])
        .split(area);

    if let Some(disk) = state.selected_disk() {
        draw_gauge_panel(
            frame,
            chunks[0],
            &format!(
                "{} {} {}/{}",
                t(state.lang, "tui.disk_used"),
                disk_label(disk),
                telemetry::format_bytes(disk.used_bytes),
                telemetry::format_bytes(disk.total_bytes)
            ),
            disk.used_percent,
            Color::Green,
        );
    }

    if chunks[1].width >= 100 {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(34), Constraint::Min(40)])
            .split(chunks[1]);
        draw_disk_list(frame, body[0], state);
        draw_selected_disk_io(frame, body[1], state);
    } else {
        draw_selected_disk_io(frame, chunks[1], state);
    }
}

fn draw_selected_disk_io(frame: &mut Frame, area: Rect, state: &TuiState) {
    let Some(disk) = state.selected_disk() else {
        draw_empty(frame, area, t(state.lang, "no_data"));
        return;
    };
    let Some(history) = state.selected_disk_history() else {
        draw_empty(frame, area, t(state.lang, "no_data"));
        return;
    };

    let title = format!(
        "{} | R {} | W {} | {} {}",
        disk_label(disk),
        telemetry::format_rate(disk.read_bytes_per_sec),
        telemetry::format_rate(disk.write_bytes_per_sec),
        t(state.lang, "tui.disk_used"),
        telemetry::format_percent(disk.used_percent)
    );
    draw_io_widget(
        frame,
        area,
        &title,
        &history.read,
        &history.write,
        Color::Green,
        Color::Yellow,
        state.chart_mode,
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
    );
    draw_gauge_panel(
        frame,
        chunks[2],
        &format!(
            "{} {}",
            t(state.lang, "tui.disk_io"),
            telemetry::format_rate(
                state
                    .latest
                    .disks
                    .iter()
                    .map(|disk| disk.read_bytes_per_sec + disk.write_bytes_per_sec)
                    .sum::<f64>(),
            )
        ),
        average_disk_used_percent(&state.latest.disks),
        Color::Green,
    );
}
