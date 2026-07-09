use std::collections::VecDeque;

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

use super::state::ChartMode;
use super::utils::{format_metric, history_peak, history_points, MetricKind};

pub(super) fn draw_history_widget(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    history: &VecDeque<f64>,
    current: f64,
    kind: MetricKind,
    color: Color,
    mode: ChartMode,
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
            )
        }
        ChartMode::Bar => draw_history_bars(frame, area, title, history, max_value, color, kind),
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

pub(super) fn draw_io_widget(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    read_history: &VecDeque<f64>,
    write_history: &VecDeque<f64>,
    read_color: Color,
    write_color: Color,
    mode: ChartMode,
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
        ),
        ChartMode::Scatter => draw_dual_chart(
            frame,
            area,
            title,
            read_history,
            write_history,
            max_value,
            GraphType::Scatter,
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
                &format!("{title} R {}", telemetry::format_rate(read_current)),
                read_history,
                max_value,
                read_color,
            );
            draw_sparkline(
                frame,
                chunks[1],
                &format!("{title} W {}", telemetry::format_rate(write_current)),
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
                &format!("{title} R {}", telemetry::format_rate(read_current)),
                read_current / max_value * 100.0,
                read_color,
            );
            draw_gauge_panel(
                frame,
                chunks[1],
                &format!("{title} W {}", telemetry::format_rate(write_current)),
                write_current / max_value * 100.0,
                write_color,
            );
        }
    }
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

fn draw_chart(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    history: &VecDeque<f64>,
    max_value: f64,
    color: Color,
    graph_type: GraphType,
    kind: MetricKind,
) {
    let points = history_points(history);
    let x_max = points.len().saturating_sub(1).max(1) as f64;
    let dataset = Dataset::default()
        .name(title.to_string())
        .marker(symbols::Marker::Braille)
        .graph_type(graph_type)
        .style(Style::default().fg(color))
        .data(&points);
    let chart = Chart::new(vec![dataset])
        .block(Block::bordered().title(title.to_string()))
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .labels(vec![Span::raw("old"), Span::raw("now")]),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, max_value])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format_metric(max_value, kind)),
                ]),
        );
    frame.render_widget(chart, area);
}

fn draw_dual_chart(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    read_history: &VecDeque<f64>,
    write_history: &VecDeque<f64>,
    max_value: f64,
    graph_type: GraphType,
) {
    let read_points = history_points(read_history);
    let write_points = history_points(write_history);
    let x_max = read_points
        .len()
        .max(write_points.len())
        .saturating_sub(1)
        .max(1) as f64;
    let read = Dataset::default()
        .name("read")
        .marker(symbols::Marker::Braille)
        .graph_type(graph_type)
        .style(Style::default().fg(Color::Green))
        .data(&read_points);
    let write = Dataset::default()
        .name("write")
        .marker(symbols::Marker::Braille)
        .graph_type(graph_type)
        .style(Style::default().fg(Color::Yellow))
        .data(&write_points);
    let chart = Chart::new(vec![read, write])
        .block(Block::bordered().title(title.to_string()))
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .labels(vec![Span::raw("old"), Span::raw("now")]),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, max_value])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(telemetry::format_rate(max_value)),
                ]),
        );
    frame.render_widget(chart, area);
}

fn draw_sparkline(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    history: &VecDeque<f64>,
    max_value: f64,
    color: Color,
) {
    let data = history
        .iter()
        .map(|value| value.round().max(0.0) as u64)
        .collect::<Vec<_>>();
    let sparkline = Sparkline::default()
        .block(Block::bordered().title(title.to_string()))
        .data(&data)
        .max(max_value.round().max(1.0) as u64)
        .style(Style::default().fg(color));
    frame.render_widget(sparkline, area);
}

fn draw_history_bars(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    history: &VecDeque<f64>,
    max_value: f64,
    color: Color,
    kind: MetricKind,
) {
    let visible = (area.width / 4).clamp(2, 40) as usize;
    let skip = history.len().saturating_sub(visible);
    let bars = history
        .iter()
        .skip(skip)
        .enumerate()
        .map(|(index, value)| (index.to_string(), value.round().max(0.0) as u64))
        .collect::<Vec<_>>();
    draw_bar_chart(
        frame,
        area,
        &format!(
            "{} {}",
            title,
            format_metric(history.back().copied().unwrap_or_default(), kind)
        ),
        bars,
        max_value.round().max(1.0) as u64,
        color,
    );
}
