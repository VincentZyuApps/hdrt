use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};

use crate::app::options::{ChartMode, SpinnerStyle, TuiBorder, TuiTab};
use crate::app::spinner::Spinner;
use crate::collector::CollectOptions;
use crate::emoji as emoji_icons;
use crate::i18n::{t, Lang};

mod charts;
mod cpu;
mod draw;
mod panels;
mod screens;
mod selection;
mod state;
mod style;
mod utils;
mod widgets;

use self::state::TuiState;

pub fn run(
    initial_tab: TuiTab,
    initial_chart_mode: ChartMode,
    border: TuiBorder,
    lang: Lang,
    emoji: bool,
    options: CollectOptions,
    interval_ms: u64,
    no_spinner: bool,
    spinner_style: SpinnerStyle,
    color: bool,
    bold: bool,
) -> Result<()> {
    let loading = Spinner::start(
        !no_spinner,
        spinner_style,
        emoji_icons::decorate(emoji, "spinner.tui", t(lang, "spinner.tui")),
    );
    let mut state = TuiState::new(
        initial_tab,
        lang,
        emoji,
        options,
        interval_ms,
        initial_chart_mode,
        border,
        color,
        bold,
    );
    loading.finish();

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

        terminal.draw(|frame| draw::draw(frame, state))?;

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
