use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};

use crate::app::options::TuiTab;
use crate::collector::CollectOptions;
use crate::i18n::Lang;

mod panels;
mod render;
mod screens;
mod state;
mod utils;
mod widgets;

use self::state::TuiState;

pub fn run(
    initial_tab: TuiTab,
    lang: Lang,
    emoji: bool,
    options: CollectOptions,
    interval_ms: u64,
) -> Result<()> {
    let mut state = TuiState::new(initial_tab, lang, emoji, options, interval_ms);
    let mut terminal = ratatui::init();
    let result = run_inner(&mut terminal, &mut state);
    ratatui::restore();
    result
}

fn run_inner(terminal: &mut ratatui::DefaultTerminal, state: &mut TuiState) -> Result<()> {
    let mut next_sample = Instant::now() + state.interval;

    loop {
        let now = Instant::now();
        if now >= next_sample {
            state.sample();
            next_sample = now + state.interval;
        }

        terminal.draw(|frame| render::draw(frame, state))?;

        let timeout = next_sample
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(200));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && state.handle_key(key.code) {
                    return Ok(());
                }
            }
        }
    }
}
