use std::collections::VecDeque;
use std::time::Duration;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Axis, Bar, BarChart, BarGroup, Block, Chart, Dataset, Gauge, GraphType, Paragraph, Sparkline,
    Wrap,
};
use ratatui::Frame;

use crate::telemetry;

use crate::app::options::ChartMode;
use super::charts::{draw_chart, draw_dual_chart, draw_history_bars, draw_sparkline};
use super::utils::{format_metric, history_peak, MetricKind};

pub(super) struct ComparisonItem {
    pub(super) label: String,
    pub(super) value: f64,
    pub(super) display: String,
}

pub(super) fn draw_comparison_widget(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[ComparisonItem],
    fixed_max: Option<f64>,
    mode: ChartMode,
    color: Color,
) {
    if items.is_empty() {
        draw_empty(frame, area, "No data collected.");
        return;
    }

    let max_value = fixed_max.unwrap_or_else(|| {
        items
            .iter()
            .map(|item| item.value)
            .fold(0.0, f64::max)
            .max(1.0)
    });

    match mode {
        ChartMode::Gauge => draw_comparison_gauges(frame, area, title, items, max_value, color),
        ChartMode::Bar => draw_comparison_bars(frame, area, title, items, max_value, color),
        ChartMode::Sparkline => draw_comparison_sparkline(frame, area, title, items, max_value, color),
        ChartMode::Line => {
            draw_static_chart(frame, area, title, items, max_value, color, GraphType::Line)
        }
        ChartMode::Scatter => draw_static_chart(
            frame,
            area,
            title,
            items,
            max_value,
            color,
            GraphType::Scatter,
        ),
    }
}

pub(super) fn draw_history_widget(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    history: &VecDeque<f64>,
    current: f64,
    kind: MetricKind,
    color: Color,
    mode: ChartMode,
    interval: Duration,
) {
    let max_value = match kind {
        MetricKind::Percent => 100.0,
        MetricKind::BytesPerSec => history_peak(history).max(current).max(1.0),
    };
    match mode {
        ChartMode::Line => draw_chart(
            frame,
            area,
            title,
            history,
            max_value,
            color,
            GraphType::Line,
            kind,
            interval,
        ),
        ChartMode::Scatter => {
            draw_chart(
                frame,
                area,
                title,
                history,
                max_value,
                color,
                GraphType::Scatter,
                kind,
                interval,
            )
        }
        ChartMode::Bar => {
            draw_history_bars(frame, area, title, history, max_value, color, kind, interval)
        }
        ChartMode::Sparkline => draw_sparkline(frame, area, title, history, max_value, color),
        ChartMode::Gauge => draw_gauge_panel(
            frame,
            area,
            &format!("{} {}", title, format_metric(current, kind)),
            if max_value <= 0.0 {
                0.0
            } else {
                current / max_value * 100.0
            },
            color,
        ),
    }
}

fn draw_comparison_gauges(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[ComparisonItem],
    max_value: f64,
    color: Color,
) {
    let inner_width = area.width.saturating_sub(4).max(10) as usize;
    let bar_width = inner_width.clamp(10, 28);
    let visible = area.height.saturating_sub(2).max(1) as usize;
    let lines = items
        .iter()
        .take(visible)
        .map(|item| {
            let percent = (item.value / max_value * 100.0).clamp(0.0, 100.0);
            Line::from(vec![
                Span::styled(
                    format!("{:<12} ", short_label(&item.label)),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    gauge_bar(percent, bar_width),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" {}", item.display)),
            ])
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::bordered().title(title.to_string()))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_comparison_sparkline(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[ComparisonItem],
    max_value: f64,
    color: Color,
) {
    let data = items
        .iter()
        .map(|item| item.value.round().max(0.0) as u64)
        .collect::<Vec<_>>();
    let sparkline = Sparkline::default()
        .block(Block::bordered().title(title.to_string()))
        .data(&data)
        .max(max_value.round().max(1.0) as u64)
        .style(Style::default().fg(color));
    frame.render_widget(sparkline, area);
}

fn draw_comparison_bars(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[ComparisonItem],
    max_value: f64,
    color: Color,
) {
    let bars = items
        .iter()
        .map(|item| {
            Bar::default()
                .value(item.value.round().max(0.0) as u64)
                .label(Line::from(short_label(&item.label)))
                .text_value(item.display.clone())
        })
        .collect::<Vec<_>>();
    let group = BarGroup::default().bars(&bars);
    let chart = BarChart::default()
        .block(Block::bordered().title(title.to_string()))
        .data(group)
        .max(max_value.round().max(1.0) as u64)
        .bar_width(5)
        .bar_gap(1)
        .bar_style(Style::default().fg(color))
        .value_style(Style::default().fg(Color::Black).bg(color));
    frame.render_widget(chart, area);
}

fn draw_static_chart(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[ComparisonItem],
    max_value: f64,
    color: Color,
    graph_type: GraphType,
) {
    let points = items
        .iter()
        .enumerate()
        .map(|(index, item)| (index as f64, item.value))
        .collect::<Vec<_>>();
    let x_max = points.len().saturating_sub(1).max(1) as f64;
    let dataset = Dataset::default()
        .name(title.to_string())
        .marker(symbols::Marker::Braille)
        .graph_type(graph_type)
        .style(Style::default().fg(color))
        .data(&points);
    let max_label = items
        .iter()
        .max_by(|left, right| left.value.total_cmp(&right.value))
        .map(|item| item.display.clone())
        .unwrap_or_else(|| format!("{:.0}", max_value.round()));
    let chart = Chart::new(vec![dataset])
        .block(Block::bordered().title(title.to_string()))
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .labels(static_axis_labels(items)),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, max_value])
                .labels(vec![Span::raw("0"), Span::raw(max_label)]),
        );
    frame.render_widget(chart, area);
}

fn static_axis_labels(items: &[ComparisonItem]) -> Vec<Span<'static>> {
    match (items.first(), items.last()) {
        (Some(first), Some(last)) if items.len() > 1 => {
            vec![Span::raw(short_label(&first.label)), Span::raw(short_label(&last.label))]
        }
        (Some(item), _) => vec![Span::raw(short_label(&item.label))],
        _ => vec![Span::raw("")],
    }
}

fn gauge_bar(percent: f64, width: usize) -> String {
    let filled = ((percent / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn short_label(value: &str) -> String {
    let value = value.trim();
    const MAX: usize = 14;
    if value.chars().count() <= MAX {
        value.to_string()
    } else {
        let mut text = value.chars().take(MAX - 1).collect::<String>();
        text.push('…');
        text
    }
}

pub(super) fn draw_io_widget(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    read_history: &VecDeque<f64>,
    write_history: &VecDeque<f64>,
    read_color: Color,
    write_color: Color,
    mode: ChartMode,
    interval: Duration,
) {
    let read_current = read_history.back().copied().unwrap_or_default();
    let write_current = write_history.back().copied().unwrap_or_default();
    let max_value = history_peak(read_history)
        .max(history_peak(write_history))
        .max(read_current)
        .max(write_current)
        .max(1.0);

    match mode {
        ChartMode::Line => draw_dual_chart(
            frame,
            area,
            title,
            read_history,
            write_history,
            max_value,
            GraphType::Line,
            interval,
        ),
        ChartMode::Scatter => draw_dual_chart(
            frame,
            area,
            title,
            read_history,
            write_history,
            max_value,
            GraphType::Scatter,
            interval,
        ),
        ChartMode::Bar => draw_bar_chart(
            frame,
            area,
            title,
            vec![
                ("R".to_string(), read_current.round() as u64),
                ("W".to_string(), write_current.round() as u64),
            ],
            max_value.round() as u64,
            read_color,
        ),
        ChartMode::Sparkline => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);
            draw_sparkline(
                frame,
                chunks[0],
                &io_panel_title(title, "R", read_current),
                read_history,
                max_value,
                read_color,
            );
            draw_sparkline(
                frame,
                chunks[1],
                &io_panel_title(title, "W", write_current),
                write_history,
                max_value,
                write_color,
            );
        }
        ChartMode::Gauge => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);
            draw_gauge_panel(
                frame,
                chunks[0],
                &io_panel_title(title, "R", read_current),
                read_current / max_value * 100.0,
                read_color,
            );
            draw_gauge_panel(
                frame,
                chunks[1],
                &io_panel_title(title, "W", write_current),
                write_current / max_value * 100.0,
                write_color,
            );
        }
    }
}

fn io_panel_title(title: &str, label: &str, current: f64) -> String {
    format!("{title} | {label} {}", telemetry::format_rate(current))
}

pub(super) fn draw_bar_chart(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    data: Vec<(String, u64)>,
    max_value: u64,
    color: Color,
) {
    let bars = data
        .iter()
        .map(|(label, value)| {
            Bar::default()
                .value(*value)
                .label(Line::from(label.clone()))
                .text_value(value.to_string())
        })
        .collect::<Vec<_>>();
    let group = BarGroup::default().bars(&bars);
    let chart = BarChart::default()
        .block(Block::bordered().title(title.to_string()))
        .data(group)
        .max(max_value.max(1))
        .bar_width(3)
        .bar_gap(1)
        .bar_style(Style::default().fg(color))
        .value_style(Style::default().fg(Color::Black).bg(color));
    frame.render_widget(chart, area);
}

pub(super) fn draw_gauge_panel(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    percent: f64,
    color: Color,
) {
    let percent = percent.clamp(0.0, 100.0);
    let gauge = Gauge::default()
        .block(Block::bordered().title(title.to_string()))
        .gauge_style(
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD),
        )
        .percent(percent.round() as u16)
        .label(telemetry::format_percent(percent));
    frame.render_widget(gauge, area);
}

pub(super) fn draw_empty(frame: &mut Frame, area: Rect, text: &str) {
    frame.render_widget(
        Paragraph::new(text.to_string())
            .block(Block::bordered())
            .wrap(Wrap { trim: true }),
        area,
    );
}
