use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::i18n::t;
use crate::telemetry;

use super::state::TuiState;
use super::widgets::draw_empty;

const CPU_COLUMN_GAP: usize = 2;
const MIN_GAUGE_WIDTH: usize = 4;

pub(super) fn draw_core_gauges(frame: &mut Frame, area: Rect, state: &TuiState) {
    let cores = &state.latest.cpu_cores_percent;
    if cores.is_empty() {
        draw_empty(frame, area, t(state.lang, "no_data"), state.style);
        return;
    }

    let rows = (cores.len() + 1) / 2;
    let visible_rows = area.height.saturating_sub(2) as usize;
    let column_width = cpu_column_width(area.width);
    let lines = (0..rows.min(visible_rows))
        .map(|row| {
            let mut spans = core_cell(row, cores[row], column_width);
            spans.push(Span::raw(" ".repeat(CPU_COLUMN_GAP)));
            if let Some(usage) = cores.get(row + rows) {
                spans.extend(core_cell(row + rows, *usage, column_width));
            }
            Line::from(spans)
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        Paragraph::new(lines).block(state.style.block().title(t(state.lang, "tui.cpu_cores"))),
        area,
    );
}

fn cpu_column_width(area_width: u16) -> usize {
    let inner_width = area_width.saturating_sub(2) as usize;
    inner_width.saturating_sub(CPU_COLUMN_GAP) / 2
}

fn core_cell(index: usize, usage: f64, width: usize) -> Vec<Span<'static>> {
    let label = format!("CPU{index:<2} ");
    let percent = telemetry::format_percent(usage);
    let fixed_width = label.len() + percent.len() + 1;
    let gauge_width = width.saturating_sub(fixed_width).max(MIN_GAUGE_WIDTH);
    let filled = ((usage.clamp(0.0, 100.0) / 100.0) * gauge_width as f64).round() as usize;
    let empty = gauge_width.saturating_sub(filled);

    vec![
        Span::styled(label, Style::default().fg(Color::Cyan)),
        Span::styled("█".repeat(filled), Style::default().fg(Color::Blue)),
        Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
        Span::raw(format!(" {percent}")),
    ]
}
