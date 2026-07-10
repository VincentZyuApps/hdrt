use std::collections::VecDeque;
use std::time::Duration;

use crossterm::event::KeyCode;

use crate::app::options::{ChartMode, TuiTab};
use crate::collector::{self, CollectOptions};
use crate::hardware::HardwareReport;
use crate::i18n::{t, Lang};
use crate::telemetry::{self, DiskTelemetry, TelemetrySampler, TelemetrySnapshot, HISTORY_LIMIT};

use super::utils::{push_history, tab_index};

const TAB_COUNT: usize = 7;

pub(super) struct TuiState {
    pub(super) tab: usize,
    pub(super) lang: Lang,
    pub(super) emoji: bool,
    pub(super) options: CollectOptions,
    pub(super) interval: Duration,
    pub(super) report: HardwareReport,
    sampler: TelemetrySampler,
    pub(super) latest: TelemetrySnapshot,
    pub(super) cpu_history: VecDeque<f64>,
    pub(super) memory_history: VecDeque<f64>,
    pub(super) disk_read_history: VecDeque<f64>,
    pub(super) disk_write_history: VecDeque<f64>,
    disk_histories: Vec<DiskHistory>,
    pub(super) selected_physical_disk: usize,
    pub(super) selected_disk: usize,
    pub(super) chart_mode: ChartMode,
    pub(super) status: String,
}

impl TuiState {
    pub(super) fn new(
        initial_tab: TuiTab,
        lang: Lang,
        emoji: bool,
        options: CollectOptions,
        interval_ms: u64,
        initial_chart_mode: ChartMode,
    ) -> Self {
        let report = collector::collect_report(options);
        let mut state = Self {
            tab: tab_index(initial_tab),
            lang,
            emoji,
            options,
            interval: Duration::from_millis(telemetry::normalized_interval_ms(interval_ms)),
            report,
            sampler: TelemetrySampler::new(),
            latest: TelemetrySnapshot::default(),
            cpu_history: VecDeque::with_capacity(HISTORY_LIMIT),
            memory_history: VecDeque::with_capacity(HISTORY_LIMIT),
            disk_read_history: VecDeque::with_capacity(HISTORY_LIMIT),
            disk_write_history: VecDeque::with_capacity(HISTORY_LIMIT),
            disk_histories: Vec::new(),
            selected_physical_disk: 0,
            selected_disk: 0,
            chart_mode: initial_chart_mode,
            status: String::new(),
        };
        state.sample();
        state.status = format!(
            "{}: {} ms",
            t(state.lang, "tui.interval"),
            state.interval.as_millis()
        );
        state
    }

    pub(super) fn sample(&mut self) {
        self.latest = self.sampler.sample();
        if self.latest.disks.is_empty() {
            self.selected_disk = 0;
        } else if self.selected_disk >= self.latest.disks.len() {
            self.selected_disk = self.latest.disks.len() - 1;
        }
        if self.report.physical_disks.is_empty() {
            self.selected_physical_disk = 0;
        } else if self.selected_physical_disk >= self.report.physical_disks.len() {
            self.selected_physical_disk = self.report.physical_disks.len() - 1;
        }

        push_history(&mut self.cpu_history, self.latest.cpu_total_percent);
        push_history(&mut self.memory_history, self.latest.memory.used_percent);

        let read_total = self
            .latest
            .disks
            .iter()
            .map(|disk| disk.read_bytes_per_sec)
            .sum::<f64>();
        let write_total = self
            .latest
            .disks
            .iter()
            .map(|disk| disk.write_bytes_per_sec)
            .sum::<f64>();
        push_history(&mut self.disk_read_history, read_total);
        push_history(&mut self.disk_write_history, write_total);

        if self.disk_histories.len() < self.latest.disks.len() {
            self.disk_histories
                .resize_with(self.latest.disks.len(), DiskHistory::default);
        } else if self.disk_histories.len() > self.latest.disks.len() {
            self.disk_histories.truncate(self.latest.disks.len());
        }

        for (history, disk) in self.disk_histories.iter_mut().zip(self.latest.disks.iter()) {
            push_history(&mut history.read, disk.read_bytes_per_sec);
            push_history(&mut history.write, disk.write_bytes_per_sec);
        }
    }

    pub(super) fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Tab | KeyCode::Right => self.next_tab(),
            KeyCode::BackTab | KeyCode::Left => self.prev_tab(),
            KeyCode::Down => self.next_disk(),
            KeyCode::Up => self.prev_disk(),
            KeyCode::Char(ch) => match ch.to_ascii_lowercase() {
                'w' | 'd' => self.next_tab(),
                'a' | 's' => self.prev_tab(),
                'j' => self.next_disk(),
                'k' => self.prev_disk(),
                'z' => self.prev_chart_mode(),
                'c' => self.next_chart_mode(),
                'r' => self.refresh_inventory(),
                'q' => return true,
                _ => {}
            },
            _ => {}
        }

        false
    }

    pub(super) fn selected_disk(&self) -> Option<&DiskTelemetry> {
        self.latest.disks.get(self.selected_disk)
    }

    pub(super) fn selected_disk_history(&self) -> Option<&DiskHistory> {
        self.disk_histories.get(self.selected_disk)
    }

    fn refresh_inventory(&mut self) {
        self.report = collector::collect_report(self.options);
        self.sampler = TelemetrySampler::new();
        self.latest = TelemetrySnapshot::default();
        self.cpu_history.clear();
        self.memory_history.clear();
        self.disk_read_history.clear();
        self.disk_write_history.clear();
        self.disk_histories.clear();
        self.selected_physical_disk = 0;
        self.selected_disk = 0;
        self.sample();
        self.status = t(self.lang, "tui.refreshed").to_string();
    }

    fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % TAB_COUNT;
    }

    fn prev_tab(&mut self) {
        self.tab = (self.tab + TAB_COUNT - 1) % TAB_COUNT;
    }

    fn next_chart_mode(&mut self) {
        self.chart_mode = self.chart_mode.next();
        self.status = format!(
            "{}: {}",
            t(self.lang, "tui.chart_mode"),
            self.chart_mode.label(self.lang)
        );
    }

    fn prev_chart_mode(&mut self) {
        self.chart_mode = self.chart_mode.prev();
        self.status = format!(
            "{}: {}",
            t(self.lang, "tui.chart_mode"),
            self.chart_mode.label(self.lang)
        );
    }

    fn next_disk(&mut self) {
        if self.tab == 1 {
            if !self.report.physical_disks.is_empty() {
                self.selected_physical_disk =
                    (self.selected_physical_disk + 1) % self.report.physical_disks.len();
            }
        } else if self.tab == 2 && !self.latest.disks.is_empty() {
            self.selected_disk = (self.selected_disk + 1) % self.latest.disks.len();
        }
    }

    fn prev_disk(&mut self) {
        if self.tab == 1 {
            if !self.report.physical_disks.is_empty() {
                self.selected_physical_disk = (self.selected_physical_disk
                    + self.report.physical_disks.len()
                    - 1)
                    % self.report.physical_disks.len();
            }
        } else if self.tab == 2 && !self.latest.disks.is_empty() {
            self.selected_disk =
                (self.selected_disk + self.latest.disks.len() - 1) % self.latest.disks.len();
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct DiskHistory {
    pub(super) read: VecDeque<f64>,
    pub(super) write: VecDeque<f64>,
}

impl ChartMode {
    fn next(self) -> Self {
        match self {
            Self::Gauge => Self::Bar,
            Self::Bar => Self::Sparkline,
            Self::Sparkline => Self::Line,
            Self::Line => Self::Scatter,
            Self::Scatter => Self::Gauge,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Gauge => Self::Scatter,
            Self::Bar => Self::Gauge,
            Self::Sparkline => Self::Bar,
            Self::Line => Self::Sparkline,
            Self::Scatter => Self::Line,
        }
    }

    pub(super) fn label(self, lang: Lang) -> &'static str {
        match self {
            Self::Line => t(lang, "tui.chart.line"),
            Self::Scatter => t(lang, "tui.chart.scatter"),
            Self::Bar => t(lang, "tui.chart.bar"),
            Self::Sparkline => t(lang, "tui.chart.sparkline"),
            Self::Gauge => t(lang, "tui.chart.gauge"),
        }
    }
}
