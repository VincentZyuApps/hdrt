use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType};

use crate::app::options::TuiBorder;

#[derive(Debug, Clone, Copy)]
pub(super) struct TuiStyle {
    border: BorderType,
    color: bool,
    bold: bool,
}

impl TuiStyle {
    pub(super) fn new(border: TuiBorder, color: bool, bold: bool) -> Self {
        Self {
            border: match border {
                TuiBorder::Rounded => BorderType::Rounded,
                TuiBorder::Plain => BorderType::Plain,
                TuiBorder::Double => BorderType::Double,
                TuiBorder::Thick => BorderType::Thick,
            },
            color,
            bold,
        }
    }

    pub(super) fn block<'a>(self) -> Block<'a> {
        Block::bordered().border_type(self.border)
    }

    pub(super) fn text(self, color: Color, bold: bool) -> Style {
        let mut style = Style::default();
        if self.color {
            style = style.fg(color);
        }
        if self.bold && bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        style
    }
}
