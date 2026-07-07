use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Tabs, Wrap};

use crate::app::options::TuiTab;
use crate::emoji;
use crate::i18n::{t, Lang};

pub fn run(initial_tab: TuiTab, lang: Lang, emoji: bool) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = run_inner(&mut terminal, initial_tab, lang, emoji);
    ratatui::restore();
    result
}

fn run_inner(
    terminal: &mut ratatui::DefaultTerminal,
    initial_tab: TuiTab,
    lang: Lang,
    emoji: bool,
) -> Result<()> {
    let mut tab = tab_index(initial_tab);
    let titles = [
        label(lang, "section.overview", emoji),
        label(lang, "section.disk", emoji),
        label(lang, "section.memory", emoji),
        label(lang, "section.cpu", emoji),
        label(lang, "section.motherboard", emoji),
        label(lang, "section.health", emoji),
        label(lang, "warnings", emoji),
    ];

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area);

            let tabs =
                Tabs::new(
                    titles
                        .iter()
                        .map(|title| Line::from(Span::raw(title.clone()))),
                )
                .select(tab)
                .block(Block::bordered().title(emoji::decorate(emoji, "app.title", "hdrt")));
            frame.render_widget(tabs, chunks[0]);

            let body = Paragraph::new(vec![
                Line::from(label(lang, "tui.subtitle", emoji)),
                Line::from(label(lang, "tui.memory_hint", emoji)),
                Line::from(""),
                Line::from(label(lang, "tui.placeholder", emoji)),
                Line::from(label(lang, "tui.help", emoji)),
            ])
            .block(Block::bordered().title(titles[tab].clone()))
            .wrap(Wrap { trim: true });
            frame.render_widget(body, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Tab | KeyCode::Right => tab = (tab + 1) % titles.len(),
                        KeyCode::Left => tab = (tab + titles.len() - 1) % titles.len(),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn tab_index(tab: TuiTab) -> usize {
    match tab {
        TuiTab::Overview => 0,
        TuiTab::Disk => 1,
        TuiTab::Memory => 2,
        TuiTab::Cpu => 3,
        TuiTab::Motherboard => 4,
        TuiTab::Health => 5,
        TuiTab::Warnings => 6,
    }
}

fn label(lang: Lang, key: &str, enabled: bool) -> String {
    emoji::decorate(enabled, key, t(lang, key))
}
