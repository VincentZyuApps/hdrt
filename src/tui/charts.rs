use std::collections::VecDeque;
use std::time::Duration;

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::text::Span;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, Sparkline};
use ratatui::Frame;

use crate::telemetry;

use super::style::TuiStyle;
use super::utils::{format_metric, history_points, MetricKind};
use super::widgets::draw_bar_chart;

pub(super) fn draw_chart(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    history: &VecDeque<f64>,
    max_value: f64,
    color: Color,
    graph_type: GraphType,
    kind: MetricKind,
    interval: Duration,
    style: TuiStyle,
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
        .block(style.block().title(title.to_string()))
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .labels(time_axis_labels(points.len(), interval, area.width)),
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

pub(super) fn draw_dual_chart(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    read_history: &VecDeque<f64>,
    write_history: &VecDeque<f64>,
    max_value: f64,
    graph_type: GraphType,
    interval: Duration,
    style: TuiStyle,
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
        .block(style.block().title(title.to_string()))
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .labels(time_axis_labels(
                    read_points.len().max(write_points.len()),
                    interval,
                    area.width,
                )),
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

pub(super) fn draw_sparkline(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    history: &VecDeque<f64>,
    max_value: f64,
    color: Color,
    style: TuiStyle,
) {
    let data = history
        .iter()
        .map(|value| value.round().max(0.0) as u64)
        .collect::<Vec<_>>();
    let sparkline = Sparkline::default()
        .block(style.block().title(title.to_string()))
        .data(&data)
        .max(max_value.round().max(1.0) as u64)
        .style(Style::default().fg(color));
    frame.render_widget(sparkline, area);
}

pub(super) fn draw_history_bars(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    history: &VecDeque<f64>,
    max_value: f64,
    color: Color,
    kind: MetricKind,
    interval: Duration,
    style: TuiStyle,
) {
    let visible = (area.width / 4).clamp(2, 40) as usize;
    let skip = history.len().saturating_sub(visible);
    let bars = history
        .iter()
        .skip(skip)
        .enumerate()
        .map(|(index, value)| {
            let sample_index = skip + index;
            (
                sample_age_label(history.len(), sample_index, interval),
                value.round().max(0.0) as u64,
            )
        })
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
        style,
    );
}

fn time_axis_labels(sample_count: usize, interval: Duration, width: u16) -> Vec<Span<'static>> {
    let max_age = sample_age(sample_count, 0, interval);
    if max_age <= 0.0 {
        return vec![Span::raw("0s")];
    }

    if width < 42 {
        vec![Span::raw(format_age(max_age)), Span::raw("0s")]
    } else {
        vec![
            Span::raw(format_age(max_age)),
            Span::raw(format_age(max_age / 2.0)),
            Span::raw("0s"),
        ]
    }
}

fn sample_age_label(sample_count: usize, sample_index: usize, interval: Duration) -> String {
    format_age(sample_age(sample_count, sample_index, interval))
}

fn sample_age(sample_count: usize, sample_index: usize, interval: Duration) -> f64 {
    sample_count
        .saturating_sub(1)
        .saturating_sub(sample_index) as f64
        * interval.as_secs_f64()
}

fn format_age(seconds: f64) -> String {
    if seconds > 0.0 && seconds < 1.0 {
        format!("{:.0}ms", seconds * 1_000.0)
    } else {
        format!("{:.0}s", seconds.round())
    }
}
