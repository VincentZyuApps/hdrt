use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Tabs, Wrap};
use ratatui::Frame;

use crate::i18n::{t, Lang};

use super::panels::{draw_health, draw_motherboard, draw_warnings};
use super::screens::{draw_cpu, draw_disk, draw_memory, draw_overview};
use super::state::TuiState;
use super::utils::tab_titles;

pub(super) fn draw(frame: &mut Frame, state: &TuiState) {
    let area = frame.area();
    if area.width < 50 || area.height < 12 {
        draw_too_small(frame, area, state.lang);
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
        1 => draw_disk(frame, chunks[1], state),
        2 => draw_memory(frame, chunks[1], state),
        3 => draw_cpu(frame, chunks[1], state),
        4 => draw_motherboard(frame, chunks[1], state),
        5 => draw_health(frame, chunks[1], state),
        6 => draw_warnings(frame, chunks[1], state),
        _ => {}
    }

    draw_help(frame, chunks[2], state);
}

fn draw_tabs(frame: &mut Frame, area: Rect, state: &TuiState) {
    let titles = tab_titles(state.lang, state.emoji);
    let tabs = Tabs::new(titles.iter().cloned())
        .select(state.tab)
        .block(Block::bordered())
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}

fn draw_help(frame: &mut Frame, area: Rect, state: &TuiState) {
    let help = format!("{} | {}", t(state.lang, "tui.help.live"), state.status);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            help,
            Style::default().fg(Color::Yellow),
        ))),
        area,
    );
}

fn draw_too_small(frame: &mut Frame, area: Rect, lang: Lang) {
    frame.render_widget(
        Paragraph::new(t(lang, "tui.too_small"))
            .block(Block::bordered().title("hdrt"))
            .wrap(Wrap { trim: true }),
        area,
    );
}
