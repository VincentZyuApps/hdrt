use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::hardware::DiskInfo;
use crate::i18n::{display_value, t};
use crate::telemetry;

use super::state::TuiState;
use super::utils::{
    collect_warnings, disk_label, health_color, label, push_kv, warning_percent,
};
use super::widgets::{draw_empty, draw_gauge_panel};

pub(super) fn draw_motherboard(frame: &mut Frame, area: Rect, state: &TuiState) {
    let mut lines = Vec::new();

    if let Some(board) = &state.report.motherboard {
        push_kv(
            &mut lines,
            label(state.lang, "motherboard.manufacturer", state.emoji),
            display_value(state.lang, &board.manufacturer),
        );
        push_kv(
            &mut lines,
            label(state.lang, "motherboard.product", state.emoji),
            display_value(state.lang, &board.product),
        );
        push_kv(
            &mut lines,
            label(state.lang, "motherboard.version", state.emoji),
            display_value(state.lang, &board.version),
        );
        push_kv(
            &mut lines,
            label(state.lang, "motherboard.serial", state.emoji),
            display_value(state.lang, &board.serial),
        );
        lines.push(Line::from(""));
        push_kv(
            &mut lines,
            label(state.lang, "motherboard.bios_vendor", state.emoji),
            display_value(state.lang, &board.bios_vendor),
        );
        push_kv(
            &mut lines,
            label(state.lang, "motherboard.bios_version", state.emoji),
            display_value(state.lang, &board.bios_version),
        );
    } else {
        lines.push(Line::from(t(state.lang, "no_data")));
    }

    let block = Block::bordered().title(label(
        state.lang,
        "section.motherboard",
        state.emoji,
    ));
    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: true }), area);
}

pub(super) fn draw_health(frame: &mut Frame, area: Rect, state: &TuiState) {
    let warnings = collect_warnings(&state.report);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(1)])
        .split(area);

    let gauges = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(chunks[0]);
    draw_gauge_panel(
        frame,
        gauges[0],
        t(state.lang, "tui.cpu_total"),
        state.latest.cpu_total_percent,
        Color::Cyan,
    );
    draw_gauge_panel(
        frame,
        gauges[1],
        t(state.lang, "tui.memory_used"),
        state.latest.memory.used_percent,
        Color::Magenta,
    );
    draw_gauge_panel(
        frame,
        gauges[2],
        &format!("{} {}", t(state.lang, "warnings"), warnings.len()),
        warning_percent(warnings.len()),
        Color::Red,
    );

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!("{}: {}", t(state.lang, "warnings"), warnings.len()),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if state.report.physical_disks.is_empty() {
        lines.push(Line::from(t(state.lang, "no_data")));
    } else {
        for disk in &state.report.physical_disks {
            lines.push(Line::from(vec![
                Span::styled(
                    display_value(state.lang, &disk.device),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("  "),
                Span::styled(
                    display_value(state.lang, &disk.health),
                    Style::default().fg(health_color(&disk.health)),
                ),
                Span::raw("  "),
                Span::raw(display_value(state.lang, &disk.model)),
            ]));
        }
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::bordered().title(label(state.lang, "section.health", state.emoji)))
            .wrap(Wrap { trim: true }),
        chunks[1],
    );
}

pub(super) fn draw_warnings(frame: &mut Frame, area: Rect, state: &TuiState) {
    let warnings = collect_warnings(&state.report);
    if warnings.is_empty() {
        draw_empty(frame, area, t(state.lang, "tui.no_warnings"));
        return;
    }

    let items = warnings
        .iter()
        .map(|warning| {
            let mut lines = vec![Line::from(vec![
                Span::styled(
                    format!("[{}] ", warning.code),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(warning.message.clone()),
            ])];
            if let Some(hint) = &warning.hint {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}: ", label(state.lang, "hint", state.emoji)),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(hint.clone()),
                ]));
            }
            ListItem::new(lines)
        })
        .collect::<Vec<_>>();

    let list = List::new(items).block(Block::bordered().title(label(
        state.lang,
        "warnings",
        state.emoji,
    )));
    frame.render_widget(list, area);
}

pub(super) fn draw_core_gauges(frame: &mut Frame, area: Rect, state: &TuiState) {
    let cores = &state.latest.cpu_cores_percent;
    if cores.is_empty() {
        draw_empty(frame, area, t(state.lang, "no_data"));
        return;
    }

    let visible = area.height.saturating_sub(2).max(1) as usize;
    let lines = cores
        .iter()
        .take(visible)
        .enumerate()
        .map(|(index, usage)| {
            let filled = ((*usage / 100.0) * 20.0).round() as usize;
            let empty = 20usize.saturating_sub(filled);
            Line::from(vec![
                Span::styled(format!("CPU{index:<2} "), Style::default().fg(Color::Cyan)),
                Span::styled("█".repeat(filled), Style::default().fg(Color::Blue)),
                Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
                Span::raw(format!(" {}", telemetry::format_percent(*usage))),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(t(state.lang, "tui.cpu_cores"))),
        area,
    );
}

pub(super) fn draw_disk_list(frame: &mut Frame, area: Rect, state: &TuiState) {
    let items = state
        .latest
        .disks
        .iter()
        .enumerate()
        .map(|(index, disk)| {
            let selected = index == state.selected_disk;
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(vec![
                Line::from(Span::styled(
                    format!("{} {}", if selected { ">" } else { " " }, disk_label(disk)),
                    style,
                )),
                Line::from(Span::styled(
                    format!(
                        "  {} / {} | {}",
                        telemetry::format_bytes(disk.used_bytes),
                        telemetry::format_bytes(disk.total_bytes),
                        telemetry::format_percent(disk.used_percent)
                    ),
                    style,
                )),
                Line::from(Span::styled(
                    format!(
                        "  {} | free {}",
                        disk.file_system,
                        telemetry::format_bytes(disk.available_bytes)
                    ),
                    style,
                )),
                Line::from(Span::styled(
                    format!(
                        "  R {} | W {}",
                        telemetry::format_rate(disk.read_bytes_per_sec),
                        telemetry::format_rate(disk.write_bytes_per_sec)
                    ),
                    style,
                )),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(Block::bordered().title(t(state.lang, "section.logical_disk"))),
        area,
    );
}

pub(super) fn draw_physical_disk_list(frame: &mut Frame, area: Rect, state: &TuiState) {
    let items = state
        .report
        .physical_disks
        .iter()
        .enumerate()
        .map(|(index, disk)| {
            let selected = index == state.selected_physical_disk;
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(vec![
                Line::from(Span::styled(
                    format!("{} {}", if selected { ">" } else { " " }, physical_disk_name(disk)),
                    style,
                )),
                Line::from(Span::styled(
                    format!("  {}", display_value(state.lang, &disk.device)),
                    style,
                )),
                Line::from(Span::styled(
                    format!(
                        "  {} | {} | {}",
                        display_value(state.lang, &disk.size),
                        display_value(state.lang, &disk.media_type),
                        display_value(state.lang, &disk.bus)
                    ),
                    style,
                )),
                Line::from(Span::styled(
                    format!(
                        "  {} | FW {}",
                        display_value(state.lang, &disk.health),
                        display_value(state.lang, &disk.firmware)
                    ),
                    style,
                )),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(Block::bordered().title(t(state.lang, "section.physical_disk"))),
        area,
    );
}

fn physical_disk_name(disk: &DiskInfo) -> String {
    let model = disk.model.trim();
    if model.is_empty() || model == crate::hardware::UNKNOWN {
        disk.device.clone()
    } else {
        model.to_string()
    }
}

pub(super) fn draw_memory_inventory(frame: &mut Frame, area: Rect, state: &TuiState) {
    let mut lines = Vec::new();
    for memory in &state.report.memory {
        lines.push(Line::from(Span::styled(
            display_value(state.lang, &memory.slot),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(format!(
            "{} | {} | {}",
            display_value(state.lang, &memory.size),
            display_value(state.lang, &memory.speed),
            display_value(state.lang, &memory.manufacturer)
        )));
        lines.push(Line::from(""));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::bordered().title(label(state.lang, "section.memory", state.emoji)))
            .wrap(Wrap { trim: true }),
        area,
    );
}
