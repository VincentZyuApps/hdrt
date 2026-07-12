use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Tabs, Wrap};
use ratatui::Frame;

use crate::emoji;
use crate::i18n::t;

use super::panels::{draw_motherboard, draw_warnings};
use super::screens::{draw_cpu, draw_logical_disk, draw_memory, draw_overview, draw_physical_disk};
use super::state::TuiState;
use super::utils::tab_titles;

pub(super) fn draw(frame: &mut Frame, state: &mut TuiState) {
    let area = frame.area();
    if area.width < 50 || area.height < 12 {
        draw_too_small(frame, area, state);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    draw_tabs(frame, chunks[0], state);

    match state.tab {
        0 => draw_overview(frame, chunks[1], state),
        1 => draw_physical_disk(frame, chunks[1], state),
        2 => draw_logical_disk(frame, chunks[1], state),
        3 => draw_memory(frame, chunks[1], state),
        4 => draw_cpu(frame, chunks[1], state),
        5 => draw_motherboard(frame, chunks[1], state),
        6 => draw_warnings(frame, chunks[1], state),
        _ => {}
    }

    draw_help(frame, chunks[2], state);
}

fn draw_tabs(frame: &mut Frame, area: Rect, state: &TuiState) {
    let titles = tab_titles(state.lang, state.emoji);
    let title = format!(
        "{}  {}: {} | {}: {} ms",
        emoji::decorate(state.emoji, "app.title", "hdrt"),
        t(state.lang, "tui.chart_mode"),
        state.chart_mode.label(state.lang),
        t(state.lang, "tui.interval"),
        state.interval.as_millis()
    );
    let tabs = Tabs::new(titles.iter().cloned())
        .select(state.tab)
        .block(state.style.block().title(Line::from(Span::styled(
            title,
            state.style.text(Color::Cyan, true),
        ))))
        .style(state.style.text(Color::Gray, false))
        .highlight_style(state.style.text(Color::Cyan, true));
    frame.render_widget(tabs, area);
}

fn draw_help(frame: &mut Frame, area: Rect, state: &TuiState) {
    let help = format!("{} | {}", t(state.lang, "tui.help.live"), state.status);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            help,
            state.style.text(Color::Yellow, false),
        ))),
        area,
    );
}

fn draw_too_small(frame: &mut Frame, area: Rect, state: &TuiState) {
    frame.render_widget(
        Paragraph::new(t(state.lang, "tui.too_small"))
            .block(state.style.block().title("hdrt"))
            .wrap(Wrap { trim: true }),
        area,
    );
}
