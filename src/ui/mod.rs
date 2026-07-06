use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Tabs, Wrap};

use crate::app::options::TuiTab;
use crate::i18n::{t, Lang};

pub fn run(initial_tab: TuiTab, lang: Lang) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = run_inner(&mut terminal, initial_tab, lang);
    ratatui::restore();
    result
}

fn run_inner(
    terminal: &mut ratatui::DefaultTerminal,
    initial_tab: TuiTab,
    lang: Lang,
) -> Result<()> {
    let mut tab = tab_index(initial_tab);
    let titles = [
        t(lang, "section.overview"),
        t(lang, "section.disk"),
        t(lang, "section.memory"),
        t(lang, "section.cpu"),
        t(lang, "section.motherboard"),
        t(lang, "section.health"),
        t(lang, "warnings"),
    ];

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area);

            let tabs = Tabs::new(titles.iter().map(|title| Line::from(Span::raw(*title))))
                .select(tab)
                .block(Block::bordered().title("hdrt"));
            frame.render_widget(tabs, chunks[0]);

            let body = Paragraph::new(vec![
                Line::from(t(lang, "tui.subtitle")),
                Line::from(t(lang, "tui.memory_hint")),
                Line::from(""),
                Line::from(t(lang, "tui.placeholder")),
                Line::from(t(lang, "tui.help")),
            ])
            .block(Block::bordered().title(titles[tab]))
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
