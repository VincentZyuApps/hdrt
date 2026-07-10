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

const DISK_LIST_ITEM_HEIGHT: u16 = 3;

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

pub(super) fn draw_disk_list(frame: &mut Frame, area: Rect, state: &mut TuiState) {
    let title = t(state.lang, "section.logical_disk");
    let visible_count = disk_list_visible_count(area);
    if visible_count == 0 {
        draw_disk_list_too_small(frame, area, title, state);
        return;
    }

    let offset = fit_disk_scroll(
        &mut state.logical_disk_scroll,
        state.selected_disk,
        state.latest.disks.len(),
        visible_count,
    );
    let items = state
        .latest
        .disks
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible_count)
        .map(|(index, disk)| {
            let selected = index == state.selected_disk;
            let mut heading = vec![disk_marker(selected)];
            append_disk_kv(
                &mut heading,
                t(state.lang, "tui.disk.key.disk"),
                disk_label(disk),
                selected,
                disk_heading_value_style(selected),
            );

            let mut usage = vec![Span::raw("  ")];
            append_disk_kv(
                &mut usage,
                t(state.lang, "tui.disk.key.used"),
                format!(
                    "{} / {}",
                    telemetry::format_bytes(disk.used_bytes),
                    telemetry::format_bytes(disk.total_bytes)
                ),
                selected,
                disk_value_style(selected),
            );
            append_disk_sep(&mut usage);
            append_disk_kv(
                &mut usage,
                t(state.lang, "tui.disk.key.free"),
                telemetry::format_bytes(disk.available_bytes),
                selected,
                disk_value_style(selected),
            );
            append_disk_sep(&mut usage);
            append_disk_kv(
                &mut usage,
                t(state.lang, "tui.disk.key.used_percent"),
                telemetry::format_percent(disk.used_percent),
                selected,
                disk_value_style(selected),
            );

            let mut io = vec![Span::raw("  ")];
            append_disk_kv(
                &mut io,
                t(state.lang, "tui.disk.key.filesystem"),
                display_value(state.lang, &disk.file_system),
                selected,
                disk_value_style(selected),
            );
            append_disk_sep(&mut io);
            append_disk_kv(
                &mut io,
                t(state.lang, "tui.disk.key.read"),
                telemetry::format_rate(disk.read_bytes_per_sec),
                selected,
                disk_rate_style(selected, Color::Green),
            );
            append_disk_sep(&mut io);
            append_disk_kv(
                &mut io,
                t(state.lang, "tui.disk.key.write"),
                telemetry::format_rate(disk.write_bytes_per_sec),
                selected,
                disk_rate_style(selected, Color::Yellow),
            );

            ListItem::new(vec![
                Line::from(heading),
                Line::from(usage),
                Line::from(io),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(Block::bordered().title(title)),
        area,
    );
}

pub(super) fn draw_physical_disk_list(frame: &mut Frame, area: Rect, state: &mut TuiState) {
    let title = t(state.lang, "section.physical_disk");
    let visible_count = disk_list_visible_count(area);
    if visible_count == 0 {
        draw_disk_list_too_small(frame, area, title, state);
        return;
    }

    let offset = fit_disk_scroll(
        &mut state.physical_disk_scroll,
        state.selected_physical_disk,
        state.report.physical_disks.len(),
        visible_count,
    );
    let items = state
        .report
        .physical_disks
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible_count)
        .map(|(index, disk)| {
            let selected = index == state.selected_physical_disk;
            let mut heading = vec![disk_marker(selected)];
            append_disk_kv(
                &mut heading,
                t(state.lang, "tui.disk.key.model"),
                display_value(state.lang, &physical_disk_name(disk)),
                selected,
                disk_heading_value_style(selected),
            );

            let mut identity = vec![Span::raw("  ")];
            append_disk_kv(
                &mut identity,
                t(state.lang, "tui.disk.key.device"),
                display_value(state.lang, &disk.device),
                selected,
                disk_value_style(selected),
            );
            append_disk_sep(&mut identity);
            append_disk_kv(
                &mut identity,
                t(state.lang, "tui.disk.key.size"),
                display_value(state.lang, &disk.size),
                selected,
                disk_value_style(selected),
            );
            append_disk_sep(&mut identity);
            append_disk_kv(
                &mut identity,
                t(state.lang, "tui.disk.key.kind"),
                display_value(state.lang, &disk.media_type),
                selected,
                disk_value_style(selected),
            );
            append_disk_sep(&mut identity);
            append_disk_kv(
                &mut identity,
                t(state.lang, "tui.disk.key.bus"),
                display_value(state.lang, &disk.bus),
                selected,
                disk_value_style(selected),
            );

            let mut status = vec![Span::raw("  ")];
            append_disk_kv(
                &mut status,
                t(state.lang, "tui.disk.key.health"),
                display_value(state.lang, &disk.health),
                selected,
                disk_rate_style(selected, health_color(&disk.health)),
            );
            append_disk_sep(&mut status);
            append_disk_kv(
                &mut status,
                t(state.lang, "tui.disk.key.firmware"),
                display_value(state.lang, &disk.firmware),
                selected,
                disk_value_style(selected),
            );

            ListItem::new(vec![
                Line::from(heading),
                Line::from(identity),
                Line::from(status),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(Block::bordered().title(title)),
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

fn disk_list_visible_count(area: Rect) -> usize {
    (area.height.saturating_sub(2) / DISK_LIST_ITEM_HEIGHT) as usize
}

fn fit_disk_scroll(
    offset: &mut usize,
    selected: usize,
    len: usize,
    visible_count: usize,
) -> usize {
    if len == 0 || visible_count == 0 {
        *offset = 0;
        return 0;
    }

    let selected = selected.min(len - 1);
    let visible_count = visible_count.min(len);
    let max_offset = len.saturating_sub(visible_count);
    *offset = (*offset).min(max_offset);

    if selected < *offset {
        *offset = selected;
    } else if selected >= *offset + visible_count {
        *offset = selected + 1 - visible_count;
    }

    *offset
}

fn draw_disk_list_too_small(frame: &mut Frame, area: Rect, title: &str, state: &TuiState) {
    frame.render_widget(
        Paragraph::new(t(state.lang, "tui.too_small"))
            .block(Block::bordered().title(title.to_string()))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn disk_marker(selected: bool) -> Span<'static> {
    if selected {
        Span::styled("> ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("  ", Style::default().fg(Color::DarkGray))
    }
}

fn append_disk_kv(
    spans: &mut Vec<Span<'static>>,
    key: &str,
    value: String,
    selected: bool,
    value_style: Style,
) {
    spans.push(Span::styled(format!("{key}: "), disk_key_style(selected)));
    spans.push(Span::styled(value, value_style));
}

fn append_disk_sep(spans: &mut Vec<Span<'static>>) {
    spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
}

fn disk_key_style(selected: bool) -> Style {
    let mut style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    if selected {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

fn disk_value_style(selected: bool) -> Style {
    let mut style = Style::default().fg(Color::White);
    if selected {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}

fn disk_heading_value_style(selected: bool) -> Style {
    disk_value_style(selected).add_modifier(Modifier::BOLD)
}

fn disk_rate_style(selected: bool, color: Color) -> Style {
    let mut style = Style::default().fg(color);
    if selected {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
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
