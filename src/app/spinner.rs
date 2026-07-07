use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossterm::cursor::MoveToColumn;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};

use super::options::SpinnerStyle;

const ASCII_FRAMES: &[&str] = &["/", "|", "\\", "-"];
const UNICODE_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const DOTS_FRAMES: &[&str] = &[".  ", ".. ", "...", " ..", "  .", "   "];

pub struct Spinner {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Spinner {
    pub fn start(enabled: bool, style: SpinnerStyle, message: impl Into<String>) -> Self {
        if !enabled || !io::stderr().is_terminal() {
            return Self::disabled();
        }

        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let message = message.into();
        let frames = frames(style);

        let handle = thread::spawn(move || {
            let mut stderr = io::stderr();
            let mut index = 0usize;

            while !thread_stop.load(Ordering::Relaxed) {
                let _ = execute!(&mut stderr, MoveToColumn(0), Clear(ClearType::CurrentLine));
                let _ = write!(stderr, "{} {}", frames[index % frames.len()], message);
                let _ = stderr.flush();
                index = index.wrapping_add(1);
                thread::sleep(Duration::from_millis(90));
            }

            clear_line(&mut stderr);
        });

        Self {
            stop,
            handle: Some(handle),
        }
    }

    pub fn finish(mut self) {
        self.stop();
    }

    fn disabled() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(true)),
            handle: None,
        }
    }

    fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}

fn frames(style: SpinnerStyle) -> &'static [&'static str] {
    match style {
        SpinnerStyle::Ascii => ASCII_FRAMES,
        SpinnerStyle::Unicode => UNICODE_FRAMES,
        SpinnerStyle::Dots => DOTS_FRAMES,
    }
}

fn clear_line(stderr: &mut io::Stderr) {
    let _ = execute!(&mut *stderr, MoveToColumn(0), Clear(ClearType::CurrentLine));
    let _ = stderr.flush();
}
